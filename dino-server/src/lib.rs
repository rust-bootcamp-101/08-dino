mod config;
mod engine;
mod error;
mod router;

pub use config::*;
pub use engine::*;
pub use error::*;
use matchit::Match;
pub use router::*;
use tracing::info;

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::{request::Parts, Response},
    response::IntoResponse,
    routing::any,
    Router,
};
use dashmap::DashMap;
use indexmap::IndexMap;
use tokio::net::TcpListener;

// indexmap 保证路由的注册顺序不变
pub type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    // router key is hostname
    routers: DashMap<String, SwappableAppRouter>,
}

#[derive(Clone)]
pub struct TennetRouter {
    host: String,
    router: SwappableAppRouter,
}

pub async fn start_server(port: u16, routers: Vec<TennetRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    info!("listening on {addr}");
    let listener = TcpListener::bind(addr).await?;
    // /*path 表示匹配所有路由
    let map = DashMap::new();
    for TennetRouter { host, router } in routers {
        map.insert(host, router);
    }
    let state = AppState::new(map);
    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(state);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

// we only support JSON requests and return JSON responses
#[allow(unused)]
async fn handler(
    State(state): State<AppState>,
    parts: Parts,
    Host(mut host): Host,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    let router = get_router_by_host(host, state)?;
    let matched = router.match_it(parts.method.clone(), parts.uri.path())?;
    let req = assemble_req(&parts, query, body, &matched)?;
    let handler = matched.value;
    let worker = JsWorker::try_new(&router.code)?;
    let res = worker.run(handler, req)?;

    Ok(Response::from(res))
}

impl AppState {
    pub fn new(routers: DashMap<String, SwappableAppRouter>) -> Self {
        Self { routers }
    }
}

impl TennetRouter {
    pub fn new(host: String, router: SwappableAppRouter) -> Self {
        Self { host, router }
    }
}

fn get_router_by_host(mut host: String, state: AppState) -> Result<AppRouter, AppError> {
    // get router from state
    let _ = host.split_off(host.find(":").unwrap_or(host.len()));
    let router = state
        .routers
        .get(&host)
        .ok_or(AppError::HostNotFound(host))?
        .load();
    Ok(router)
}

fn assemble_req(
    parts: &Parts,
    query: HashMap<String, String>,
    body: Option<Bytes>,
    matched: &Match<&str>,
) -> Result<Req, AppError> {
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    // convert request data into Req and call handler with a js runtime
    let body = body.and_then(|v| String::from_utf8(v.into()).ok());

    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    let req = Req::builder()
        .method(parts.method.to_string())
        .url(parts.uri.to_string())
        .query(query)
        .params(params)
        .headers(headers)
        .body(body)
        .build();
    Ok(req)
}

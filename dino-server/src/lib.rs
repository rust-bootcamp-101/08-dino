mod config;
mod engine;
mod error;
mod router;

pub use config::*;
pub use engine::*;
pub use error::*;
pub use router::*;

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::request::Parts,
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use dashmap::DashMap;
use indexmap::IndexMap;
use serde_json::json;
use tokio::net::TcpListener;

// indexmap 保证路由的注册顺序不变
pub type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    // router key is hostname
    routers: DashMap<String, SwappableAppRouter>,
}

pub async fn start_server(port: u16, router: DashMap<String, SwappableAppRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    // /*path 表示匹配所有路由
    let state = AppState::new(router);
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
    Query(query): Query<serde_json::Value>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    // get router from state
    host.split_off(host.find(":").unwrap_or(host.len()));
    let router = state
        .routers
        .get(&host)
        .ok_or(AppError::HostNotFound(host))?
        .load();

    // match router with parts.path get a handler
    let matched = router.match_it(parts.method, parts.uri.path())?;
    let handler = matched.value;
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    // convert request data into Req and call handler with a js runtime

    // convert Req into response and return

    let body = match body {
        Some(body) => {
            if body.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_slice(&body)?
            }
        }
        None => serde_json::Value::Null,
    };
    Ok(Json(json!({
        "handler": handler,
        "params": params,
        "query": query,
        "body": body,
    })))
}

impl AppState {
    pub fn new(routers: DashMap<String, SwappableAppRouter>) -> Self {
        Self { routers }
    }
}

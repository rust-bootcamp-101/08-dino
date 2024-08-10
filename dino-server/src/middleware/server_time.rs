use axum::{extract::Request, middleware::Next, response::Response};
use tokio::time::Instant;
use tracing::warn;

use super::{REQUEST_ID_HEADER, SERVER_TIME_HEADER};

/// 中间件，在request和response添加request id
pub async fn set_server_time(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut res = next.run(req).await;
    let elapsed = format!("{}us", start.elapsed().as_micros());
    match elapsed.parse() {
        Ok(v) => {
            res.headers_mut().insert(SERVER_TIME_HEADER, v);
        }
        Err(e) => {
            warn!(
                "Parse elapsed time failed: {} for request {:?}",
                e,
                res.headers().get(REQUEST_ID_HEADER)
            )
        }
    }
    res
}

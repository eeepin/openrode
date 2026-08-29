//! HTTP 请求路由
//!
//! 根据请求路径和方法分发到对应的处理器。

use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::{Full, BodyExt, combinators::BoxBody as HttpBoxBody};
use bytes::Bytes;
use std::sync::Arc;

use crate::server::state::AppState;
use crate::server::handler;

type BoxBody = HttpBoxBody<Bytes, std::convert::Infallible>;

/// 处理 HTTP 请求
pub async fn handle_request(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = match (method.as_str(), path.as_str()) {
        // 事件流端点
        ("GET", "/event") => handler::global_events(req, state).await,

        // 会话端点
        ("POST", "/session") => handler::create_session(req, state).await,
        ("GET", "/session") => handler::list_sessions(req, state).await,
        ("GET", path) if path.starts_with("/session/") && path.ends_with("/event") => {
            handler::session_events(req, state).await
        }
        ("GET", path) if path.starts_with("/session/") && path.ends_with("/message") => {
            handler::list_messages(req, state).await
        }
        ("POST", path) if path.starts_with("/session/") && path.ends_with("/prompt") => {
            handler::send_prompt(req, state).await
        }
        ("POST", path) if path.starts_with("/session/") && path.ends_with("/abort") => {
            handler::abort_session(req, state).await
        }
        ("GET", path) if path.matches('/').count() == 2 && path.starts_with("/session/") => {
            handler::get_session(req, state).await
        }
        ("DELETE", path) if path.matches('/').count() == 2 && path.starts_with("/session/") => {
            handler::delete_session(req, state).await
        }

        // 权限端点
        ("GET", "/permission") => handler::list_permissions(req, state).await,
        ("POST", "/permission/reply") => handler::reply_permission(req, state).await,

        // 配置端点
        ("GET", "/config") => handler::get_config(req, state).await,
        ("PUT", "/config") => handler::update_config(req, state).await,

        // 模型端点
        ("GET", "/model") => handler::list_models(req, state).await,

        // 404 未找到
        _ => {
            let body: BoxBody = Full::new(Bytes::from("Not Found")).boxed();
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body)
                .unwrap())
        }
    };

    response.or_else(|e: hyper::Error| {
        let body: BoxBody = Full::new(Bytes::from(format!("Error: {}", e))).boxed();
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap())
    })
}

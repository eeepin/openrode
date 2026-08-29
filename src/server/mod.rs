//! HTTP 服务器模块
//!
//! 基于 hyper 实现的 REST API 服务器，提供会话管理、消息处理和实时事件推送。

mod event;
mod handler;
mod router;
mod state;

pub use state::AppState;

use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::storage::Storage;

/// 启动 HTTP 服务器
pub async fn start_server(storage: Arc<dyn Storage>, addr: &str) -> Result<()> {
    let state = AppState::new(storage);
    let state = Arc::new(state);

    let listener = TcpListener::bind(addr).await?;
    println!("服务器启动于 http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let state = state.clone();
                async move { router::handle_request(req, state).await }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("服务器错误: {}", err);
            }
        });
    }
}

//! 事件总线和 SSE 流
//!
//! 提供应用级事件发布/订阅机制，支持 SSE (Server-Sent Events) 实时推送。

use bytes::Bytes;
use futures::stream::{self, Stream};
use http_body::Frame;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// 应用事件
#[derive(Debug, Clone, Serialize)]
pub struct AppEvent {
    /// 事件序号（全局递增）
    pub id: u64,
    /// 事件类型
    pub kind: String,
    /// 事件数据
    pub payload: serde_json::Value,
}

/// 事件总线
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
    seq: Arc<RwLock<u64>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            seq: Arc::new(RwLock::new(0)),
        }
    }

    /// 发布事件
    pub async fn emit(&self, kind: impl Into<String>, payload: serde_json::Value) {
        let mut seq = self.seq.write().await;
        *seq += 1;
        let event = AppEvent {
            id: *seq,
            kind: kind.into(),
            payload,
        };
        // 忽略发送错误（没有接收者时）
        let _ = self.sender.send(event);
    }

    /// 订阅事件流
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    /// 获取当前事件序号
    pub async fn current_seq(&self) -> u64 {
        *self.seq.read().await
    }
}

/// 创建 SSE 事件流
pub fn create_sse_stream(
    rx: broadcast::Receiver<AppEvent>,
) -> impl Stream<Item = Result<Frame<Bytes>, std::convert::Infallible>> {
    stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                    let sse_data = format!(
                        "id: {}\nevent: {}\ndata: {}\n\n",
                        event.id, event.kind, data
                    );
                    return Some((Ok(Frame::data(Bytes::from(sse_data))), rx));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // 跳过滞后的消息，继续接收
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // 通道关闭，结束流
                    return None;
                }
            }
        }
    })
}

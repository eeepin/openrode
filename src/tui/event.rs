//! TUI SSE 事件处理

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::session::{Message, Session};

/// SSE 事件
#[derive(Debug, Clone, Deserialize)]
pub struct SseEvent {
    pub id: Option<u64>,
    pub event: Option<String>,
    pub data: String,
}

/// 应用事件（从 SSE 转换）
#[derive(Debug)]
pub enum AppEvent {
    /// 会话创建
    SessionCreated(Session),
    /// 会话删除
    SessionDeleted(String),
    /// 新消息
    MessageCreated(Message),
    /// 权限请求
    PermissionRequest {
        id: String,
        tool: String,
        operation: String,
    },
    /// 权限回复
    PermissionReply { id: String, allow: bool },
    /// 错误
    Error(String),
    /// 连接状态
    Connected,
    Disconnected,
}

/// SSE 客户端
pub struct SseClient {
    client: Client,
    base_url: String,
}

impl SseClient {
    /// 创建新的 SSE 客户端
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// 订阅全局事件流
    pub async fn subscribe_events(&self, tx: mpsc::UnboundedSender<AppEvent>) -> Result<()> {
        let url = format!("{}/event", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to connect to event stream"));
        }

        tx.send(AppEvent::Connected)?;

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // 解析 SSE 事件
            while let Some(event) = parse_sse_event(&mut buffer) {
                if let Some(app_event) = convert_sse_to_app_event(&event) {
                    if tx.send(app_event).is_err() {
                        return Ok(());
                    }
                }
            }
        }

        tx.send(AppEvent::Disconnected)?;
        Ok(())
    }

    /// 创建会话
    pub async fn create_session(&self, model: Option<String>) -> Result<Session> {
        let url = format!("{}/session", self.base_url);
        let body = serde_json::json!({
            "model": model.unwrap_or_else(|| "default".to_string())
        });

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to create session"));
        }

        let session: Session = response.json().await?;
        Ok(session)
    }

    /// 发送消息
    pub async fn send_prompt(&self, session_id: &str, prompt: &str) -> Result<()> {
        let url = format!("{}/session/{}/prompt", self.base_url, session_id);
        let body = serde_json::json!({
            "prompt": prompt
        });

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to send prompt"));
        }

        Ok(())
    }

    /// 回复权限请求
    pub async fn reply_permission(&self, request_id: &str, allow: bool) -> Result<()> {
        let url = format!("{}/permission/reply", self.base_url);
        let body = serde_json::json!({
            "request_id": request_id,
            "allow": allow
        });

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to reply to permission request"));
        }

        Ok(())
    }

    /// 列出会话
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let url = format!("{}/session", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to list sessions"));
        }

        let sessions: Vec<Session> = response.json().await?;
        Ok(sessions)
    }
}

/// 解析 SSE 事件
fn parse_sse_event(buffer: &mut String) -> Option<SseEvent> {
    // 查找事件结束标记（两个换行）
    if let Some(end_pos) = buffer.find("\n\n") {
        let event_str = buffer[..end_pos].to_string();
        buffer.drain(..end_pos + 2);

        let mut event = SseEvent {
            id: None,
            event: None,
            data: String::new(),
        };

        for line in event_str.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "id" => event.id = value.parse().ok(),
                    "event" => event.event = Some(value.to_string()),
                    "data" => {
                        if !event.data.is_empty() {
                            event.data.push('\n');
                        }
                        event.data.push_str(value);
                    }
                    _ => {}
                }
            }
        }

        Some(event)
    } else {
        None
    }
}

/// 将 SSE 事件转换为应用事件
fn convert_sse_to_app_event(event: &SseEvent) -> Option<AppEvent> {
    let event_type = event.event.as_deref()?;
    let data = &event.data;

    match event_type {
        "session.created" => {
            if let Ok(session) = serde_json::from_str::<Session>(data) {
                Some(AppEvent::SessionCreated(session))
            } else {
                None
            }
        }
        "session.deleted" => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                json.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| AppEvent::SessionDeleted(id.to_string()))
            } else {
                None
            }
        }
        "message.created" => {
            if let Ok(message) = serde_json::from_str::<Message>(data) {
                Some(AppEvent::MessageCreated(message))
            } else {
                None
            }
        }
        "permission.request" => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                let id = json.get("id")?.as_str()?.to_string();
                let tool = json.get("tool")?.as_str()?.to_string();
                let operation = json.get("operation")?.as_str()?.to_string();
                Some(AppEvent::PermissionRequest {
                    id,
                    tool,
                    operation,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

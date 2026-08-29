//! HTTP 请求处理器
//!
//! 实现所有 REST API 端点的处理逻辑。

use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::{Full, BodyExt, StreamBody, combinators::BoxBody as HttpBoxBody};
use bytes::Bytes;
use std::sync::Arc;

use crate::server::state::AppState;
use crate::server::event::create_sse_stream;
use crate::session::{Session, Message, Role, Part};

type BoxBody = HttpBoxBody<Bytes, std::convert::Infallible>;

/// 辅助函数：创建 JSON 响应
fn json_response<T: serde::Serialize>(status: StatusCode, data: T) -> Response<BoxBody> {
    let json = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)).map_err(|_| unreachable!()).boxed())
        .unwrap()
}

/// 辅助函数：创建错误响应
fn error_response(status: StatusCode, message: &str) -> Response<BoxBody> {
    let error = serde_json::json!({ "error": message });
    json_response(status, error)
}

/// 辅助函数：从请求体读取 JSON
async fn read_json<T: serde::de::DeserializeOwned>(req: Request<Incoming>) -> Result<T, String> {
    let body = req.into_body();
    let bytes = body.collect().await
        .map_err(|e| format!("读取请求体失败: {}", e))?
        .to_bytes();
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("解析 JSON 失败: {}", e))
}

/// 辅助函数：从路径中提取会话 ID
fn extract_session_id(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 && parts[0] == "session" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

// ============================================================================
// 事件端点
// ============================================================================

/// GET /event - 全局事件流
pub async fn global_events(
    _req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let rx = state.event_bus.subscribe();
    let stream = create_sse_stream(rx);
    let body = StreamBody::new(stream).map_err(|_| unreachable!()).boxed();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(body)
        .unwrap())
}

/// GET /session/:id/event - 会话级事件流
pub async fn session_events(
    _req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // 目前与全局事件流相同，后续可以实现会话级过滤
    let rx = state.event_bus.subscribe();
    let stream = create_sse_stream(rx);
    let body = StreamBody::new(stream).map_err(|_| unreachable!()).boxed();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(body)
        .unwrap())
}

// ============================================================================
// 会话端点
// ============================================================================

/// POST /session - 创建新会话
pub async fn create_session(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // 解析请求体
    let create_req: serde_json::Value = match read_json(req).await {
        Ok(v) => v,
        Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
    };

    let model = create_req.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let session = Session::new(model);

    // 保存到存储
    match state.storage.create_session(&session).await {
        Ok(()) => {
            // 发布事件
            state.event_bus.emit(
                "session.created",
                serde_json::to_value(&session).unwrap_or_default(),
            ).await;

            Ok(json_response(StatusCode::CREATED, session))
        }
        Err(e) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("创建会话失败: {}", e),
        )),
    }
}

/// GET /session - 列出所有会话
pub async fn list_sessions(
    _req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    match state.storage.list_sessions().await {
        Ok(sessions) => Ok(json_response(StatusCode::OK, sessions)),
        Err(e) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("列出会话失败: {}", e),
        )),
    }
}

/// GET /session/:id - 获取单个会话
pub async fn get_session(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = match extract_session_id(req.uri().path()) {
        Some(id) => id,
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "无效的会话 ID")),
    };

    match state.storage.get_session(&session_id).await {
        Ok(Some(session_data)) => Ok(json_response(StatusCode::OK, session_data.session)),
        Ok(None) => Ok(error_response(StatusCode::NOT_FOUND, "会话不存在")),
        Err(e) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("获取会话失败: {}", e),
        )),
    }
}

/// DELETE /session/:id - 删除会话
pub async fn delete_session(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = match extract_session_id(req.uri().path()) {
        Some(id) => id,
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "无效的会话 ID")),
    };

    // 先中止活跃的代理循环
    state.abort_agent(&session_id).await;

    match state.storage.delete_session(&session_id).await {
        Ok(()) => {
            state.event_bus.emit(
                "session.deleted",
                serde_json::json!({ "id": session_id }),
            ).await;

            Ok(json_response(StatusCode::OK, serde_json::json!({ "ok": true })))
        }
        Err(e) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("删除会话失败: {}", e),
        )),
    }
}

/// POST /session/:id/prompt - 发送消息到会话
pub async fn send_prompt(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = match extract_session_id(req.uri().path()) {
        Some(id) => id,
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "无效的会话 ID")),
    };

    // 解析请求体
    let prompt_req: serde_json::Value = match read_json(req).await {
        Ok(v) => v,
        Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
    };

    let prompt = match prompt_req.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "缺少 prompt 字段")),
    };

    // 检查会话是否存在
    let _session_data = match state.storage.get_session(&session_id).await {
        Ok(Some(data)) => data,
        Ok(None) => return Ok(error_response(StatusCode::NOT_FOUND, "会话不存在")),
        Err(e) => return Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("获取会话失败: {}", e),
        )),
    };

    // 检查是否已有活跃的代理循环
    if state.has_active_agent(&session_id).await {
        return Ok(error_response(StatusCode::CONFLICT, "会话正在处理中"));
    }

    // 保存用户消息
    let msg = Message::new(
        session_id.clone(),
        Role::User,
        vec![Part::Text { content: prompt.clone() }],
    );

    if let Err(e) = state.storage.append_message(&session_id, &msg).await {
        return Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("保存消息失败: {}", e),
        ));
    }

    // 发布事件
    state.event_bus.emit(
        "message.created",
        serde_json::to_value(&msg).unwrap_or_default(),
    ).await;

    // TODO: 启动代理循环处理消息
    // 这里暂时只返回成功，后续需要集成 agent loop

    Ok(json_response(StatusCode::ACCEPTED, serde_json::json!({ "ok": true })))
}

/// POST /session/:id/abort - 中止会话的代理循环
pub async fn abort_session(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = match extract_session_id(req.uri().path()) {
        Some(id) => id,
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "无效的会话 ID")),
    };

    if state.abort_agent(&session_id).await {
        state.event_bus.emit(
            "session.aborted",
            serde_json::json!({ "id": session_id }),
        ).await;

        Ok(json_response(StatusCode::OK, serde_json::json!({ "ok": true })))
    } else {
        Ok(error_response(StatusCode::NOT_FOUND, "没有活跃的代理循环"))
    }
}

// ============================================================================
// 消息端点
// ============================================================================

/// GET /session/:id/message - 获取会话的所有消息
pub async fn list_messages(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = match extract_session_id(req.uri().path()) {
        Some(id) => id,
        None => return Ok(error_response(StatusCode::BAD_REQUEST, "无效的会话 ID")),
    };

    match state.storage.get_session(&session_id).await {
        Ok(Some(session_data)) => Ok(json_response(StatusCode::OK, session_data.messages)),
        Ok(None) => Ok(error_response(StatusCode::NOT_FOUND, "会话不存在")),
        Err(e) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("获取消息失败: {}", e),
        )),
    }
}

// ============================================================================
// 权限端点
// ============================================================================

/// GET /permission - 列出待处理的权限请求
pub async fn list_permissions(
    _req: Request<Incoming>,
    _state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // TODO: 实现权限请求队列
    Ok(json_response(StatusCode::OK, serde_json::json!([])))
}

/// POST /permission/reply - 回复权限请求
pub async fn reply_permission(
    req: Request<Incoming>,
    _state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // 解析请求体
    let reply_req: serde_json::Value = match read_json(req).await {
        Ok(v) => v,
        Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
    };

    // TODO: 实现权限回复逻辑
    let _request_id = reply_req.get("request_id").and_then(|v| v.as_str());
    let _allow = reply_req.get("allow").and_then(|v| v.as_bool()).unwrap_or(false);

    Ok(json_response(StatusCode::OK, serde_json::json!({ "ok": true })))
}

// ============================================================================
// 配置端点
// ============================================================================

/// GET /config - 获取当前配置
pub async fn get_config(
    _req: Request<Incoming>,
    _state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // TODO: 从配置文件或数据库加载配置
    let config = serde_json::json!({
        "default_model": "default",
        "default_provider": "openai",
    });

    Ok(json_response(StatusCode::OK, config))
}

/// PUT /config - 更新配置
pub async fn update_config(
    req: Request<Incoming>,
    _state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // 解析请求体
    let _config: serde_json::Value = match read_json(req).await {
        Ok(v) => v,
        Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
    };

    // TODO: 保存配置到配置文件或数据库

    Ok(json_response(StatusCode::OK, serde_json::json!({ "ok": true })))
}

// ============================================================================
// 模型端点
// ============================================================================

/// GET /model - 列出可用模型
pub async fn list_models(
    _req: Request<Incoming>,
    _state: Arc<AppState>,
) -> Result<Response<BoxBody>, hyper::Error> {
    use crate::provider::models::ModelCatalog;

    let catalog = ModelCatalog::new();
    let models: Vec<_> = catalog
        .list()
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "name": m.name,
                "provider": m.provider,
                "context_window": m.context_window,
                "max_output_tokens": m.max_output_tokens,
                "supports_tools": m.supports_tools,
                "supports_vision": m.supports_vision,
                "supports_streaming": m.supports_streaming,
            })
        })
        .collect();

    Ok(json_response(StatusCode::OK, models))
}

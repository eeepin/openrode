//! 应用状态管理
//!
//! 定义服务器的全局状态，包括存储、事件总线、代理循环管理器等。

use crate::server::event::EventBus;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    /// 存储层
    pub storage: Arc<dyn Storage>,
    /// 事件总线
    pub event_bus: EventBus,
    /// 活跃的代理循环（按会话 ID）
    active_agents: Arc<RwLock<HashMap<String, AgentHandle>>>,
}

/// 代理循环句柄
struct AgentHandle {
    abort_handle: tokio::task::JoinHandle<()>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            event_bus: EventBus::new(1024),
            active_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查会话是否有活跃的代理循环
    pub async fn has_active_agent(&self, session_id: &str) -> bool {
        self.active_agents.read().await.contains_key(session_id)
    }

    /// 注册活跃的代理循环
    pub async fn register_agent(&self, session_id: String, abort_handle: tokio::task::JoinHandle<()>) {
        self.active_agents.write().await.insert(
            session_id,
            AgentHandle { abort_handle },
        );
    }

    /// 移除活跃的代理循环
    pub async fn unregister_agent(&self, session_id: &str) {
        self.active_agents.write().await.remove(session_id);
    }

    /// 中止会话的代理循环
    pub async fn abort_agent(&self, session_id: &str) -> bool {
        if let Some(agent) = self.active_agents.write().await.remove(session_id) {
            agent.abort_handle.abort();
            true
        } else {
            false
        }
    }
}

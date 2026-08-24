use anyhow::Result;
use async_trait::async_trait;

use crate::session::{Session, SessionData};

pub mod file;

/// 存储 trait — 会话和消息的持久化接口
#[async_trait]
#[allow(dead_code)]
pub trait Storage: Send + Sync {
    /// 创建新会话
    async fn create_session(&self, session: &Session) -> Result<()>;

    /// 获取会话（含所有消息）
    async fn get_session(&self, id: &str) -> Result<Option<SessionData>>;

    /// 列出所有会话（按更新时间倒序）
    async fn list_sessions(&self) -> Result<Vec<Session>>;

    /// 追加消息到会话
    async fn append_message(
        &self,
        session_id: &str,
        message: &crate::session::Message,
    ) -> Result<()>;

    /// 更新会话元信息（标题等）
    async fn update_session(&self, session: &Session) -> Result<()>;

    /// 删除会话
    async fn delete_session(&self, id: &str) -> Result<()>;

    /// 获取最近的会话 ID（用于 --continue）
    async fn latest_session_id(&self) -> Result<Option<String>>;
}

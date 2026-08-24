use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::Storage;
use crate::session::{Message, Session, SessionData};

/// 基于文件系统的存储
///
/// 目录结构：
/// ```
/// ~/.openrode/sessions/<session_id>.json
/// ```
///
/// 每个文件是一个 SessionData JSON（包含 session 元信息和所有 messages）
pub struct FileStorage {
    base_dir: PathBuf,
}

impl FileStorage {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// 使用默认目录 ~/.openrode/sessions
    pub async fn default_storage() -> Result<Self> {
        let home = dirs::home_dir().context("无法获取 home 目录")?;
        let base_dir = home.join(".openrode").join("sessions");
        fs::create_dir_all(&base_dir)
            .await
            .context("创建存储目录失败")?;
        Ok(Self::new(base_dir))
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("{id}.json"))
    }

    async fn read_session_data(&self, path: &Path) -> Result<SessionData> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("读取会话文件失败: {}", path.display()))?;
        serde_json::from_str(&content).context("解析会话数据失败")
    }

    async fn write_session_data(&self, path: &Path, data: &SessionData) -> Result<()> {
        let content = serde_json::to_string_pretty(data).context("序列化会话数据失败")?;
        fs::write(path, content)
            .await
            .with_context(|| format!("写入会话文件失败: {}", path.display()))?;
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn create_session(&self, session: &Session) -> Result<()> {
        let data = SessionData {
            session: session.clone(),
            messages: Vec::new(),
        };
        let path = self.session_path(&session.id);
        self.write_session_data(&path, &data).await
    }

    async fn get_session(&self, id: &str) -> Result<Option<SessionData>> {
        let path = self.session_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let data = self.read_session_data(&path).await?;
        Ok(Some(data))
    }

    async fn list_sessions(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        let mut entries = fs::read_dir(&self.base_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match self.read_session_data(&path).await {
                    Ok(data) => sessions.push(data.session),
                    Err(_) => continue, // 跳过损坏的文件
                }
            }
        }

        // 按更新时间倒序
        sessions.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
        Ok(sessions)
    }

    async fn append_message(&self, session_id: &str, message: &Message) -> Result<()> {
        let path = self.session_path(session_id);
        let mut data = self.read_session_data(&path).await?;
        data.messages.push(message.clone());
        data.session.updated_at = Utc::now();

        // 如果还没有标题，用第一条用户消息的前 50 字符作为标题
        if data.session.title.is_empty()
            && let Some(first_user_msg) = data
                .messages
                .iter()
                .find(|m| m.role == crate::session::Role::User)
        {
            let text = first_user_msg.text();
            let title = if text.len() > 50 {
                format!("{}...", &text[..47])
            } else {
                text
            };
            data.session.title = title;
        }

        self.write_session_data(&path, &data).await
    }

    async fn update_session(&self, session: &Session) -> Result<()> {
        let path = self.session_path(&session.id);
        let mut data = self.read_session_data(&path).await?;
        data.session = session.clone();
        self.write_session_data(&path, &data).await
    }

    async fn delete_session(&self, id: &str) -> Result<()> {
        let path = self.session_path(id);
        if path.exists() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    async fn latest_session_id(&self) -> Result<Option<String>> {
        let sessions = self.list_sessions().await?;
        Ok(sessions.first().map(|s| s.id.clone()))
    }
}

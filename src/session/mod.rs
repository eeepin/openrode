use chrono::{DateTime, Utc};
use hillm::types::{
    AssistantMessage, FunctionCall, Message as HillmMessage, MessageContent, SystemMessage,
    ToolCall, ToolMessage as HillmToolMessage, UserMessage,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(model: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new().to_string(),
            title: String::new(),
            model,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// 消息内容部件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text {
        content: String,
    },
    ToolUse {
        id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        tool_use_id: String,
        name: String,
        content: String,
        is_error: bool,
    },
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: Role,
    pub parts: Vec<Part>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(session_id: String, role: Role, parts: Vec<Part>) -> Self {
        Self {
            id: Ulid::new().to_string(),
            session_id,
            role,
            parts,
            created_at: Utc::now(),
        }
    }

    pub fn user_text(session_id: String, text: String) -> Self {
        Self::new(session_id, Role::User, vec![Part::Text { content: text }])
    }

    pub fn system_text(session_id: String, text: String) -> Self {
        Self::new(session_id, Role::System, vec![Part::Text { content: text }])
    }

    pub fn assistant_tool_calls(
        session_id: String,
        text: Option<String>,
        calls: &[ToolCall],
    ) -> Self {
        let mut parts = Vec::new();
        if let Some(text) = text
            && !text.is_empty()
        {
            parts.push(Part::Text { content: text });
        }
        for call in calls {
            parts.push(Part::ToolUse {
                id: call.id.clone(),
                name: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
            });
        }
        Self::new(session_id, Role::Assistant, parts)
    }

    pub fn tool_result(
        session_id: String,
        tool_use_id: String,
        name: String,
        content: String,
        is_error: bool,
    ) -> Self {
        Self::new(
            session_id,
            Role::Tool,
            vec![Part::ToolResult {
                tool_use_id,
                name,
                content,
                is_error,
            }],
        )
    }

    /// 获取文本内容（合并所有 Text parts）
    pub fn text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| match p {
                Part::Text { content } => Some(content.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// 转换为 hillm 的 Message 类型（用于发送给 LLM）
    pub fn to_hillm(&self) -> HillmMessage {
        match self.role {
            Role::System => HillmMessage::System(SystemMessage {
                content: MessageContent::Text(self.text()),
                name: None,
            }),
            Role::User => HillmMessage::User(UserMessage {
                content: MessageContent::Text(self.text()),
                name: None,
            }),
            Role::Assistant => {
                let text = self.text();
                let tool_calls: Vec<ToolCall> = self
                    .parts
                    .iter()
                    .filter_map(|p| match p {
                        Part::ToolUse {
                            id,
                            name,
                            arguments,
                        } => Some(ToolCall {
                            id: id.clone(),
                            call_type: hillm::types::ToolType::Function,
                            function: FunctionCall {
                                name: name.clone(),
                                arguments: arguments.clone(),
                            },
                        }),
                        _ => None,
                    })
                    .collect();
                HillmMessage::Assistant(AssistantMessage {
                    content: if text.is_empty() {
                        None
                    } else {
                        Some(MessageContent::Text(text))
                    },
                    tool_calls: if tool_calls.is_empty() {
                        None
                    } else {
                        Some(tool_calls)
                    },
                    ..Default::default()
                })
            }
            Role::Tool => {
                // Tool messages: extract ToolResult parts
                let content = self
                    .parts
                    .iter()
                    .filter_map(|p| match p {
                        Part::ToolResult { content, .. } => Some(content.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let tool_call_id = self
                    .parts
                    .iter()
                    .find_map(|p| match p {
                        Part::ToolResult { tool_use_id, .. } => Some(tool_use_id.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                HillmMessage::Tool(HillmToolMessage {
                    content: MessageContent::Text(content),
                    tool_call_id,
                    name: None,
                })
            }
        }
    }
}

/// 会话中完整的消息列表（用于序列化/反序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session: Session,
    pub messages: Vec<Message>,
}

/// 从现有会话分叉出新会话
///
/// 复制源会话从开始到指定消息（包含）的所有消息到新会话。
#[allow(dead_code)]
pub fn fork_session_data(source: &SessionData, until_message_id: &str) -> Option<SessionData> {
    let cut_index = source
        .messages
        .iter()
        .position(|m| m.id == until_message_id)?;

    let new_session = Session::new(source.session.model.clone());

    // 复制消息，但重新生成 ID
    let new_messages: Vec<Message> = source.messages[..=cut_index]
        .iter()
        .map(|msg| Message {
            id: Ulid::new().to_string(),
            session_id: new_session.id.clone(),
            role: msg.role.clone(),
            parts: msg.parts.clone(),
            created_at: msg.created_at,
        })
        .collect();

    Some(SessionData {
        session: new_session,
        messages: new_messages,
    })
}

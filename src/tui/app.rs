//! TUI 应用状态管理

use crate::session::{Message, Part, Role};

/// 权限请求
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: String,
    pub tool: String,
    pub operation: String,
}

/// 应用状态
pub struct App {
    /// 当前会话 ID
    pub session_id: Option<String>,
    /// 消息列表
    pub messages: Vec<Message>,
    /// 输入框内容
    pub input: String,
    /// 当前权限请求
    pub permission_request: Option<PermissionRequest>,
    /// 是否应该退出
    pub should_quit: bool,
    /// 状态消息
    pub status: String,
    /// 是否正在加载
    pub loading: bool,
    /// 消息滚动位置
    pub scroll: u16,
}

impl App {
    /// 创建新的 App 实例
    pub fn new() -> Self {
        Self {
            session_id: None,
            messages: Vec::new(),
            input: String::new(),
            permission_request: None,
            should_quit: false,
            status: "准备就绪".to_string(),
            loading: false,
            scroll: 0,
        }
    }

    /// 提交输入
    pub fn submit(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return None;
        }
        let input = std::mem::take(&mut self.input);
        Some(input)
    }

    /// 添加消息
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        // 自动滚动到底部
        self.scroll = self.messages.len() as u16;
    }

    /// 处理权限请求
    pub fn handle_permission_request(&mut self, request: PermissionRequest) {
        self.permission_request = Some(request);
    }

    /// 允许权限请求
    pub fn allow_permission(&mut self) -> Option<String> {
        self.permission_request.take().map(|r| r.id)
    }

    /// 拒绝权限请求
    pub fn deny_permission(&mut self) -> Option<String> {
        self.permission_request.take().map(|r| r.id)
    }

    /// 设置状态消息
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    /// 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
        if loading {
            self.status = "处理中...".to_string();
        } else if self.status == "处理中..." {
            self.status = "准备就绪".to_string();
        }
    }

    /// 向上滚动
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// 向下滚动
    pub fn scroll_down(&mut self) {
        let max_scroll = self.messages.len() as u16;
        self.scroll = (self.scroll + 1).min(max_scroll);
    }

    /// 清空消息
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll = 0;
    }

    /// 获取格式化后的消息用于显示
    pub fn get_display_messages(&self) -> Vec<DisplayMessage> {
        self.messages
            .iter()
            .map(|msg| DisplayMessage::from_message(msg))
            .collect()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// 用于显示的消息格式
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub role: String,
    pub content: String,
    pub is_user: bool,
    pub is_assistant: bool,
    pub is_tool: bool,
}

impl DisplayMessage {
    /// 从 Message 转换为 DisplayMessage
    pub fn from_message(msg: &Message) -> Self {
        let role = match msg.role {
            Role::User => "用户",
            Role::Assistant => "助手",
            Role::System => "系统",
            Role::Tool => "工具",
        };

        let content = msg
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { content } => Some(content.clone()),
                Part::ToolUse { name, arguments, .. } => {
                    Some(format!("调用工具: {}({})", name, arguments))
                }
                Part::ToolResult { content, is_error, .. } => {
                    if *is_error {
                        Some(format!("错误: {}", content))
                    } else {
                        Some(content.clone())
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            role: role.to_string(),
            content,
            is_user: msg.role == Role::User,
            is_assistant: msg.role == Role::Assistant,
            is_tool: msg.role == Role::Tool,
        }
    }
}

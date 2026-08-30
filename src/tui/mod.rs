//! TUI 模块

mod app;
mod event;
mod ui;

pub use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use self::event::{AppEvent, SseClient};

/// TUI 客户端
pub struct TuiClient {
    base_url: String,
    sse_client: SseClient,
}

impl TuiClient {
    /// 创建新的 TUI 客户端
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let sse_client = SseClient::new(base_url.clone());
        Self {
            base_url,
            sse_client,
        }
    }

    /// 运行 TUI
    pub async fn run(&self) -> anyhow::Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 创建应用和事件通道
        let mut app = App::new();
        let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

        // 启动 SSE 事件订阅
        let sse_client = SseClient::new(self.base_url.clone());
        let sse_tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = sse_client.subscribe_events(sse_tx).await {
                eprintln!("SSE error: {}", e);
            }
        });

        // 创建会话
        match self.sse_client.create_session(None).await {
            Ok(session) => {
                app.session_id = Some(session.id.clone());
                app.set_status(format!("已连接到会话: {}...{}", &session.id[..8], &session.id[session.id.len()-4..]));
            }
            Err(e) => {
                app.set_status(format!("创建会话失败: {}", e));
            }
        }

        // 主循环
        let result = self.main_loop(&mut terminal, &mut app, &mut rx, tx).await;

        // 恢复终端
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    /// 主循环
    async fn main_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        app: &mut App,
        rx: &mut mpsc::UnboundedReceiver<AppEvent>,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> anyhow::Result<()> {
        loop {
            // 渲染 UI
            terminal.draw(|f| ui::render(f, app))?;

            // 处理事件（非阻塞）
            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            // 退出
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                if app.permission_request.is_none() {
                                    app.should_quit = true;
                                }
                            }
                            // Ctrl+C 强制退出
                            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                app.should_quit = true;
                            }
                            // 回车提交
                            KeyCode::Enter => {
                                if let Some(request) = &app.permission_request {
                                    // 权限对话框中，回车允许
                                    let request_id = request.id.clone();
                                    if let Err(e) = self.sse_client.reply_permission(&request_id, true).await {
                                        app.set_status(format!("回复权限失败: {}", e));
                                    }
                                    app.allow_permission();
                                } else if let Some(prompt) = app.submit() {
                                    // 输入框中，回车发送消息
                                    app.set_loading(true);
                                    let session_id = app.session_id.clone();
                                    let sse_client = SseClient::new(self.base_url.clone());
                                    let tx_clone = tx.clone();

                                    tokio::spawn(async move {
                                        if let Some(session_id) = session_id {
                                            if let Err(e) = sse_client.send_prompt(&session_id, &prompt).await {
                                                let _ = tx_clone.send(AppEvent::Error(format!("发送失败: {}", e)));
                                            }
                                        }
                                    });
                                }
                            }
                            // 权限对话框中的拒绝
                            KeyCode::Char('n') | KeyCode::Char('N') if app.permission_request.is_some() => {
                                let request_id = app.deny_permission().unwrap();
                                if let Err(e) = self.sse_client.reply_permission(&request_id, false).await {
                                    app.set_status(format!("回复权限失败: {}", e));
                                }
                            }
                            // 权限对话框中的允许
                            KeyCode::Char('y') | KeyCode::Char('Y') if app.permission_request.is_some() => {
                                let request_id = app.allow_permission().unwrap();
                                if let Err(e) = self.sse_client.reply_permission(&request_id, true).await {
                                    app.set_status(format!("回复权限失败: {}", e));
                                }
                            }
                            // 滚动
                            KeyCode::Up => {
                                if app.permission_request.is_none() {
                                    app.scroll_up();
                                }
                            }
                            KeyCode::Down => {
                                if app.permission_request.is_none() {
                                    app.scroll_down();
                                }
                            }
                            // 输入字符
                            KeyCode::Char(c) if app.permission_request.is_none() => {
                                app.input.push(c);
                            }
                            // 删除字符
                            KeyCode::Backspace if app.permission_request.is_none() => {
                                app.input.pop();
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 处理后端事件
            while let Ok(event) = rx.try_recv() {
                match event {
                    AppEvent::SessionCreated(session) => {
                        app.session_id = Some(session.id.clone());
                        app.set_status("已创建新会话");
                    }
                    AppEvent::SessionDeleted(id) => {
                        if app.session_id.as_deref() == Some(&id) {
                            app.session_id = None;
                            app.clear_messages();
                            app.set_status("会话已删除");
                        }
                    }
                    AppEvent::MessageCreated(message) => {
                        app.add_message(message);
                        app.set_loading(false);
                    }
                    AppEvent::PermissionRequest { id, tool, operation } => {
                        app.handle_permission_request(super::tui::app::PermissionRequest {
                            id,
                            tool,
                            operation,
                        });
                    }
                    AppEvent::PermissionReply { id, allow } => {
                        app.set_status(if allow { "权限已允许" } else { "权限已拒绝" });
                    }
                    AppEvent::Error(msg) => {
                        app.set_status(format!("错误: {}", msg));
                        app.set_loading(false);
                    }
                    AppEvent::Connected => {
                        app.set_status("已连接到服务器");
                    }
                    AppEvent::Disconnected => {
                        app.set_status("已断开连接");
                    }
                }
            }

            if app.should_quit {
                break;
            }
        }

        Ok(())
    }
}

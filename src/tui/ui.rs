//! TUI UI 渲染

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::App;

/// 渲染 UI
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 创建主布局：消息列表 + 状态栏 + 输入框
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // 消息列表
            Constraint::Length(1),   // 状态栏
            Constraint::Length(3),   // 输入框
        ])
        .split(area);

    // 渲染消息列表
    render_messages(frame, app, chunks[0]);

    // 渲染状态栏
    render_status_bar(frame, app, chunks[1]);

    // 渲染输入框
    render_input(frame, app, chunks[2]);

    // 渲染权限对话框（如果有）
    if let Some(request) = &app.permission_request {
        render_permission_dialog(frame, request, area);
    }
}

/// 渲染消息列表
fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    let display_messages = app.get_display_messages();

    let items: Vec<ListItem> = display_messages
        .iter()
        .map(|msg| {
            let style = if msg.is_user {
                Style::default().fg(Color::Cyan)
            } else if msg.is_assistant {
                Style::default().fg(Color::Green)
            } else if msg.is_tool {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content_lines: Vec<Line> = msg
                .content
                .lines()
                .map(|line| Line::from(Span::styled(line, style)))
                .collect();

            let header = Line::from(vec![
                Span::styled(
                    format!("[{}] ", msg.role),
                    style.add_modifier(Modifier::BOLD),
                ),
            ]);

            let mut lines = vec![header];
            lines.extend(content_lines);
            lines.push(Line::from("")); // 空行分隔

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 消息 "),
        );

    frame.render_widget(list, area);
}

/// 渲染状态栏
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_style = if app.loading {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let session_info = if let Some(id) = &app.session_id {
        format!("会话: {}...{}", &id[..8], &id[id.len()-4..])
    } else {
        "未连接".to_string()
    };

    let status_line = Line::from(vec![
        Span::styled(format!("[{}] ", session_info), Style::default().fg(Color::Blue)),
        Span::styled(&app.status, status_style),
        Span::raw(" | "),
        Span::styled("q:退出 Enter:发送 ↑↓:滚动", Style::default().fg(Color::DarkGray)),
    ]);

    let status_bar = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(status_bar, area);
}

/// 渲染输入框
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_style = if app.loading {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(app.input.as_str())
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 输入 (Enter 发送) "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);
}

/// 渲染权限对话框
fn render_permission_dialog(frame: &mut Frame, request: &super::app::PermissionRequest, area: Rect) {
    // 计算对话框位置和大小
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 10.min(area.height.saturating_sub(4));
    let dialog_area = Rect::new(
        (area.width.saturating_sub(dialog_width)) / 2,
        (area.height.saturating_sub(dialog_height)) / 2,
        dialog_width,
        dialog_height,
    );

    // 清除背景
    frame.render_widget(Clear, dialog_area);

    // 创建对话框内容
    let content = vec![
        Line::from(Span::styled(
            "⚠️  权限请求",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("工具: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&request.tool),
        ]),
        Line::from(vec![
            Span::styled("操作: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&request.operation),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "y:允许  n:拒绝",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let dialog = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow))
                .title(" 权限确认 "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(dialog, dialog_area);
}

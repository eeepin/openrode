use anyhow::Result;
use futures::StreamExt;
use hillm::client::ChatCompletionClient;
use hillm::types::{
    ChatCompletionRequest, FinishReason, FunctionCall, Message as HillmMessage, StreamFunctionCall,
    StreamToolCall, Tool, ToolCall, ToolType,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::permission::PermissionManager;
use crate::plugin::PluginRegistry;
use crate::mcp::McpRegistry;
use crate::prompt;
use crate::session::{Message, Part, Role, Session};
use crate::skill::SkillRegistry;
use crate::snapshot::Snapshot;
use crate::storage::Storage;
use crate::tool::{self, ToolRegistry};

/// Doom loop 检测阈值：相同工具调用连续出现次数
const DOOM_LOOP_THRESHOLD: usize = 3;

/// 默认最大步数
const DEFAULT_MAX_STEPS: usize = 100;

/// 代理循环状态
pub struct AgentLoop {
    client: Arc<dyn ChatCompletionClient>,
    session: Session,
    storage: Arc<dyn Storage>,
    tool_defs: Vec<Tool>,
    registry: ToolRegistry,
    permissions: PermissionManager,
    #[allow(dead_code)]
    cwd: PathBuf,
    snapshot: Option<Snapshot>,
    step_count: usize,
    max_steps: usize,
}

impl AgentLoop {
    /// 创建新的代理循环（新会话）
    pub async fn new(
        client: Box<dyn ChatCompletionClient>,
        model: String,
        storage: Box<dyn Storage>,
        cwd: PathBuf,
        skill_registry: Option<SkillRegistry>,
        plugin_registry: Option<PluginRegistry>,
        mcp_registry: Option<McpRegistry>,
    ) -> Result<Self> {
        // 获取技能列表（在传递给 tool registry 之前）
        let skill_list = if let Some(ref registry) = skill_registry {
            Some(crate::skill::skill_list(registry).await)
        } else {
            None
        };

        // 将 storage 和 client 转换为 Arc 以便在工具中使用
        let storage_arc: Arc<dyn Storage> = Arc::from(storage);
        let client_arc: Arc<dyn ChatCompletionClient> = Arc::from(client);

        let registry = tool::create_registry(
            skill_registry,
            Some(storage_arc.clone()),
            Some(client_arc.clone()),
            plugin_registry,
            mcp_registry,
        ).await;
        let tool_defs = tool::get_tools(&registry).await;
        let session = Session::new(model.clone());

        // 初始化权限管理器
        let mut permissions = PermissionManager::new();
        permissions.load_project_config(&cwd).await?;

        // 初始化快照管理器
        let snapshot = Snapshot::new(&session.id, &cwd).ok();

        // 持久化新会话
        storage_arc.create_session(&session).await?;

        // 构建系统提示并添加系统消息
        let system_prompt = prompt::build_system_prompt(&model, &cwd, skill_list.as_deref());
        let system_msg = Message::system_text(session.id.clone(), system_prompt);
        storage_arc.append_message(&session.id, &system_msg).await?;

        Ok(Self {
            client: client_arc,
            session,
            storage: storage_arc,
            tool_defs,
            registry,
            permissions,
            cwd,
            snapshot,
            step_count: 0,
            max_steps: DEFAULT_MAX_STEPS,
        })
    }

    /// 从已有会话恢复代理循环
    pub async fn resume(
        client: Box<dyn ChatCompletionClient>,
        session_id: &str,
        storage: Box<dyn Storage>,
        cwd: PathBuf,
        skill_registry: Option<SkillRegistry>,
        plugin_registry: Option<PluginRegistry>,
        mcp_registry: Option<McpRegistry>,
    ) -> Result<Self> {
        // 将 storage 和 client 转换为 Arc 以便在工具中使用
        let storage_arc: Arc<dyn Storage> = Arc::from(storage);
        let client_arc: Arc<dyn ChatCompletionClient> = Arc::from(client);

        let registry = tool::create_registry(
            skill_registry,
            Some(storage_arc.clone()),
            Some(client_arc.clone()),
            plugin_registry,
            mcp_registry,
        ).await;
        let tool_defs = tool::get_tools(&registry).await;

        // 初始化权限管理器
        let mut permissions = PermissionManager::new();
        permissions.load_project_config(&cwd).await?;

        let session_data = storage_arc
            .get_session(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("会话不存在: {session_id}"))?;

        // 初始化快照管理器
        let snapshot = Snapshot::new(session_id, &cwd).ok();

        Ok(Self {
            client: client_arc,
            session: session_data.session,
            storage: storage_arc,
            tool_defs,
            registry,
            permissions,
            cwd,
            snapshot,
            step_count: 0,
            max_steps: DEFAULT_MAX_STEPS,
        })
    }

    /// 获取当前会话 ID
    pub fn session_id(&self) -> &str {
        &self.session.id
    }

    /// 获取历史消息（用于构建 LLM 请求）
    async fn load_messages(&self) -> Result<Vec<HillmMessage>> {
        let session_data = self
            .storage
            .get_session(&self.session.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("会话不存在"))?;

        Ok(session_data.messages.iter().map(|m| m.to_hillm()).collect())
    }

    /// 运行代理循环，直到 LLM 不再调用工具
    pub async fn run(&mut self, user_prompt: &str) -> Result<()> {
        // 添加并持久化用户消息
        let user_msg = Message::user_text(self.session.id.clone(), user_prompt.to_string());
        self.storage
            .append_message(&self.session.id, &user_msg)
            .await?;

        loop {
            self.step_count += 1;

            // 检查步数上限
            if self.step_count > self.max_steps {
                let limit_msg = Message::new(
                    self.session.id.clone(),
                    Role::System,
                    vec![Part::Text {
                        content: format!(
                            "已达到步数上限（{}步），请总结当前进度并结束任务。",
                            self.max_steps
                        ),
                    }],
                );
                self.storage
                    .append_message(&self.session.id, &limit_msg)
                    .await?;
            }

            // Doom loop 检测
            if let Some(warning) = self.detect_doom_loop().await? {
                let warning_msg = Message::new(
                    self.session.id.clone(),
                    Role::System,
                    vec![Part::Text { content: warning }],
                );
                self.storage
                    .append_message(&self.session.id, &warning_msg)
                    .await?;
            }

            // 从存储加载所有消息
            let messages = self.load_messages().await?;

            let request = ChatCompletionRequest {
                model: self.session.model.clone(),
                messages,
                tools: Some(self.tool_defs.clone()),
                stream: Some(true),
                ..Default::default()
            };

            let mut stream = self.client.chat_stream(request).await?;

            // 流式处理：累积文本和工具调用
            let mut text_content = String::new();
            let mut tool_calls_map: HashMap<u32, StreamToolCall> = HashMap::new();
            let mut finish_reason = None;

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;

                for choice in chunk.choices {
                    // 累积文本
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                        text_content.push_str(content);
                    }

                    // 累积工具调用
                    if let Some(stream_tool_calls) = &choice.delta.tool_calls {
                        for tc in stream_tool_calls {
                            let entry =
                                tool_calls_map
                                    .entry(tc.index)
                                    .or_insert_with(|| StreamToolCall {
                                        index: tc.index,
                                        id: None,
                                        call_type: None,
                                        function: None,
                                    });

                            if let Some(id) = &tc.id {
                                entry.id = Some(id.clone());
                            }
                            if let Some(ct) = &tc.call_type {
                                entry.call_type = Some(ct.clone());
                            }
                            if let Some(func) = &tc.function {
                                let entry_func = entry.function.get_or_insert(StreamFunctionCall {
                                    name: None,
                                    arguments: None,
                                });
                                if let Some(name) = &func.name {
                                    entry_func.name = Some(name.clone());
                                }
                                if let Some(args) = &func.arguments {
                                    let existing =
                                        entry_func.arguments.get_or_insert_with(String::new);
                                    existing.push_str(args);
                                }
                            }
                        }
                    }

                    // 记录 finish reason
                    if let Some(reason) = &choice.finish_reason {
                        finish_reason = Some(reason.clone());
                    }
                }
            }
            println!(); // 换行

            // 构建 tool calls 列表
            let mut completed_tool_calls: Vec<ToolCall> = tool_calls_map
                .values()
                .filter_map(|stc| {
                    let id = stc.id.as_ref()?;
                    let func = stc.function.as_ref()?;
                    let name = func.name.as_ref()?;
                    let args = func.arguments.clone().unwrap_or_default();
                    Some(ToolCall {
                        id: id.clone(),
                        call_type: ToolType::Function,
                        function: FunctionCall {
                            name: name.clone(),
                            arguments: args,
                        },
                    })
                })
                .collect();
            completed_tool_calls.sort_by(|a, b| a.id.cmp(&b.id));

            // 持久化 assistant 消息
            let assistant_text = if text_content.is_empty() {
                None
            } else {
                Some(text_content.clone())
            };
            let assistant_msg = Message::assistant_tool_calls(
                self.session.id.clone(),
                assistant_text,
                &completed_tool_calls,
            );
            self.storage
                .append_message(&self.session.id, &assistant_msg)
                .await?;

            // 检查是否结束
            let should_stop = match &finish_reason {
                Some(FinishReason::ToolCalls) => tool_calls_map.is_empty(),
                _ => true,
            };

            if should_stop {
                break;
            }

            // 执行工具并将结果持久化
            for stc in tool_calls_map.values() {
                let Some(id) = &stc.id else { continue };
                let Some(func) = &stc.function else { continue };
                let Some(name) = &func.name else { continue };
                let args = func.arguments.clone().unwrap_or_default();

                // 工具执行前打快照
                if let Some(snapshot) = &self.snapshot {
                    let _ = snapshot.before_tool_execution(&format!("{} {}", name, args));
                }

                println!("[Tool] {name}");
                let result =
                    tool::execute_tool(&self.registry, &self.permissions, name, &args).await;

                if result.error {
                    println!("[Tool Error] {}", result.output);
                }

                let tool_msg = Message::tool_result(
                    self.session.id.clone(),
                    id.clone(),
                    name.clone(),
                    result.output,
                    result.error,
                );
                self.storage
                    .append_message(&self.session.id, &tool_msg)
                    .await?;
            }
        }

        Ok(())
    }

    /// 检测 doom loop（重复工具调用）
    async fn detect_doom_loop(&self) -> Result<Option<String>> {
        let session_data = self
            .storage
            .get_session(&self.session.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("会话不存在"))?;

        let messages = &session_data.messages;

        // 取最近的消息进行分析
        let recent: Vec<_> = messages
            .iter()
            .rev()
            .take(DOOM_LOOP_THRESHOLD * 4)
            .collect();

        // 提取最近的工具调用
        let tool_calls: Vec<(&str, &str)> = recent
            .iter()
            .filter(|m| m.role == Role::Assistant)
            .flat_map(|m| {
                m.parts.iter().filter_map(|p| match p {
                    Part::ToolUse {
                        name, arguments, ..
                    } => Some((name.as_str(), arguments.as_str())),
                    _ => None,
                })
            })
            .collect();

        // 检查是否有连续的相同调用
        if tool_calls.len() >= DOOM_LOOP_THRESHOLD {
            let last_n: Vec<_> = tool_calls.iter().take(DOOM_LOOP_THRESHOLD).collect();
            let first = last_n[0];
            let all_same = last_n
                .iter()
                .all(|(name, args)| *name == first.0 && *args == first.1);

            if all_same {
                return Ok(Some(format!(
                    "检测到重复工具调用：{} 连续执行了 {} 次相同参数的 {}。\
                    \n请尝试不同的方法，或说明卡住的原因。",
                    first.0, DOOM_LOOP_THRESHOLD, first.0
                )));
            }
        }

        Ok(None)
    }

    /// 获取当前步数
    #[allow(dead_code)]
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// 设置最大步数
    #[allow(dead_code)]
    pub fn set_max_steps(&mut self, max_steps: usize) {
        self.max_steps = max_steps;
    }
}

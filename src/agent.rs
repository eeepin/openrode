use anyhow::Result;
use futures::StreamExt;
use hillm::client::ChatCompletionClient;
use hillm::types::{
    ChatCompletionRequest, FinishReason, FunctionCall, Message as HillmMessage, StreamFunctionCall,
    StreamToolCall, Tool, ToolCall, ToolType,
};
use std::collections::HashMap;

use crate::session::{Message, Session};
use crate::storage::Storage;
use crate::tool::{self, ToolRegistry};

const SYSTEM_PROMPT: &str = "你是一个编程助手，可用工具操作文件和执行命令。\
    先调查再动手，回答简洁。";

/// 代理循环状态
pub struct AgentLoop {
    client: Box<dyn ChatCompletionClient>,
    session: Session,
    storage: Box<dyn Storage>,
    tool_defs: Vec<Tool>,
    registry: ToolRegistry,
}

impl AgentLoop {
    /// 创建新的代理循环（新会话）
    pub async fn new(
        client: Box<dyn ChatCompletionClient>,
        model: String,
        storage: Box<dyn Storage>,
    ) -> Result<Self> {
        let registry = tool::create_registry().await;
        let tool_defs = tool::get_tools(&registry).await;
        let session = Session::new(model);

        // 持久化新会话
        storage.create_session(&session).await?;

        // 添加系统消息
        let system_msg = Message::system_text(session.id.clone(), SYSTEM_PROMPT.to_string());
        storage.append_message(&session.id, &system_msg).await?;

        Ok(Self {
            client,
            session,
            storage,
            tool_defs,
            registry,
        })
    }

    /// 从已有会话恢复代理循环
    pub async fn resume(
        client: Box<dyn ChatCompletionClient>,
        session_id: &str,
        storage: Box<dyn Storage>,
    ) -> Result<Self> {
        let registry = tool::create_registry().await;
        let tool_defs = tool::get_tools(&registry).await;

        let session_data = storage
            .get_session(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("会话不存在: {session_id}"))?;

        Ok(Self {
            client,
            session: session_data.session,
            storage,
            tool_defs,
            registry,
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

                println!("[Tool] {name}");
                let result = tool::execute_tool(&self.registry, name, &args).await;

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
}

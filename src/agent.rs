use anyhow::Result;
use futures::StreamExt;
use hillm::client::ChatCompletionClient;
use hillm::types::{
    AssistantMessage, ChatCompletionRequest, FinishReason, FunctionCall, Message, MessageContent,
    StreamFunctionCall, StreamToolCall, SystemMessage, Tool, ToolCall, ToolMessage, ToolType,
};
use std::collections::HashMap;

use crate::tool::{self, ToolRegistry};

const SYSTEM_PROMPT: &str = "你是一个编程助手，可用工具操作文件和执行命令。\
    先调查再动手，回答简洁。";

/// 代理循环状态
pub struct AgentLoop {
    client: Box<dyn ChatCompletionClient>,
    model: String,
    messages: Vec<Message>,
    tool_defs: Vec<Tool>,
    registry: ToolRegistry,
}

impl AgentLoop {
    /// 创建代理循环（异步，因为需要初始化工具注册表）
    pub async fn new(client: Box<dyn ChatCompletionClient>, model: String) -> Self {
        let registry = tool::create_registry().await;
        let tool_defs = tool::get_tools(&registry).await;
        let system_msg = Message::System(SystemMessage {
            content: MessageContent::Text(SYSTEM_PROMPT.to_string()),
            name: None,
        });
        Self {
            client,
            model,
            messages: vec![system_msg],
            tool_defs,
            registry,
        }
    }

    /// 运行代理循环，直到 LLM 不再调用工具
    pub async fn run(&mut self, user_prompt: &str) -> Result<()> {
        // 添加用户消息
        self.messages.push(Message::User(hillm::UserMessage {
            content: MessageContent::Text(user_prompt.to_string()),
            name: None,
        }));

        loop {
            let request = ChatCompletionRequest {
                model: self.model.clone(),
                messages: self.messages.clone(),
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

            // 构建 assistant 消息
            let assistant_msg = if tool_calls_map.is_empty() {
                // 纯文本回复
                Message::Assistant(AssistantMessage {
                    content: if text_content.is_empty() {
                        None
                    } else {
                        Some(MessageContent::Text(text_content.clone()))
                    },
                    tool_calls: None,
                    ..Default::default()
                })
            } else {
                // 有工具调用
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

                Message::Assistant(AssistantMessage {
                    content: if text_content.is_empty() {
                        None
                    } else {
                        Some(MessageContent::Text(text_content.clone()))
                    },
                    tool_calls: Some(completed_tool_calls),
                    ..Default::default()
                })
            };

            self.messages.push(assistant_msg);

            // 检查是否结束
            let should_stop = match &finish_reason {
                Some(FinishReason::ToolCalls) => tool_calls_map.is_empty(),
                _ => true,
            };

            if should_stop {
                break;
            }

            // 执行工具并将结果添加到消息列表
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

                self.messages.push(Message::Tool(ToolMessage {
                    content: MessageContent::Text(result.output),
                    tool_call_id: id.clone(),
                    name: Some(name.clone()),
                }));
            }
        }

        Ok(())
    }
}

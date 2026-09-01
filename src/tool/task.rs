//! 子代理工具
//!
//! 允许主代理创建子任务，派给专门的 agent 执行。

use async_trait::async_trait;
use hillm::client::ChatCompletionClient;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use super::{AgentTool, ToolResult};
use crate::session::{Message, Role};
use crate::storage::Storage;

/// 子代理定义
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub max_steps: usize,
}

impl AgentDefinition {
    /// 获取预定义的 agent
    pub fn get(name: &str) -> Option<Self> {
        match name {
            "general" => Some(Self {
                name: "general".to_string(),
                description: "通用代理，可以执行各种任务".to_string(),
                system_prompt: "你是一个通用编程助手。请帮助用户完成任务。".to_string(),
                max_steps: 50,
            }),
            "explore" => Some(Self {
                name: "explore".to_string(),
                description: "代码探索代理，专注于阅读和理解代码".to_string(),
                system_prompt: "你是一个代码探索助手。你的任务是帮助用户理解代码结构和逻辑。\
                    请使用 read 工具读取文件，分析代码，并提供清晰的解释。\
                    不要修改任何文件，只进行只读操作。"
                    .to_string(),
                max_steps: 30,
            }),
            "search" => Some(Self {
                name: "search".to_string(),
                description: "搜索代理，专注于在代码库中查找信息".to_string(),
                system_prompt: "你是一个搜索助手。你的任务是在代码库中查找特定信息。\
                    使用 bash 工具执行 grep、find 等命令来搜索代码。\
                    提供准确的搜索结果和文件位置。"
                    .to_string(),
                max_steps: 20,
            }),
            _ => None,
        }
    }

    /// 列出所有可用的 agent
    pub fn list_all() -> Vec<Self> {
        vec![
            Self::get("general").unwrap(),
            Self::get("explore").unwrap(),
            Self::get("search").unwrap(),
        ]
    }
}

/// 子代理工具
pub struct TaskTool {
    storage: Arc<dyn Storage>,
    client: Arc<dyn ChatCompletionClient>,
}

impl TaskTool {
    pub fn new(storage: Arc<dyn Storage>, client: Arc<dyn ChatCompletionClient>) -> Self {
        Self { storage, client }
    }
}

#[derive(Deserialize)]
struct TaskInput {
    agent: String,
    prompt: String,
}

const DESCRIPTION: &str = include_str!("prompts/task.txt");

#[async_trait]
impl AgentTool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "agent": {
                    "type": "string",
                    "description": "代理类型: general, explore, search",
                    "enum": ["general", "explore", "search"]
                },
                "prompt": {
                    "type": "string",
                    "description": "子任务的具体指令"
                }
            },
            "required": ["agent", "prompt"]
        })
    }

    async fn run(&self, input: Value) -> ToolResult {
        let TaskInput { agent, prompt } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {e}")),
        };

        // 获取 agent 定义
        let agent_def = match AgentDefinition::get(&agent) {
            Some(def) => def,
            None => {
                return ToolResult::error(format!(
                    "未知的代理类型: {}。可用类型: general, explore, search",
                    agent
                ))
            }
        };

        // 创建子会话
        let model = self.storage.get_model().await.ok().flatten().unwrap_or_else(|| "default".to_string());
        let child_session = crate::session::Session::new(model);
        if let Err(e) = self.storage.create_session(&child_session).await {
            return ToolResult::error(format!("创建子会话失败: {e}"));
        }

        // 添加系统消息
        let system_msg = Message::system_text(child_session.id.clone(), agent_def.system_prompt);
        if let Err(e) = self.storage.append_message(&child_session.id, &system_msg).await {
            return ToolResult::error(format!("添加系统消息失败: {e}"));
        }

        // 克隆 client 和 storage 用于子代理
        let client_clone = self.client.clone();
        let storage_clone = self.storage.clone();

        // 创建并运行子代理循环
        // 注意：这里我们简化了实现，直接创建一个临时的 agent loop
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // 由于 AgentLoop 需要 Box<dyn ChatCompletionClient>，我们需要特殊处理
        // 这里我们创建一个简化的版本
        let result = run_subtask(
            client_clone,
            storage_clone,
            &child_session.id,
            &prompt,
            agent_def.max_steps,
            cwd,
        )
        .await;

        match result {
            Ok(response) => ToolResult::success(format!(
                "子代理 ({}) 完成任务:\n\n{}",
                agent_def.name, response
            )),
            Err(e) => ToolResult::error(format!("子代理执行失败: {e}")),
        }
    }
}

/// 运行子任务
async fn run_subtask(
    client: Arc<dyn ChatCompletionClient>,
    storage: Arc<dyn Storage>,
    session_id: &str,
    prompt: &str,
    max_steps: usize,
    _cwd: std::path::PathBuf,
) -> anyhow::Result<String> {
    // 创建一个简化的代理循环
    // 这里我们需要重新实现一个简化版的 agent loop

    use crate::permission::PermissionManager;
    use crate::tool;

    // 创建工具注册表（子代理可以使用所有工具）
    let registry = tool::create_registry(None, Some(storage.clone()), Some(client.clone())).await;
    let tool_defs = tool::get_tools(&registry).await;

    // 添加用户消息
    let user_msg = Message::user_text(session_id.to_string(), prompt.to_string());
    storage.append_message(session_id, &user_msg).await?;

    // 简化的代理循环
    let mut step = 0;
    let mut last_response = String::new();

    loop {
        if step >= max_steps {
            return Err(anyhow::anyhow!("达到最大步数限制: {}", max_steps));
        }

        // 加载消息
        let session_data = storage.get_session(session_id).await?
            .ok_or_else(|| anyhow::anyhow!("会话不存在"))?;

        let messages: Vec<hillm::Message> = session_data.messages.iter().map(|m| m.to_hillm()).collect();

        // 调用 LLM
        let model = storage.get_model().await.ok().flatten().unwrap_or_else(|| "default".to_string());
        let request = hillm::ChatCompletionRequest {
            model,
            messages,
            tools: Some(tool_defs.clone()),
            stream: Some(false),
            ..Default::default()
        };

        let client_ref = client.as_ref();
        let response = client_ref.chat(request).await?;

        if response.choices.is_empty() {
            return Err(anyhow::anyhow!("LLM 未返回任何选择"));
        }

        let choice = &response.choices[0];
        let assistant_msg = choice.message.clone();

        // 保存助手消息
        let mut parts = Vec::new();

        // 添加文本内容
        if let Some(content) = &assistant_msg.content {
            if let Some(text) = content.as_text() {
                last_response = text.clone();
                parts.push(crate::session::Part::Text { content: text });
            }
        }

        // 添加工具调用
        if let Some(tool_calls) = &assistant_msg.tool_calls {
            for tc in tool_calls {
                parts.push(crate::session::Part::ToolUse {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                });
            }
        }

        let msg = Message::new(
            session_id.to_string(),
            Role::Assistant,
            parts,
        );
        storage.append_message(session_id, &msg).await?;

        // 如果没有工具调用，结束循环
        if assistant_msg.tool_calls.is_none() || assistant_msg.tool_calls.as_ref().unwrap().is_empty() {
            break;
        }

        // 执行工具调用
        let permissions = PermissionManager::new();
        for tc in assistant_msg.tool_calls.as_ref().unwrap() {
            let result = tool::execute_tool(
                &registry,
                &permissions,
                &tc.function.name,
                &tc.function.arguments,
            ).await;

            let tool_msg = Message::tool_result(
                session_id.to_string(),
                tc.id.clone(),
                tc.function.name.clone(),
                result.output,
                result.error,
            );
            storage.append_message(session_id, &tool_msg).await?;
        }

        step += 1;
    }

    Ok(last_response)
}

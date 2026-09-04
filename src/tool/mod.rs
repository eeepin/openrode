use async_trait::async_trait;
use hillm::client::ChatCompletionClient;
use hillm::types::{FunctionDefinition, Tool, ToolType};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::permission::{PermissionDecision, PermissionManager, PermissionRequest};
use crate::plugin::PluginRegistry;
use crate::skill::{SkillRegistry, SkillTool};
use crate::storage::Storage;

pub mod bash;
pub mod read;
pub mod task;
pub mod write;

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub error: bool,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            error: false,
        }
    }

    pub fn error(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            error: true,
        }
    }
}

/// 工具 trait - 所有工具必须实现
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述（用于系统提示）
    fn description(&self) -> &str;

    /// 工具的 JSON Schema 参数定义
    fn input_schema(&self) -> Value;

    /// 执行工具
    async fn run(&self, input: Value) -> ToolResult;

    /// 生成权限请求（用于权限检查）
    fn permission_request(&self, input: &Value) -> PermissionRequest {
        // 默认实现：使用工具名和输入的简单表示
        PermissionRequest::new(
            self.name(),
            serde_json::to_string(input).unwrap_or_default(),
        )
    }

    /// 转换为 LLM API 所需的 Tool 定义
    fn to_tool(&self) -> Tool {
        Tool {
            tool_type: ToolType::Function,
            function: FunctionDefinition {
                name: self.name().to_string(),
                description: Some(self.description().to_string()),
                parameters: Some(self.input_schema()),
                strict: None,
            },
        }
    }
}

/// 工具注册表
pub type ToolRegistry = Arc<RwLock<HashMap<String, Box<dyn AgentTool>>>>;

/// 创建并初始化工具注册表
pub async fn create_registry(
    skill_registry: Option<SkillRegistry>,
    storage: Option<Arc<dyn Storage>>,
    client: Option<Arc<dyn ChatCompletionClient>>,
    plugin_registry: Option<PluginRegistry>,
    mcp_registry: Option<crate::mcp::McpRegistry>,
) -> ToolRegistry {
    let registry: ToolRegistry = Arc::new(RwLock::new(HashMap::new()));

    // 注册所有工具
    {
        let mut reg = registry.write().await;
        reg.insert("bash".to_string(), Box::new(bash::BashTool));
        reg.insert("read".to_string(), Box::new(read::ReadTool));
        reg.insert("write".to_string(), Box::new(write::WriteTool));

        // 注册技能工具（如果有技能注册表）
        if let Some(skill_reg) = skill_registry {
            reg.insert("skill".to_string(), Box::new(SkillTool::new(skill_reg)));
        }

        // 注册子代理工具（如果有 storage 和 client）
        if let (Some(storage), Some(client)) = (storage, client) {
            reg.insert(
                "task".to_string(),
                Box::new(task::TaskTool::new(storage, client)),
            );
        }

        // 注册插件工具（如果有插件注册表）
        if let Some(plugin_reg) = plugin_registry {
            let plugin_tools = crate::plugin::extract_tools(&plugin_reg).await;
            for tool in plugin_tools {
                let tool_name = tool.name().to_string();
                println!("注册插件工具: {}", tool_name);
                reg.insert(tool_name, tool);
            }
        }

        // 注册 MCP 工具（如果有 MCP 注册表）
        if let Some(mcp_reg) = mcp_registry {
            match crate::mcp::load_tools(&mcp_reg).await {
                Ok(mcp_tools) => {
                    for tool in mcp_tools {
                        let tool_name = tool.name().to_string();
                        println!("注册 MCP 工具: {}", tool_name);
                        reg.insert(tool_name, tool);
                    }
                }
                Err(e) => {
                    eprintln!("加载 MCP 工具失败: {}", e);
                }
            }
        }
    }

    registry
}

/// 执行工具调用（带权限检查）
pub async fn execute_tool(
    registry: &ToolRegistry,
    permissions: &PermissionManager,
    name: &str,
    arguments: &str,
) -> ToolResult {
    let reg = registry.read().await;

    let Some(tool) = reg.get(name) else {
        let available: Vec<_> = reg.keys().cloned().collect();
        return ToolResult::error(format!(
            "未知工具: {name}，可用工具: {}",
            available.join(", ")
        ));
    };

    // 解析参数
    let input: Value = match serde_json::from_str(arguments) {
        Ok(v) => v,
        Err(e) => {
            return ToolResult::error(format!("参数解析失败: {e}\n请检查 JSON 格式"));
        }
    };

    // 生成权限请求
    let request = tool.permission_request(&input);

    // 检查权限
    match permissions.check(&request).await {
        Ok(PermissionDecision::Allowed) | Ok(PermissionDecision::Asked) => {
            // 允许执行
        }
        Ok(PermissionDecision::Denied(reason)) => {
            return ToolResult::error(format!("权限被拒绝: {reason}"));
        }
        Err(e) => {
            return ToolResult::error(format!("权限检查失败: {e}"));
        }
    }

    // 执行工具并截断输出
    let result = tool.run(input).await;
    ToolResult {
        output: truncate_output(&result.output, 50_000),
        error: result.error,
    }
}

/// 获取所有工具的 API 定义
pub async fn get_tools(registry: &ToolRegistry) -> Vec<Tool> {
    let reg = registry.read().await;
    reg.values().map(|t| t.to_tool()).collect()
}

/// 截断过长的工具输出（保头保尾）
pub fn truncate_output(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let half = max_len / 2;
        let mut result = s[..half].to_string();
        result.push_str(&format!(
            "\n\n... 已截断 {} 字节 ...\n\n",
            s.len() - max_len
        ));
        result.push_str(&s[s.len() - half..]);
        result
    }
}

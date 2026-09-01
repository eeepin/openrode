//! MCP (Model Context Protocol) 支持
//!
//! 实现 MCP 客户端，可以连接到 MCP 服务器并使用其提供的工具。
//! MCP 是一个标准协议，允许 AI 模型访问外部工具和数据源。

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tool::{AgentTool, ToolResult};

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP 客户端
pub struct McpClient {
    config: McpServerConfig,
    // 未来可以添加实际的 MCP 连接
    // 目前只实现基本框架
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        Self { config }
    }

    /// 列出服务器提供的工具
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        // 目前返回空列表，未来需要实现实际的 MCP 协议
        // 这需要使用 rmcp 或类似的 MCP 客户端库
        Ok(Vec::new())
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, _input: Value) -> Result<Value> {
        // 目前返回错误，未来需要实现实际的 MCP 协议
        Err(anyhow::anyhow!("MCP 工具调用未实现: {}", name))
    }
}

/// MCP 工具包装器
pub struct McpTool {
    client: Arc<McpClient>,
    definition: McpToolDefinition,
}

impl McpTool {
    pub fn new(client: Arc<McpClient>, definition: McpToolDefinition) -> Self {
        Self { client, definition }
    }
}

#[async_trait]
impl AgentTool for McpTool {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        &self.definition.description
    }

    fn input_schema(&self) -> Value {
        self.definition.input_schema.clone()
    }

    async fn run(&self, input: Value) -> ToolResult {
        match self.client.call_tool(&self.definition.name, input).await {
            Ok(result) => ToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())),
            Err(e) => ToolResult::error(format!("MCP 工具调用失败: {}", e)),
        }
    }
}

/// MCP 注册表
pub type McpRegistry = Arc<RwLock<Vec<Arc<McpClient>>>>;

/// 创建 MCP 注册表
pub async fn create_registry(configs: &[McpServerConfig]) -> Result<McpRegistry> {
    let mut clients = Vec::new();

    for config in configs {
        let client = Arc::new(McpClient::new(config.clone()));
        clients.push(client);
    }

    Ok(Arc::new(RwLock::new(clients)))
}

/// 从所有 MCP 服务器加载工具
pub async fn load_tools(registry: &McpRegistry) -> Result<Vec<Box<dyn AgentTool>>> {
    let mut tools = Vec::new();
    let clients = registry.read().await;

    for client in clients.iter() {
        match client.list_tools().await {
            Ok(definitions) => {
                for def in definitions {
                    tools.push(Box::new(McpTool::new(client.clone(), def)) as Box<dyn AgentTool>);
                }
            }
            Err(e) => {
                eprintln!("从 MCP 服务器 {} 加载工具失败: {}", client.config.name, e);
            }
        }
    }

    Ok(tools)
}

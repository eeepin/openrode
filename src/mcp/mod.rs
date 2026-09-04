//! MCP (Model Context Protocol) 支持
//!
//! 实现 MCP 客户端，可以连接到 MCP 服务器并使用其提供的工具。
//! MCP 是一个标准协议，允许 AI 模型访问外部工具和数据源。

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex, RwLock};

use crate::tool::{AgentTool, ToolResult};

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC 响应
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC 错误
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// MCP 客户端
pub struct McpClient {
    config: McpServerConfig,
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    request_id: Arc<Mutex<u64>>,
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
        }
    }

    /// 启动 MCP 服务器进程
    pub async fn start(&self) -> Result<()> {
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(env) = &self.config.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        let mut child = cmd.spawn().context("启动 MCP 服务器失败")?;

        let stdin = child.stdin.take().context("无法获取 stdin")?;
        let stdout = child.stdout.take().context("无法获取 stdout")?;

        *self.process.lock().await = Some(child);
        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));

        // 发送初始化请求
        self.send_request("initialize", Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "openrode",
                "version": "0.1.0"
            }
        })))
        .await?;

        Ok(())
    }

    /// 发送 JSON-RPC 请求
    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let mut request_id = self.request_id.lock().await;
        *request_id += 1;
        let id = *request_id;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;

        // 写入请求
        {
            let mut stdin = self.stdin.lock().await;
            if let Some(ref mut stdin) = *stdin {
                stdin.write_all(request_json.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;
            } else {
                anyhow::bail!("MCP 服务器未启动");
            }
        }

        // 读取响应
        {
            let mut stdout = self.stdout.lock().await;
            if let Some(ref mut stdout) = *stdout {
                let mut response_line = String::new();
                stdout.read_line(&mut response_line).await?;

                let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

                if response.id != id {
                    anyhow::bail!("响应 ID 不匹配");
                }

                if let Some(error) = response.error {
                    anyhow::bail!("MCP 错误: {} - {}", error.code, error.message);
                }

                Ok(response.result.unwrap_or(Value::Null))
            } else {
                anyhow::bail!("MCP 服务器未启动");
            }
        }
    }

    /// 列出服务器提供的工具
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        let result = self.send_request("tools/list", None).await?;

        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        let tool_defs: Vec<McpToolDefinition> = tools
            .into_iter()
            .filter_map(|t| {
                serde_json::from_value(t).ok()
            })
            .collect();

        Ok(tool_defs)
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, input: Value) -> Result<Value> {
        let result = self
            .send_request(
                "tools/call",
                Some(serde_json::json!({
                    "name": name,
                    "arguments": input
                })),
            )
            .await?;

        Ok(result)
    }

    /// 停止 MCP 服务器
    pub async fn stop(&self) -> Result<()> {
        if let Some(mut child) = self.process.lock().await.take() {
            let _ = child.kill().await;
        }
        *self.stdin.lock().await = None;
        *self.stdout.lock().await = None;
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // 在 drop 时无法执行异步操作，但进程会在父进程退出时自动终止
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
            Ok(result) => {
                // MCP 工具返回的结果通常包含 content 数组
                if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                    let text = content
                        .iter()
                        .filter_map(|item| {
                            item.get("text").and_then(|t| t.as_str())
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    ToolResult::success(if text.is_empty() {
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    } else {
                        text
                    })
                } else {
                    ToolResult::success(serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()))
                }
            }
            Err(e) => ToolResult::error(format!("MCP 工具调用失败: {}", e)),
        }
    }
}

/// MCP 注册表
pub type McpRegistry = Arc<RwLock<Vec<Arc<McpClient>>>>;

/// 创建 MCP 注册表并启动所有服务器
pub async fn create_registry(configs: &[McpServerConfig]) -> Result<McpRegistry> {
    let mut clients = Vec::new();

    for config in configs {
        let client = Arc::new(McpClient::new(config.clone()));

        // 启动 MCP 服务器
        match client.start().await {
            Ok(()) => {
                println!("已启动 MCP 服务器: {}", config.name);
                clients.push(client);
            }
            Err(e) => {
                eprintln!("启动 MCP 服务器 {} 失败: {}", config.name, e);
            }
        }
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
                    println!("加载 MCP 工具: {}", def.name);
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

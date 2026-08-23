use async_trait::async_trait;
use hillm::types::{FunctionDefinition, Tool, ToolType};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod bash;
pub mod read;
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
pub async fn create_registry() -> ToolRegistry {
    let registry: ToolRegistry = Arc::new(RwLock::new(HashMap::new()));

    // 注册所有工具
    {
        let mut reg = registry.write().await;
        reg.insert("bash".to_string(), Box::new(bash::BashTool));
        reg.insert("read".to_string(), Box::new(read::ReadTool));
        reg.insert("write".to_string(), Box::new(write::WriteTool));
    }

    registry
}

/// 执行工具调用
pub async fn execute_tool(registry: &ToolRegistry, name: &str, arguments: &str) -> ToolResult {
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

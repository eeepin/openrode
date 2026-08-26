use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use super::{AgentTool, ToolResult};
use crate::permission::PermissionRequest;

pub struct WriteTool;

#[derive(Deserialize)]
struct WriteInput {
    path: String,
    content: String,
}

const DESCRIPTION: &str = include_str!("prompts/write.txt");

#[async_trait]
impl AgentTool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要写入的文件路径"
                },
                "content": {
                    "type": "string",
                    "description": "要写入的文件内容"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn permission_request(&self, input: &Value) -> PermissionRequest {
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("");
        PermissionRequest::new("write", path).with_detail("path", path)
    }

    async fn run(&self, input: Value) -> ToolResult {
        let WriteInput { path, content } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {e}")),
        };

        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(&path).parent()
            && !parent.as_os_str().is_empty()
            && let Err(e) = tokio::fs::create_dir_all(parent).await
        {
            return ToolResult::error(format!("创建目录失败: {e}"));
        }

        // 写入文件
        match tokio::fs::write(&path, &content).await {
            Ok(_) => {
                let lines = content.lines().count();
                let bytes = content.len();
                ToolResult::success(format!("已写入 {path}（{lines} 行，{bytes} 字节）"))
            }
            Err(e) => ToolResult::error(format!("写入失败: {e}")),
        }
    }
}

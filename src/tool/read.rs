use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use super::{AgentTool, ToolResult};
use crate::permission::PermissionRequest;

pub struct ReadTool;

#[derive(Deserialize)]
struct ReadInput {
    path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

const DESCRIPTION: &str = include_str!("prompts/read.txt");

#[async_trait]
impl AgentTool for ReadTool {
    fn name(&self) -> &str {
        "read"
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
                    "description": "要读取的文件路径"
                },
                "offset": {
                    "type": "integer",
                    "description": "起始行号（从 0 开始，可选）"
                },
                "limit": {
                    "type": "integer",
                    "description": "读取行数（可选，默认全部）"
                }
            },
            "required": ["path"]
        })
    }

    fn permission_request(&self, input: &Value) -> PermissionRequest {
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("");
        PermissionRequest::new("read", path)
    }

    async fn run(&self, input: Value) -> ToolResult {
        let ReadInput {
            path,
            offset,
            limit,
        } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {e}")),
        };

        // 读取文件
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => return ToolResult::error(format!("读取失败: {e}")),
        };

        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        // 应用 offset 和 limit
        let start = offset.unwrap_or(0);
        let end = limit.map(|l| (start + l).min(total)).unwrap_or(total);

        if start >= total {
            return ToolResult::success(format!("(文件共 {total} 行，offset={start} 超出范围)"));
        }

        let selected = &lines[start..end];

        // 带行号输出
        let mut output = String::new();
        for (i, line) in selected.iter().enumerate() {
            let line_num = start + i + 1; // 1-indexed
            output.push_str(&format!("{line_num:>5}\t{line}\n"));
        }

        if end < total {
            output.push_str(&format!(
                "\n... 还有 {} 行未显示 (使用 limit 参数继续) ...",
                total - end
            ));
        }

        ToolResult::success(output)
    }
}

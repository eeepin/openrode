use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

use super::{AgentTool, ToolResult};
use crate::permission::PermissionRequest;

pub struct BashTool;

#[derive(Deserialize)]
struct BashInput {
    command: String,
}

const DESCRIPTION: &str = include_str!("prompts/bash.txt");

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的 shell 命令"
                }
            },
            "required": ["command"]
        })
    }

    fn permission_request(&self, input: &Value) -> PermissionRequest {
        let command = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
        PermissionRequest::new("bash", command)
    }

    async fn run(&self, input: Value) -> ToolResult {
        let BashInput { command } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {e}")),
        };

        let result = Command::new("bash")
            .arg("-c")
            .arg(&command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let mut combined = stdout;
                if !stderr.is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr);
                }

                if !output.status.success() {
                    let exit_code = output.status.code().unwrap_or(-1);
                    ToolResult::error(format!("exit {exit_code}\n{combined}"))
                } else {
                    ToolResult::success(combined)
                }
            }
            Err(e) => ToolResult::error(format!("执行失败: {e}")),
        }
    }
}

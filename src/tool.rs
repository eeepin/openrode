use hillm::types::{FunctionDefinition, Tool, ToolType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub output: String,
    pub error: bool,
}

/// 执行 bash 命令
pub async fn execute_bash(command: &str) -> ToolResult {
    let result = Command::new("bash")
        .arg("-c")
        .arg(command)
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
                ToolResult {
                    output: format!("exit {exit_code}\n{combined}"),
                    error: true,
                }
            } else {
                ToolResult {
                    output: combined,
                    error: false,
                }
            }
        }
        Err(e) => ToolResult {
            output: format!("执行失败: {e}"),
            error: true,
        },
    }
}

/// 执行工具调用
pub async fn execute_tool(name: &str, arguments: &str) -> ToolResult {
    match name {
        "bash" => {
            // 解析参数
            match serde_json::from_str::<Value>(arguments) {
                Ok(args) => {
                    if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                        execute_bash(cmd).await
                    } else {
                        ToolResult {
                            output: "错误: 缺少 command 参数".to_string(),
                            error: true,
                        }
                    }
                }
                Err(e) => ToolResult {
                    output: format!("参数解析失败: {e}"),
                    error: true,
                },
            }
        }
        _ => ToolResult {
            output: format!("未知工具: {name}"),
            error: true,
        },
    }
}

/// 获取可用工具定义
pub fn get_tools() -> Vec<Tool> {
    vec![Tool {
        tool_type: ToolType::Function,
        function: FunctionDefinition {
            name: "bash".to_string(),
            description: Some("执行 shell 命令，返回 stdout+stderr".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "要执行的 shell 命令"
                    }
                },
                "required": ["command"]
            })),
            strict: None,
        },
    }]
}

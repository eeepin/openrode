//! 示例插件：日志记录插件
//!
//! 这个插件演示了如何：
//! 1. 提供插件元数据
//! 2. 创建自定义工具
//! 3. 实现工具执行钩子

use async_trait::async_trait;
use openrode::plugin::PluginMetadata;
use openrode::tool::{AgentTool, ToolResult};
use serde::Deserialize;
use serde_json::Value;

/// 插件元数据
static METADATA: PluginMetadata = PluginMetadata {
    name: "logging-plugin",
    version: "0.1.0",
    description: "记录所有工具调用到日志文件",
    author: Some("OpenRode Team"),
};

/// 必须实现：返回插件元数据
#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *const PluginMetadata {
    &METADATA as *const PluginMetadata
}

/// 可选实现：创建插件提供的工具
#[no_mangle]
pub extern "C" fn create_plugin_tools() -> *mut Vec<Box<dyn AgentTool>> {
    let tools: Vec<Box<dyn AgentTool>> = vec![
        Box::new(LogTool),
        Box::new(StatsTool::new()),
    ];
    Box::into_raw(Box::new(tools))
}

/// 可选实现：工具执行前钩子
#[no_mangle]
pub extern "C" fn on_tool_before(
    name: *const std::ffi::c_char,
    input: *const std::ffi::c_char,
) -> bool {
    unsafe {
        let name = std::ffi::CStr::from_ptr(name).to_str().unwrap_or("");
        let input = std::ffi::CStr::from_ptr(input).to_str().unwrap_or("");

        // 记录到日志文件
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("plugin.log")
        {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 调用: {}({})", chrono_timestamp(), name, input);
        }
    }
    true // 允许执行
}

/// 可选实现：工具执行后钩子
#[no_mangle]
pub extern "C" fn on_tool_after(
    name: *const std::ffi::c_char,
    input: *const std::ffi::c_char,
    output: *const std::ffi::c_char,
) {
    unsafe {
        let name = std::ffi::CStr::from_ptr(name).to_str().unwrap_or("");
        let output = std::ffi::CStr::from_ptr(output).to_str().unwrap_or("");

        // 记录到日志文件
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("plugin.log")
        {
            use std::io::Write;
            let _ = writeln!(file, "[{}] 完成: {} -> {} 字节", chrono_timestamp(), name, output.len());
        }
    }
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", duration.as_secs())
}

// ============================================================================
// 自定义工具示例
// ============================================================================

/// 日志工具：写入自定义日志
struct LogTool;

#[derive(Deserialize)]
struct LogInput {
    message: String,
    level: Option<String>,
}

#[async_trait]
impl AgentTool for LogTool {
    fn name(&self) -> &str {
        "log"
    }

    fn description(&self) -> &str {
        "写入自定义日志消息"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "日志消息"
                },
                "level": {
                    "type": "string",
                    "description": "日志级别 (info, warn, error)",
                    "enum": ["info", "warn", "error"]
                }
            },
            "required": ["message"]
        })
    }

    async fn run(&self, input: Value) -> ToolResult {
        let LogInput { message, level } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {}", e)),
        };

        let level = level.unwrap_or_else(|| "info".to_string());
        let timestamp = chrono_timestamp();

        // 写入日志文件
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("plugin.log")
        {
            use std::io::Write;
            let _ = writeln!(file, "[{}] [{}] {}", timestamp, level.to_uppercase(), message);
        }

        ToolResult::success(format!("日志已写入: [{}] {}", level, message))
    }
}

/// 统计工具：显示工具调用统计
struct StatsTool {
    call_count: std::sync::atomic::AtomicUsize,
}

impl StatsTool {
    fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl AgentTool for StatsTool {
    fn name(&self) -> &str {
        "stats"
    }

    fn description(&self) -> &str {
        "显示工具调用统计信息"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn run(&self, _input: Value) -> ToolResult {
        let count = self.call_count.load(std::sync::atomic::Ordering::Relaxed);

        // 读取日志文件统计
        let log_stats = if let Ok(content) = std::fs::read_to_string("plugin.log") {
            let lines: Vec<&str> = content.lines().collect();
            let tool_calls = lines.iter().filter(|l| l.contains("调用:")).count();
            let tool_completes = lines.iter().filter(|l| l.contains("完成:")).count();

            format!(
                "日志文件统计:\n\
                 - 总行数: {}\n\
                 - 工具调用: {}\n\
                 - 工具完成: {}",
                lines.len(),
                tool_calls,
                tool_completes
            )
        } else {
            "日志文件不存在".to_string()
        };

        ToolResult::success(format!(
            "统计工具调用次数: {}\n\n{}",
            count,
            log_stats
        ))
    }
}

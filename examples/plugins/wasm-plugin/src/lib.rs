//! WASM 插件示例
//!
//! 这个插件演示了如何编写 WASM 插件

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 插件元数据
#[derive(Serialize)]
struct PluginMetadata {
    name: String,
    version: String,
    description: String,
    author: Option<String>,
}

/// 工具定义
#[derive(Serialize, Deserialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: Value,
}

/// 工具执行结果
#[derive(Serialize)]
struct ToolResult {
    output: Option<String>,
    error: Option<String>,
}

/// 获取插件元数据
#[no_mangle]
pub extern "C" fn get_metadata() -> *const u8 {
    let metadata = PluginMetadata {
        name: "wasm-example".to_string(),
        version: "0.1.0".to_string(),
        description: "WASM 插件示例".to_string(),
        author: Some("OpenRode Team".to_string()),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}

/// 列出工具
#[no_mangle]
pub extern "C" fn list_tools() -> *const u8 {
    let tools = vec![
        ToolDefinition {
            name: "wasm_echo".to_string(),
            description: "回显输入的消息".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "要回显的消息"
                    }
                },
                "required": ["message"]
            }),
        },
        ToolDefinition {
            name: "wasm_calculate".to_string(),
            description: "执行简单的数学计算".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "操作类型"
                    },
                    "a": {
                        "type": "number",
                        "description": "第一个操作数"
                    },
                    "b": {
                        "type": "number",
                        "description": "第二个操作数"
                    }
                },
                "required": ["operation", "a", "b"]
            }),
        },
    ];

    let json = serde_json::to_string(&tools).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}

/// 分配内存
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// 释放内存
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}

/// 执行工具
#[no_mangle]
pub extern "C" fn run(input_ptr: i32) -> i32 {
    // 读取输入（简化实现）
    let input_str = unsafe {
        let ptr = input_ptr as *const u8;
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
    };

    let input: Value = match serde_json::from_str(input_str) {
        Ok(v) => v,
        Err(e) => {
            let result = ToolResult {
                output: None,
                error: Some(format!("解析输入失败: {}", e)),
            };
            let json = serde_json::to_string(&result).unwrap();
            let bytes = json.into_bytes();
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);
            return ptr;
        }
    };

    // 简单的工具路由
    let result = if let Some(tool_name) = input.get("tool").and_then(|v| v.as_str()) {
        match tool_name {
            "wasm_echo" => handle_echo(&input),
            "wasm_calculate" => handle_calculate(&input),
            _ => ToolResult {
                output: None,
                error: Some(format!("未知工具: {}", tool_name)),
            },
        }
    } else {
        ToolResult {
            output: None,
            error: Some("缺少 tool 字段".to_string()),
        }
    };

    let json = serde_json::to_string(&result).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as i32;
    std::mem::forget(bytes);
    ptr
}

/// 处理回显工具
fn handle_echo(input: &Value) -> ToolResult {
    let message = input
        .get("arguments")
        .and_then(|args| args.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    ToolResult {
        output: Some(format!("Echo: {}", message)),
        error: None,
    }
}

/// 处理计算工具
fn handle_calculate(input: &Value) -> ToolResult {
    let args = match input.get("arguments") {
        Some(args) => args,
        None => {
            return ToolResult {
                output: None,
                error: Some("缺少 arguments 字段".to_string()),
            };
        }
    };

    let operation = match args.get("operation").and_then(|v| v.as_str()) {
        Some(op) => op,
        None => {
            return ToolResult {
                output: None,
                error: Some("缺少 operation 字段".to_string()),
            };
        }
    };

    let a = match args.get("a").and_then(|v| v.as_f64()) {
        Some(n) => n,
        None => {
            return ToolResult {
                output: None,
                error: Some("缺少或无效的 a 字段".to_string()),
            };
        }
    };

    let b = match args.get("b").and_then(|v| v.as_f64()) {
        Some(n) => n,
        None => {
            return ToolResult {
                output: None,
                error: Some("缺少或无效的 b 字段".to_string()),
            };
        }
    };

    let result = match operation {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return ToolResult {
                    output: None,
                    error: Some("除数不能为零".to_string()),
                };
            }
            a / b
        }
        _ => {
            return ToolResult {
                output: None,
                error: Some(format!("未知操作: {}", operation)),
            };
        }
    };

    ToolResult {
        output: Some(format!("{} {} {} = {}", a, operation, b, result)),
        error: None,
    }
}

//! WASM 插件加载器
//!
//! 使用 wasmtime 加载和执行 WASM 插件，提供更好的沙箱隔离。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use wasmtime::*;

use super::PluginMetadata;
use crate::tool::{AgentTool, ToolResult};

/// WASM 插件
pub struct WasmPlugin {
    pub metadata: PluginMetadata,
    engine: Engine,
    module: Module,
    store: Store<WasmContext>,
}

/// WASM 插件上下文
struct WasmContext {
    // 可以添加共享状态
}

/// WASM 工具包装器
struct WasmTool {
    name: String,
    description: String,
    input_schema: Value,
    instance: Arc<Mutex<Instance>>,
    store: Arc<Mutex<Store<WasmContext>>>,
}

use std::sync::Mutex;

#[async_trait::async_trait]
impl AgentTool for WasmTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn run(&self, input: Value) -> ToolResult {
        let mut store = self.store.lock().unwrap();
        let instance = self.instance.lock().unwrap();

        // 调用 WASM 模块的 run 函数
        let run_func = instance
            .get_typed_func::<i32, i32>(&mut *store, "run")
            .map_err(|e| ToolResult::error(format!("找不到 run 函数: {}", e)));

        let run_func = match run_func {
            Ok(f) => f,
            Err(e) => return e,
        };

        // 将输入序列化为 JSON 字符串
        let input_json = serde_json::to_string(&input).unwrap_or_default();

        // 分配内存并写入输入
        let alloc_func = instance
            .get_typed_func::<i32, i32>(&mut *store, "alloc")
            .map_err(|e| ToolResult::error(format!("找不到 alloc 函数: {}", e)));

        let alloc_func = match alloc_func {
            Ok(f) => f,
            Err(e) => return e,
        };

        let input_ptr = match alloc_func.call(&mut *store, input_json.len() as i32) {
            Ok(ptr) => ptr,
            Err(e) => return ToolResult::error(format!("内存分配失败: {}", e)),
        };

        // 写入输入数据
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| ToolResult::error("找不到 memory".to_string()));

        let memory = match memory {
            Ok(m) => m,
            Err(e) => return e,
        };

        let input_bytes = input_json.as_bytes();
        if let Err(e) = memory.write(&mut *store, input_ptr as usize, input_bytes) {
            return ToolResult::error(format!("写入内存失败: {}", e));
        }

        // 调用 run 函数
        let result_ptr = match run_func.call(&mut *store, input_ptr) {
            Ok(ptr) => ptr,
            Err(e) => return ToolResult::error(format!("执行失败: {}", e)),
        };

        // 读取结果
        // 假设结果以 null 结尾的字符串形式存储
        let mut result_bytes = Vec::new();
        let mut offset = result_ptr as usize;
        loop {
            let mut byte = [0u8; 1];
            if let Err(e) = memory.read(&mut *store, offset, &mut byte) {
                return ToolResult::error(format!("读取内存失败: {}", e));
            }
            if byte[0] == 0 {
                break;
            }
            result_bytes.push(byte[0]);
            offset += 1;
        }

        let result_str = String::from_utf8_lossy(&result_bytes).to_string();

        // 尝试解析为 JSON
        match serde_json::from_str::<Value>(&result_str) {
            Ok(result_json) => {
                if let Some(error) = result_json.get("error") {
                    ToolResult::error(error.as_str().unwrap_or("未知错误").to_string())
                } else if let Some(output) = result_json.get("output") {
                    ToolResult::success(output.as_str().unwrap_or("").to_string())
                } else {
                    ToolResult::success(result_str)
                }
            }
            Err(_) => ToolResult::success(result_str),
        }
    }
}

impl WasmPlugin {
    /// 加载 WASM 插件
    pub fn load(path: &Path) -> Result<Self> {
        let wasm_bytes = std::fs::read(path)
            .with_context(|| format!("读取 WASM 文件失败: {:?}", path))?;

        let engine = Engine::default();
        let module = Module::new(&engine, &wasm_bytes)
            .with_context(|| "编译 WASM 模块失败")?;

        let mut store = Store::new(&engine, WasmContext {});

        // 创建临时实例以获取元数据
        let linker = Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module)?;

        // 调用 get_metadata 函数
        let get_metadata = instance
            .get_typed_func::<(), i32>(&mut store, "get_metadata")
            .context("找不到 get_metadata 函数")?;

        let metadata_ptr = get_metadata.call(&mut store, ())?;

        // 读取元数据
        let memory = instance
            .get_memory(&mut store, "memory")
            .context("找不到 memory")?;

        // 假设元数据以 JSON 格式存储
        let mut metadata_bytes = Vec::new();
        let mut offset = metadata_ptr as usize;
        loop {
            let mut byte = [0u8; 1];
            memory.read(&mut store, offset, &mut byte)?;
            if byte[0] == 0 {
                break;
            }
            metadata_bytes.push(byte[0]);
            offset += 1;
        }

        let metadata_str = String::from_utf8_lossy(&metadata_bytes);
        let metadata: PluginMetadata = serde_json::from_str(&metadata_str)
            .context("解析插件元数据失败")?;

        Ok(Self {
            metadata,
            engine,
            module,
            store,
        })
    }

    /// 获取插件提供的工具列表
    pub fn get_tools(&mut self) -> Result<Vec<Box<dyn AgentTool>>> {
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut self.store, &self.module)?;

        // 调用 list_tools 函数
        let list_tools = instance
            .get_typed_func::<(), i32>(&mut self.store, "list_tools")
            .context("找不到 list_tools 函数")?;

        let tools_ptr = list_tools.call(&mut self.store, ())?;

        // 读取工具列表（JSON 格式）
        let memory = instance
            .get_memory(&mut self.store, "memory")
            .context("找不到 memory")?;

        let mut tools_bytes = Vec::new();
        let mut offset = tools_ptr as usize;
        loop {
            let mut byte = [0u8; 1];
            memory.read(&mut self.store, offset, &mut byte)?;
            if byte[0] == 0 {
                break;
            }
            tools_bytes.push(byte[0]);
            offset += 1;
        }

        let tools_str = String::from_utf8_lossy(&tools_bytes);

        // 解析工具定义
        #[derive(Deserialize)]
        struct ToolDef {
            name: String,
            description: String,
            input_schema: Value,
        }

        let tool_defs: Vec<ToolDef> = serde_json::from_str(&tools_str)
            .context("解析工具列表失败")?;

        let instance = Arc::new(Mutex::new(instance));
        let store = Arc::new(Mutex::new(Store::new(&self.engine, WasmContext {})));

        let tools: Vec<Box<dyn AgentTool>> = tool_defs
            .into_iter()
            .map(|def| {
                Box::new(WasmTool {
                    name: def.name,
                    description: def.description,
                    input_schema: def.input_schema,
                    instance: instance.clone(),
                    store: store.clone(),
                }) as Box<dyn AgentTool>
            })
            .collect();

        Ok(tools)
    }
}

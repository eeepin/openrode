//! 插件系统
//!
//! 支持通过动态库和 WASM 加载插件，扩展系统功能。
//! 插件可以提供自定义工具、钩子函数等。

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tool::AgentTool;

pub mod wasm;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub path: PathBuf,
    #[serde(default)]
    pub config: Option<Value>,
}

/// 插件注册表
pub type PluginRegistry = Arc<RwLock<Vec<LoadedPlugin>>>;

/// 从插件注册表中提取所有工具（消耗性操作）
pub async fn extract_tools(registry: &PluginRegistry) -> Vec<Box<dyn AgentTool>> {
    let mut plugins = registry.write().await;
    let mut tools = Vec::new();

    for plugin in plugins.iter_mut() {
        // 使用 std::mem::take 将工具从插件中移出
        let plugin_tools = std::mem::take(&mut plugin.tools);
        tools.extend(plugin_tools);
    }

    tools
}

/// 已加载的插件
pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub library: Option<Library>,
    pub tools: Vec<Box<dyn AgentTool>>,
}

// 确保 LoadedPlugin 可以在线程间共享
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

/// 插件 API：获取元数据的函数签名
type GetMetadataFn = unsafe extern "C" fn() -> *const PluginMetadata;

/// 插件 API：创建工具的函数签名
type CreateToolsFn = unsafe extern "C" fn() -> *mut Vec<Box<dyn AgentTool>>;

/// 插件 API：工具执行前钩子
type OnToolBeforeFn = unsafe extern "C" fn(name: *const std::ffi::c_char, input: *const std::ffi::c_char) -> bool;

/// 插件 API：工具执行后钩子
type OnToolAfterFn = unsafe extern "C" fn(name: *const std::ffi::c_char, input: *const std::ffi::c_char, output: *const std::ffi::c_char);

/// 动态插件加载器
pub struct DynamicPluginLoader;

impl DynamicPluginLoader {
    /// 加载插件
    pub fn load(config: PluginConfig) -> Result<LoadedPlugin> {
        if !config.path.exists() {
            anyhow::bail!("插件文件不存在: {:?}", config.path);
        }

        unsafe {
            let lib = Library::new(&config.path)
                .with_context(|| format!("加载插件库失败: {:?}", config.path))?;

            // 加载元数据
            let get_metadata: Symbol<GetMetadataFn> = lib.get(b"get_plugin_metadata\0")
                .with_context(|| "找不到 get_plugin_metadata 函数")?;

            let metadata_ptr = get_metadata();
            if metadata_ptr.is_null() {
                anyhow::bail!("get_plugin_metadata 返回空指针");
            }

            let metadata = (*metadata_ptr).clone();

            // 加载工具（可选）
            let tools = if let Ok(create_tools) = lib.get::<CreateToolsFn>(b"create_plugin_tools\0") {
                let tools_ptr = create_tools();
                if tools_ptr.is_null() {
                    Vec::new()
                } else {
                    // 直接获取 Vec 的所有权，而不是克隆
                    *Box::from_raw(tools_ptr)
                }
            } else {
                Vec::new()
            };

            Ok(LoadedPlugin {
                metadata,
                config,
                library: Some(lib),
                tools,
            })
        }
    }

    /// 调用插件的工具执行前钩子
    pub fn call_on_tool_before(plugin: &LoadedPlugin, name: &str, input: &Value) -> bool {
        if let Some(ref lib) = plugin.library {
            unsafe {
                if let Ok(on_before) = lib.get::<OnToolBeforeFn>(b"on_tool_before\0") {
                    let name_c = std::ffi::CString::new(name).unwrap();
                    let input_json = serde_json::to_string(input).unwrap();
                    let input_c = std::ffi::CString::new(input_json).unwrap();

                    on_before(name_c.as_ptr(), input_c.as_ptr())
                } else {
                    true // 没有钩子函数，允许执行
                }
            }
        } else {
            true // WASM 插件暂不支持钩子
        }
    }

    /// 调用插件的工具执行后钩子
    pub fn call_on_tool_after(plugin: &LoadedPlugin, name: &str, input: &Value, output: &str) {
        if let Some(ref lib) = plugin.library {
            unsafe {
                if let Ok(on_after) = lib.get::<OnToolAfterFn>(b"on_tool_after\0") {
                    let name_c = std::ffi::CString::new(name).unwrap();
                    let input_json = serde_json::to_string(input).unwrap();
                    let input_c = std::ffi::CString::new(input_json).unwrap();
                    let output_c = std::ffi::CString::new(output).unwrap();

                    on_after(name_c.as_ptr(), input_c.as_ptr(), output_c.as_ptr());
                }
            }
        }
        // WASM 插件暂不支持钩子
    }
}

/// 加载插件配置
pub async fn load_plugin_config(config_path: &Path) -> Result<Vec<PluginConfig>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(config_path).await
        .with_context(|| format!("读取插件配置文件失败: {:?}", config_path))?;

    let configs: Vec<PluginConfig> = serde_json::from_str(&content)
        .with_context(|| "解析插件配置失败")?;

    Ok(configs)
}

/// 创建插件注册表
pub async fn create_registry(config_paths: &[PathBuf]) -> Result<PluginRegistry> {
    let mut plugins = Vec::new();

    for config_path in config_paths {
        match load_plugin_config(config_path).await {
            Ok(configs) => {
                for config in configs {
                    if !config.enabled {
                        continue;
                    }

                    // 根据文件扩展名决定加载方式
                    let extension = config.path.extension().and_then(|e| e.to_str());

                    let load_result = match extension {
                        Some("wasm") => {
                            // 加载 WASM 插件
                            match wasm::WasmPlugin::load(&config.path) {
                                Ok(mut wasm_plugin) => {
                                    let metadata = wasm_plugin.metadata.clone();
                                    let tools = wasm_plugin.get_tools().unwrap_or_else(|e| {
                                        eprintln!("获取 WASM 插件工具失败: {}", e);
                                        Vec::new()
                                    });

                                    Ok(LoadedPlugin {
                                        metadata: metadata.clone(),
                                        config: config.clone(),
                                        library: None, // WASM 插件不使用动态库
                                        tools,
                                    })
                                }
                                Err(e) => Err(e),
                            }
                        }
                        _ => {
                            // 加载动态库插件
                            DynamicPluginLoader::load(config.clone())
                        }
                    };

                    match load_result {
                        Ok(plugin) => {
                            println!("已加载插件: {} v{}", plugin.metadata.name, plugin.metadata.version);
                            plugins.push(plugin);
                        }
                        Err(e) => {
                            eprintln!("加载插件 {:?} 失败: {}", config.path, e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("加载插件配置 {:?} 失败: {}", config_path, e);
            }
        }
    }

    Ok(Arc::new(RwLock::new(plugins)))
}

/// 列出已加载的插件
pub async fn list_plugins(registry: &PluginRegistry) -> Vec<String> {
    let plugins = registry.read().await;
    plugins.iter().map(|p| {
        format!(
            "{} v{} - {} ({} 个工具)",
            p.metadata.name,
            p.metadata.version,
            p.metadata.description,
            p.tools.len()
        )
    }).collect()
}

/// 获取所有插件提供的工具
pub async fn get_all_tools(registry: &PluginRegistry) -> Vec<Box<dyn AgentTool>> {
    let plugins = registry.read().await;
    let tools = Vec::new();

    for _plugin in plugins.iter() {
        // 注意：由于工具已经注册到 ToolRegistry，这里不需要再返回
        // 保留此函数供未来扩展使用
    }

    tools
}

/// 执行所有插件的工具执行前钩子
pub async fn run_tool_before_hooks(
    registry: &PluginRegistry,
    name: &str,
    input: &Value,
) -> bool {
    let plugins = registry.read().await;

    for plugin in plugins.iter() {
        if !DynamicPluginLoader::call_on_tool_before(plugin, name, input) {
            return false; // 某个插件阻止了执行
        }
    }

    true
}

/// 执行所有插件的工具执行后钩子
pub async fn run_tool_after_hooks(
    registry: &PluginRegistry,
    name: &str,
    input: &Value,
    output: &str,
) {
    let plugins = registry.read().await;

    for plugin in plugins.iter() {
        DynamicPluginLoader::call_on_tool_after(plugin, name, input, output);
    }
}

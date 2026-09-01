//! 插件系统
//!
//! 支持通过动态库加载插件，扩展系统功能。
//! 插件可以提供自定义工具、钩子函数等。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub config: Option<serde_json::Value>,
}

/// 插件注册表
pub type PluginRegistry = Arc<RwLock<Vec<LoadedPlugin>>>;

/// 已加载的插件
#[derive(Clone)]
pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    // 未来可以添加工具、钩子等
}

/// 加载插件配置
pub async fn load_plugin_config(config_path: &Path) -> Result<Vec<PluginConfig>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(config_path).await?;
    let configs: Vec<PluginConfig> = serde_json::from_str(&content)?;
    Ok(configs)
}

/// 创建插件注册表
pub async fn create_registry(config_paths: &[PathBuf]) -> Result<PluginRegistry> {
    let mut plugins = Vec::new();

    for config_path in config_paths {
        match load_plugin_config(config_path).await {
            Ok(configs) => {
                for config in configs {
                    if config.enabled {
                        // 目前只是记录插件配置，实际加载动态库需要 libloading
                        // 这里暂时只加载元数据
                        let metadata = PluginMetadata {
                            name: config.path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            version: "0.1.0".to_string(),
                            description: "Plugin".to_string(),
                            author: None,
                        };

                        plugins.push(LoadedPlugin {
                            metadata,
                            config,
                        });
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
        format!("{} v{} - {}", p.metadata.name, p.metadata.version, p.metadata.description)
    }).collect()
}

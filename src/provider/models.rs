//! 模型目录
//!
//! 已知模型及其能力信息

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// Provider
    pub provider: String,
    /// 上下文窗口大小（tokens）
    pub context_window: usize,
    /// 最大输出 tokens
    pub max_output_tokens: Option<usize>,
    /// 是否支持工具调用
    pub supports_tools: bool,
    /// 是否支持视觉
    pub supports_vision: bool,
    /// 是否支持流式输出
    pub supports_streaming: bool,
    /// 输入价格（每百万 tokens）
    pub input_price_usd: Option<f64>,
    /// 输出价格（每百万 tokens）
    pub output_price_usd: Option<f64>,
}

impl ModelInfo {
    pub fn new(id: impl Into<String>, provider: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            provider: provider.into(),
            context_window: 128_000,
            max_output_tokens: Some(4096),
            supports_tools: true,
            supports_vision: false,
            supports_streaming: true,
            input_price_usd: None,
            output_price_usd: None,
        }
    }

    pub fn with_context_window(mut self, size: usize) -> Self {
        self.context_window = size;
        self
    }

    pub fn with_max_output(mut self, size: usize) -> Self {
        self.max_output_tokens = Some(size);
        self
    }

    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }

    pub fn with_pricing(mut self, input: f64, output: f64) -> Self {
        self.input_price_usd = Some(input);
        self.output_price_usd = Some(output);
        self
    }
}

/// 模型目录
pub struct ModelCatalog {
    models: HashMap<String, ModelInfo>,
}

impl ModelCatalog {
    /// 创建包含内置模型的目录
    pub fn new() -> Self {
        let mut catalog = Self {
            models: HashMap::new(),
        };
        catalog.load_builtin_models();
        catalog
    }

    /// 加载内置模型
    fn load_builtin_models(&mut self) {
        // Anthropic Claude
        self.add(
            ModelInfo::new("claude-3-opus-20240229", "anthropic")
                .with_context_window(200_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(15.0, 75.0),
        );

        self.add(
            ModelInfo::new("claude-3-sonnet-20240229", "anthropic")
                .with_context_window(200_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(3.0, 15.0),
        );

        self.add(
            ModelInfo::new("claude-3-haiku-20240307", "anthropic")
                .with_context_window(200_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(0.25, 1.25),
        );

        self.add(
            ModelInfo::new("claude-sonnet-4-5", "anthropic")
                .with_context_window(200_000)
                .with_max_output(8192)
                .with_vision()
                .with_pricing(3.0, 15.0),
        );

        self.add(
            ModelInfo::new("claude-opus-4-5", "anthropic")
                .with_context_window(200_000)
                .with_max_output(8192)
                .with_vision()
                .with_pricing(15.0, 75.0),
        );

        // OpenAI GPT
        self.add(
            ModelInfo::new("gpt-4-turbo", "openai")
                .with_context_window(128_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(10.0, 30.0),
        );

        self.add(
            ModelInfo::new("gpt-4o", "openai")
                .with_context_window(128_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(5.0, 15.0),
        );

        self.add(
            ModelInfo::new("gpt-4o-mini", "openai")
                .with_context_window(128_000)
                .with_max_output(4096)
                .with_vision()
                .with_pricing(0.15, 0.60),
        );

        self.add(
            ModelInfo::new("o1-preview", "openai")
                .with_context_window(128_000)
                .with_max_output(32768)
                .with_pricing(15.0, 60.0),
        );

        self.add(
            ModelInfo::new("o1-mini", "openai")
                .with_context_window(128_000)
                .with_max_output(65536)
                .with_pricing(3.0, 12.0),
        );

        // Google Gemini
        self.add(
            ModelInfo::new("gemini-pro", "google")
                .with_context_window(32_760)
                .with_max_output(8192),
        );

        self.add(
            ModelInfo::new("gemini-1.5-pro", "google")
                .with_context_window(1_000_000)
                .with_max_output(8192)
                .with_vision()
                .with_pricing(3.5, 10.5),
        );

        self.add(
            ModelInfo::new("gemini-1.5-flash", "google")
                .with_context_window(1_000_000)
                .with_max_output(8192)
                .with_vision()
                .with_pricing(0.35, 1.05),
        );

        // Qwen (通过 OpenAI 兼容 API)
        self.add(
            ModelInfo::new("qwen-plus", "openai")
                .with_context_window(131_072)
                .with_max_output(8192),
        );

        self.add(
            ModelInfo::new("qwen-max", "openai")
                .with_context_window(32_768)
                .with_max_output(8192),
        );

        self.add(
            ModelInfo::new("qwen3.7-plus", "openai")
                .with_context_window(131_072)
                .with_max_output(8192),
        );
    }

    /// 添加模型
    pub fn add(&mut self, model: ModelInfo) {
        self.models.insert(model.id.clone(), model);
    }

    /// 获取模型信息
    #[allow(dead_code)]
    pub fn get(&self, model_id: &str) -> Option<&ModelInfo> {
        self.models.get(model_id)
    }

    /// 获取所有模型
    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// 按 provider 过滤模型
    #[allow(dead_code)]
    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// 检查模型是否支持工具调用
    #[allow(dead_code)]
    pub fn supports_tools(&self, model_id: &str) -> bool {
        self.get(model_id).map(|m| m.supports_tools).unwrap_or(true)
    }

    /// 获取模型的上下文窗口
    #[allow(dead_code)]
    pub fn context_window(&self, model_id: &str) -> usize {
        self.get(model_id)
            .map(|m| m.context_window)
            .unwrap_or(128_000)
    }
}

impl Default for ModelCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_catalog_builtin() {
        let catalog = ModelCatalog::new();

        // 检查内置模型
        let claude = catalog.get("claude-3-opus-20240229").unwrap();
        assert_eq!(claude.provider, "anthropic");
        assert_eq!(claude.context_window, 200_000);
        assert!(claude.supports_vision);

        let gpt4 = catalog.get("gpt-4o").unwrap();
        assert_eq!(gpt4.provider, "openai");
        assert!(gpt4.supports_vision);
    }

    #[test]
    fn test_model_catalog_list_by_provider() {
        let catalog = ModelCatalog::new();

        let anthropic_models = catalog.list_by_provider("anthropic");
        assert!(!anthropic_models.is_empty());
        assert!(anthropic_models.iter().all(|m| m.provider == "anthropic"));

        let openai_models = catalog.list_by_provider("openai");
        assert!(!openai_models.is_empty());
        assert!(openai_models.iter().all(|m| m.provider == "openai"));
    }

    #[test]
    fn test_model_info_builder() {
        let model = ModelInfo::new("test-model", "test-provider")
            .with_context_window(100_000)
            .with_max_output(2048)
            .with_vision()
            .with_pricing(1.0, 2.0);

        assert_eq!(model.context_window, 100_000);
        assert_eq!(model.max_output_tokens, Some(2048));
        assert!(model.supports_vision);
        assert_eq!(model.input_price_usd, Some(1.0));
        assert_eq!(model.output_price_usd, Some(2.0));
    }
}

//! Provider 工具函数
//!
//! 提供 provider 推断和默认配置的辅助函数

pub mod models;

/// 从模型名称推断 provider 名称
///
/// 返回 hillm 使用的 provider 名称字符串
pub fn infer_provider_from_model(model: &str) -> &'static str {
    let model_lower = model.to_lowercase();

    // OpenRouter 格式: provider/model（优先检查）
    if model_lower.contains('/') {
        return "openrouter";
    }

    // Anthropic Claude
    if model_lower.contains("claude") {
        return "anthropic";
    }

    // Google Gemini
    if model_lower.contains("gemini") {
        return "google";
    }

    // OpenAI GPT / O1 / O3
    if model_lower.starts_with("gpt-")
        || model_lower.starts_with("o1")
        || model_lower.starts_with("o3")
        || model_lower.starts_with("chatgpt")
    {
        return "openai";
    }

    // Ollama 本地模型
    if model_lower.contains("llama")
        || model_lower.contains("mistral:")
        || model_lower.contains("qwen:")
    {
        return "ollama";
    }

    // 默认使用 OpenAI 兼容
    "openai"
}

/// 获取 provider 的默认 base URL
pub fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "anthropic" => "https://api.anthropic.com/v1",
        "google" => "https://generativelanguage.googleapis.com/v1beta",
        "ollama" => "http://localhost:11434/v1",
        "openrouter" => "https://openrouter.ai/api/v1",
        "bedrock" => "", // AWS SDK handles this
        _ => "",
    }
}

/// 获取 provider 的默认环境变量名（API key）
pub fn default_env_var(provider: &str) -> &'static str {
    match provider {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "google" => "GOOGLE_API_KEY",
        "ollama" => "",
        "openrouter" => "OPENROUTER_API_KEY",
        "bedrock" => "AWS_ACCESS_KEY_ID",
        _ => "API_KEY",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_provider_from_model() {
        assert_eq!(infer_provider_from_model("claude-3-opus"), "anthropic");
        assert_eq!(infer_provider_from_model("claude-sonnet-4-5"), "anthropic");
        assert_eq!(infer_provider_from_model("gpt-4-turbo"), "openai");
        assert_eq!(infer_provider_from_model("gpt-4o"), "openai");
        assert_eq!(infer_provider_from_model("o1-preview"), "openai");
        assert_eq!(infer_provider_from_model("gemini-pro"), "google");
        assert_eq!(
            infer_provider_from_model("anthropic/claude-3"),
            "openrouter"
        );
    }

    #[test]
    fn test_default_base_url() {
        assert_eq!(default_base_url("openai"), "https://api.openai.com/v1");
        assert_eq!(
            default_base_url("anthropic"),
            "https://api.anthropic.com/v1"
        );
        assert_eq!(default_base_url("ollama"), "http://localhost:11434/v1");
    }

    #[test]
    fn test_default_env_var() {
        assert_eq!(default_env_var("openai"), "OPENAI_API_KEY");
        assert_eq!(default_env_var("anthropic"), "ANTHROPIC_API_KEY");
        assert_eq!(default_env_var("ollama"), "");
    }
}

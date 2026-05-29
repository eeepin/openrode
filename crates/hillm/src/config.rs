use secrecy::{ExposeSecret, SecretString};

/// Default OpenAI API base url
pub const BASE_URL_OPENAI: &str = "https://api.openai.com/v1";

/// [crate::Client] relies on this for api calls
pub trait Config: Send + Sync {
    fn base_url(&self) -> &str;
    fn api_key(&self) -> &SecretString;
}

pub struct OpenAIConfig {
    base_url: String,
    api_key: SecretString,
}

impl OpenAIConfig {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| BASE_URL_OPENAI.to_string()),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
        }
    }
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = base_url.into();
        self
    }
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = SecretString::from(api_key.into());
        self
    }
}

impl Config for OpenAIConfig {
    fn base_url(&self) -> &str {
        self.base_url
    }
    fn api_key(&self) -> &SecretString {
        self.api_key
    }
}

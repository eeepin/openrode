use crate::config::Config;
use reqwest;
pub struct Client<C: Config> {
    request_client: reqwest::Client,
    config: C,
}

impl<C: Config> Client {
    pub fn new() -> Self {
        let request_client = reqwest::Client::new();
        Self {
            request_client,
            config: C::new(),
        }
    }
    pub fn build(request_client: reqwest::Client, config: C) -> Self {
        Self {
            request_client,
            config,
        }
    }
    pub fn with_request_client(mut self, request_client: reqwest::Client) -> Self {
        self.request_client = request_client;
        self
    }
    pub fn with_config(mut self, config: C) -> Self {
        self.config = config;
        self
    }
}

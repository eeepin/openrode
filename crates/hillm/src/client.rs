use crate::config::Config;
use crate::types::error::{APICallError, HiError, ParseError};
use crate::types::header::RequestHeader;
use crate::types::request::Request;
use crate::types::response::{Response, ResponseStreamChunk};
use futures::stream::{self, Stream, StreamExt};
use reqwest;
use reqwest::header::HeaderMap;
use secrecy::ExposeSecret;

pub struct Client<C: Config> {
    request_client: reqwest::Client,
    config: C,
}

impl<C: Config> Client<C> {
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

    pub async fn chat(&self, mut request: Request) -> Result<Response, HiError> {
        request.stream = false;
        let url = format!("{}/chat/completions", self.config.base_url());
        let api_key = self.config.api_key().expose_secret();
        let header = HeaderMap::from(RequestHeader::new().with_auth(api_key.to_string()));
        let response = self
            .request_client
            .post(&url)
            .headers(header)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                HiError::ParseError(ParseError {
                    message: format!("Request failed: {}", e),
                    raw_body: None,
                })
            })?;
        let status = response.status();
        let body = response.text().await.map_err(|e| {
            HiError::ParseError(ParseError {
                message: format!("Failed to read response body: {}", e),
                raw_body: None,
            })
        })?;
        if !status.is_success() {
            let api_error: APICallError = serde_json::from_str(&body).map_err(|e| {
                HiError::ParseError(ParseError {
                    message: format!("Failed to parse error response: {}", e),
                    raw_body: Some(body.clone()),
                })
            })?;
            return Err(HiError::APICallError(api_error));
        }
        let response: Response = serde_json::from_str(&body).map_err(|e| {
            HiError::ParseError(ParseError {
                message: format!("Failed to parse response: {}", e),
                raw_body: Some(body.clone()),
            })
        })?;
        Ok(response)
    }

    pub async fn chat_stream(
        &self,
        mut request: Request,
    ) -> Result<impl Stream<Item = Result<ResponseStreamChunk, HiError>>, HiError> {
        request.stream = true;
        let url = format!("{}/chat/completions", self.config.base_url());
        let api_key = self.config.api_key().expose_secret();
        let header = HeaderMap::from(RequestHeader::new().with_auth(api_key.to_string()));
        let response = self
            .request_client
            .post(&url)
            .headers(header)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                HiError::ParseError(ParseError {
                    message: format!("Request failed: {}", e),
                    raw_body: None,
                })
            })?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.map_err(|e| {
                HiError::ParseError(ParseError {
                    message: format!("Failed to read error response body: {}", e),
                    raw_body: None,
                })
            })?;
            let api_error: APICallError = serde_json::from_str(&body).map_err(|e| {
                HiError::ParseError(ParseError {
                    message: format!("Failed to parse error response: {}", e),
                    raw_body: Some(body.clone()),
                })
            })?;
            return Err(HiError::APICallError(api_error));
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(|chunk_result| {
                chunk_result.map_err(|e| {
                    HiError::ParseError(ParseError {
                        message: format!("Stream error: {}", e),
                        raw_body: None,
                    })
                })
            })
            .flat_map(|chunk_result| match chunk_result {
                Err(e) => {
                    let items: Vec<Result<ResponseStreamChunk, HiError>> = vec![Err(e)];
                    stream::iter(items).left_stream()
                }
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    let chunks = parse_sse_lineselines(&text);
                    stream::iter(chunks).right_stream()
                }
            });

        Ok(stream)
    }
}

fn parse_sse_lineselines(text: &str) -> Vec<Result<ResponseStreamChunk, HiError>> {
    let mut results = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                break;
            }

            match serde_json::from_str::<ResponseStreamChunk>(data) {
                Ok(chunk) => results.push(Ok(chunk)),
                Err(e) => {
                    results.push(Err(HiError::ParseError(ParseError {
                        message: format!("Failed to parse stream chunk: {}", e),
                        raw_body: Some(data.to_string()),
                    })));
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpenRouterConfig;
    use serde_json::json;

    #[tokio::test]
    #[ignore]
    async fn test_chat_basic() {
        let config = OpenRouterConfig::new()
            .with_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1");
        let client = Client::build(reqwest::Client::new(), config);
        let request: Request = serde_json::from_value(json!({
            "messages": [
                {"role": "user", "content": "Say hello"}
            ],
            "model": "qwen3.7-plus"
        }))
        .unwrap();

        let response = client.chat(request).await.unwrap();
        println!("{:#?}", response);
    }
}

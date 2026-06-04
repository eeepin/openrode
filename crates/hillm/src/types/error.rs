use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror;

#[derive(Deserialize, Serialize, Debug, thiserror::Error)]
pub enum HiError {
    #[error("Error when doing API calls: {0}")]
    APICallError(APICallError),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct APICallError {
    error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    openrouter_metadata: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

impl std::fmt::Display for APICallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut e = Vec::new();
        if let Some(user_id) = &self.user_id {
            e.push(format!("[{}]", user_id));
        };
        e.push(format!("{}", &self.error));
        if let Some(openrouter_metadata) = &self.openrouter_metadata {
            e.push(format!("[Error Openrouter Metadata("));
            for (key, value) in openrouter_metadata.iter() {
                e.push(format!("{}:{}", key, value));
            }
            e.push(format!(")]"));
        };
        write!(f, "{}", e.join(" "))
    }
}

impl std::error::Error for APICallError {}

#[derive(Deserialize, Serialize, Debug)]
struct ErrorDetail {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Value>>,
}
impl std::fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut e = Vec::new();
        match reqwest::StatusCode::from_u16(self.code) {
            Ok(status_code) => {
                e.push(format!("[{}]", &status_code));
            }
            Err(_) => {
                e.push(format!("[Invalid Status Code({})]", &self.code));
            }
        }
        e.push(format!("[Error Message({})]", &self.message));
        if let Some(metadata) = &self.metadata {
            e.push(format!("[Error Metadata("));
            for (key, value) in metadata.iter() {
                e.push(format!("{}:{}", key, value));
            }
            e.push(format!(")]"));
        };
        write!(f, "{}", e.join(" "))
    }
}

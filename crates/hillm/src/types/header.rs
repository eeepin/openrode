use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestHeader {
    #[serde(rename = "Authorization")]
    authorization: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "X-OpenRouter-Metadata")]
    xopen_router_metadata: Option<XOpenRouterMetadata>,
    #[serde(rename = "Content-Type")]
    content_type: ContentType,
}

impl RequestHeader {
    pub fn new() -> Self {
        Self {
            authorization: String::new(),
            xopen_router_metadata: Some(XOpenRouterMetadata::Disabled),
            content_type: ContentType::Json,
        }
    }

    pub fn build(auth: String, xopen_router_metadata: Option<XOpenRouterMetadata>) -> Self {
        Self {
            authorization: format!("Bearer {}", auth),
            xopen_router_metadata: xopen_router_metadata,
            content_type: ContentType::Json,
        }
    }

    pub fn with_auth(mut self, auth: String) -> Self {
        self.authorization = format!("Bearer {}", auth);
        self
    }

    pub fn with_xopen_router_metadata(
        mut self,
        xopen_router_metadata: Option<XOpenRouterMetadata>,
    ) -> Self {
        self.xopen_router_metadata = xopen_router_metadata;
        self
    }
}

impl From<RequestHeader> for HeaderMap {
    fn from(header: RequestHeader) -> Self {
        let mut map = HeaderMap::new();
        let header = serde_json::to_value(header).unwrap();
        if let Some(obj) = header.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.parse::<HeaderName>().unwrap(), s.parse().unwrap());
                }
            }
        }
        map
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum XOpenRouterMetadata {
    Disabled,
    Enabled,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ContentType {
    #[serde(rename = "application/json")]
    Json,
}

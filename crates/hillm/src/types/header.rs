use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestHeader {
    #[serde(rename = "Authorization")]
    authorization: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "X-OpenRouter-Metadata")]
    xopen_router_metadata: Option<XOpenRouterMetadata>,
}

impl RequestHeader {
    pub fn new() -> Self {
        Self {
            authorization: build_header_auth(std::env::var("API_KEY").unwrap_or_default().into()),
            xopen_router_metadata: Some(XOpenRouterMetadata::Disabled),
        }
    }

    pub fn build(auth: String, xopen_router_metadata: Option<XOpenRouterMetadata>) -> Self {
        Self {
            authorization: build_header_auth(auth),
            xopen_router_metadata: xopen_router_metadata,
        }
    }

    pub fn with_auth(mut self, auth: String) -> Self {
        self.authorization = build_header_auth(auth);
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum XOpenRouterMetadata {
    Disabled,
    Enabled,
}

fn build_header_auth(auth: String) -> String {
    "Bearer ".to_string() + &auth
}

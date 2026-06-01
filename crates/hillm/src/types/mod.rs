use serde::{Deserialize, Serialize};
pub mod message;
pub mod request;

#[derive(Deserialize, Serialize, Debug)]
pub struct CacheControl {
    #[serde(rename = "type")]
    #[serde(default = "cache_control_type")]
    cache_control_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<TTLType>,
}

fn cache_control_type() -> String {
    "ephemeral".to_string()
}

#[derive(Deserialize, Serialize, Debug)]
enum TTLType {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "1h")]
    OneHour,
}

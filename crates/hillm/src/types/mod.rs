use serde::{Deserialize, Serialize};
pub mod message;
pub mod request;
use crate::func_return_string;

#[derive(Deserialize, Serialize, Debug)]
pub struct CacheControl {
    #[serde(rename = "type")]
    #[serde(default = "ephemeral")]
    cache_control_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<TTLType>,
}

#[derive(Deserialize, Serialize, Debug)]
enum TTLType {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "1h")]
    OneHour,
}

func_return_string!(ephemeral);

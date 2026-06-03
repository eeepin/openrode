use crate::types::CacheControl;
use crate::{func_return_string, func_return_string_};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Messages {
    messages: Vec<Message>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Message {
    System {
        #[serde(default = "system")]
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: MessageContent,
    },
    Developer {
        #[serde(default = "developer")]
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: MessageContent,
    },
    User {
        #[serde(default = "system")]
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        content: MessageContent,
    },
    Tool {
        #[serde(default = "tool")]
        role: String,
        tool_call_id: String,
        content: MessageContent,
    },
    Assistant {
        #[serde(default = "assistant")]
        role: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<AudioPart>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<MessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        images: Option<Vec<ImagePart>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning_details: Option<Vec<ReasoningDetail>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum MessageContent {
    TextContentPart(String),
    ArrayContentPart(Vec<ContentPart>),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ContentPart {
    Text {
        #[serde(rename = "type")]
        #[serde(default = "text")]
        content_type: String,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    File {
        #[serde(rename = "type")]
        #[serde(default = "file")]
        content_type: String,
        file: FilePart,
    },
    Image {
        #[serde(rename = "type")]
        #[serde(default = "image_url")]
        content_type: String,
        image_url: ImageContent,
    },
    Audio {
        #[serde(rename = "type")]
        #[serde(default = "input_audio")]
        content_type: String,
        input_audio: AudioContent,
    },
    Vedio {
        #[serde(rename = "type")]
        #[serde(default = "video_url")]
        content_type: String,
        video_url: UrlPart,
    },
}

#[derive(Deserialize, Serialize, Debug)]
struct AudioContent {
    data: String,
    format: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct FilePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AudioPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcript: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ImagePart {
    image_url: UrlPart,
}
#[derive(Deserialize, Serialize, Debug)]
struct UrlPart {
    url: String,
}
#[derive(Deserialize, Serialize, Debug)]
struct ImageContent {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<VisionDetailLevel>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
enum VisionDetailLevel {
    Auto,
    Low,
    High,
}
#[derive(Deserialize, Serialize, Debug)]
struct ReasoningDetail {
    reasoning: ReasoningDetailType,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ReasoningDetailType {
    Encrypted {
        #[serde(rename = "type")]
        #[serde(default = "reasoning_encrypted")]
        reasoning_type: String,
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ReasoningFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
    Summary {
        #[serde(rename = "type")]
        #[serde(default = "reasoning_summary")]
        reasoning_type: String,
        summary: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ReasoningFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
    Text {
        #[serde(rename = "type")]
        #[serde(default = "reasoning_text")]
        reasoning_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ReasoningFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
enum ReasoningFormat {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "openai-responses-v1")]
    OpenaiResponsesV1,
    #[serde(rename = "azure-openai-responses-v1")]
    AzureOpenaiResponsesV1,
    #[serde(rename = "xai-responses-v1")]
    XaiResponsesV1,
    #[serde(rename = "anthropic-claude-v1")]
    AnthropicClaudeV1,
    #[serde(rename = "google-gemini-v1")]
    GoogleGeminiV1,
}

#[derive(Deserialize, Serialize, Debug)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    #[serde(default = "function")]
    tool_type: String,
    function: FunctionCallInfo,
}

#[derive(Deserialize, Serialize, Debug)]
struct FunctionCallInfo {
    arguments: String,
    name: String,
}

func_return_string!(system);
func_return_string!(developer);
func_return_string!(user);
func_return_string!(tool);
func_return_string!(assistant);
func_return_string!(text);
func_return_string!(file);
func_return_string_!(image_url);
func_return_string_!(input_audio);
func_return_string_!(video_url);
fn reasoning_encrypted() -> String {
    "reasoning.encrypted".to_string()
}
fn reasoning_summary() -> String {
    "reasoning.summary".to_string()
}
fn reasoning_text() -> String {
    "reasoning.text".to_string()
}
func_return_string!(function);

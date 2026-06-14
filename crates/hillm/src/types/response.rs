use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{func_return_string, types::message::AssistantMessage};

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    choices: Vec<Choice>,
    created: u32,
    id: String,
    model: String,
    object: ResponseType,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    openrouter_metadata: Option<OpenRouterMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,
    usage: Usage,
}

#[derive(Deserialize, Serialize, Debug)]
struct Choice {
    finish_reason: FinishReason,
    index: u32,
    message: AssistantMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<LogProbs>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum FinishReason {
    #[serde(rename = "tool_calls")]
    ToolCalls,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
    #[serde(rename = "error")]
    Error,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LogProbs {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Vec<LogProb>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refusal: Option<Vec<LogProb>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct LogProb {
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<Vec<u32>>,
    logprob: f32,
    token: String,
    top_logprobs: Vec<TopLogProb>,
}

#[derive(Deserialize, Serialize, Debug)]
struct TopLogProb {
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<Vec<u32>>,
    logprob: f32,
    token: String,
}

#[derive(Deserialize, Serialize, Debug)]
enum ResponseType {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenRouterMetaData {
    attempt: u32,
    endpoints: Endpoints,
    is_byok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    requested: String,
    strategy: Strategy,
    summary: String,
    attempts: Vec<Attempt>,
    params: Params,
    pipeline: Vec<Pipeline>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Endpoints {
    available: Vec<Available>,
    total: u32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Available {
    model: String,
    provider: String,
    selected: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Strategy {
    Direct,
    Auto,
    Free,
    Latest,
    Alias,
    Fallback,
    Pareto,
    Bodybuilder,
    Fusion,
}

#[derive(Deserialize, Serialize, Debug)]
struct Attempt {
    model: String,
    provider: String,
    status: u32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Params {
    quality_floor: f32,
    throughput_floor: f32,
    version_group: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Pipeline {
    name: String,
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_usd: Option<f32>,
    data: HashMap<String, Value>,
    guardrail_id: String,
    guardrail_scope: String,
    summary: String,
}

#[derive(Deserialize, Serialize, Debug)]
enum PipelineType {
    #[serde(rename = "guardrail")]
    Guardrail,
    #[serde(rename = "plugin")]
    Plugin,
    #[serde(rename = "server_tools")]
    ServerTools,
    #[serde(rename = "response_healing")]
    ResponseHealing,
    #[serde(rename = "context_compression")]
    ContextCompression,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Usage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    completion_tokens_details: Option<CompletionTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_details: Option<CostDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_byok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    accepted_prediction_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rejected_prediction_tokens: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostDetails {
    upstream_inference_completions_cost: f32,
    upstream_inference_prompt_cost: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream_inference_cost: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct PromptTokensDetails {
    audio_tokens: u32,
    cache_write_tokens: u32,
    cached_tokens: u32,
    video_tokens: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamChunk {
    pub id: String,
    pub created: u32,
    pub model: String,
    pub object: StreamChunkType,
    pub choices: Vec<StreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_metadata: Option<OpenRouterMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum StreamChunkType {
    #[serde(rename = "chat.completion.chunk")]
    ChatCompletionChunk,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: StreamDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<StreamToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamToolCall {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    #[serde(default = "function")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<StreamFunctionCall>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamFunctionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

func_return_string!(function);

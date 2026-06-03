use std::collections::HashMap;

use crate::types::{CacheControl, message::Messages};
use crate::{func_return_string, func_return_string_};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    messages: Messages,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<DebugFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<ImageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logit_bias: Option<HashMap<String, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modalities: Option<Modalities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugins: Option<Vec<Plugin>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Reasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<ServiceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Stop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_server_tools_when: Option<Vec<StopServerToolsWhen>>,
    #[serde(default = "default_false")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Trace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

fn default_false() -> bool {
    false
}

#[derive(Deserialize, Serialize, Debug)]
struct DebugFlag {
    #[serde(skip_serializing_if = "Option::is_none")]
    echo_upstream_body: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Modalities {
    Text,
    Image,
    Audio,
}

#[derive(Deserialize, Serialize, Debug)]
enum ImageConfig {
    ImageConfigStr(String),
    ImageConfigDouble(f32),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Plugin {
    AutoRouter {
        #[serde(default = "auto_router")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_models: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cost_quality_tradeoff: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    ContextCompression {
        #[serde(default = "context_compression")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        engine: Option<CompressionEngine>,
    },
    FileParser {
        #[serde(default = "file_parser")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pdf: Option<Pdf>,
    },
    Fusion {
        #[serde(default = "fusion")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        analysis_models: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_tool_calls: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    Moderation {
        #[serde(default = "moderation")]
        id: String,
    },
    ParetoRouter {
        #[serde(default = "pareto_router")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_coding_score: Option<f32>,
    },
    ResponseHealing {
        #[serde(default = "response_healing")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    Web {
        #[serde(default = "web")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        engine: Option<WebEngine>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        include_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_results: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_uses: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        search_prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<UserLocation>,
    },
    WebFetch {
        #[serde(default = "web_fetch")]
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocked_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_content_tokens: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_uses: Option<u32>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
enum CompressionEngine {
    #[serde(rename = "middle-out")]
    MiddleOut,
}

#[derive(Deserialize, Serialize, Debug)]
struct Pdf {
    engine: PdfEngine,
}

#[derive(Deserialize, Serialize, Debug)]
enum PdfEngine {
    #[serde(rename = "mistral-ocr")]
    MistralOCR,
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "cloudflare-ai")]
    CloudflareAI,
}

#[derive(Deserialize, Serialize, Debug)]
enum WebEngine {
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "exa")]
    Exa,
    #[serde(rename = "firecrawl")]
    Firecrawl,
    #[serde(rename = "parallel")]
    Parallel,
}

#[derive(Deserialize, Serialize, Debug)]
struct UserLocation {
    #[serde(rename = "approximate")]
    user_location_type: UserLocationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
enum UserLocationType {
    #[serde(rename = "approximate")]
    Approximate,
}

#[derive(Deserialize, Serialize, Debug)]
struct Provider {
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_collection: Option<DataCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_distillable_text: Option<bool>,
    // ignore ProviderName maybe a enum
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_price: Option<MaxPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    only: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_max_latency: Option<Latency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_min_throughput: Option<Latency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantizations: Option<Vec<Quantizations>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    require_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<SortStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zdr: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum DataCollection {
    Allow,
    Deny,
}

// The object specifying the maximum price you want to pay for this request.
// USD price per million tokens, for prompt and completion.
#[derive(Deserialize, Serialize, Debug)]
struct MaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Latency {
    Double(f32),
    Obj(PercentileLatencyCutoffs),
}

#[derive(Deserialize, Serialize, Debug)]
struct PercentileLatencyCutoffs {
    #[serde(skip_serializing_if = "Option::is_none")]
    p50: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    p75: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    p90: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    p99: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Quantizations {
    Int4,
    Int8,
    Fp4,
    Fp6,
    Fp8,
    Fp16,
    Bf16,
    Fp32,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum SortStrategy {
    ProviderSort(ProviderSortStrategy),
    ProviderSortConfig(ProviderSortConfigStrategy),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ProviderSortStrategy {
    Price,
    Throughput,
    Latency,
    Exacto,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum PartitioningStrategy {
    Model,
    None,
}

#[derive(Deserialize, Serialize, Debug)]
struct ProviderSortConfigStrategy {
    #[serde(skip_serializing_if = "Option::is_none")]
    by: Option<ProviderSortStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    partition: Option<PartitioningStrategy>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<EffortType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<SummaryType>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum EffortType {
    Xhigh,
    High,
    Medium,
    Low,
    Minimal,
    None,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum SummaryType {
    Auto,
    Concise,
    Detailed,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ResponseFormat {
    Grammar {
        #[serde(rename = "type")]
        #[serde(default = "grammar")]
        format_type: String,
        grammar: String,
    },
    JsonObject {
        #[serde(rename = "type")]
        #[serde(default = "json_object")]
        format_type: String,
    },
    JsonSchema {
        #[serde(rename = "type")]
        #[serde(default = "json_schema")]
        format_type: String,
        json_schema: JsonSchemaConfig,
    },
    Python {
        #[serde(rename = "type")]
        #[serde(default = "python")]
        format_type: String,
    },
    Text {
        #[serde(rename = "type")]
        #[serde(default = "text")]
        format_type: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
struct JsonSchemaConfig {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strict: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ServiceTier {
    Auto,
    Default,
    Flex,
    Priority,
    Scale,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Stop {
    Single(String),
    Muilt(Vec<String>),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum StopServerToolsWhen {
    FinishReasonIs {
        #[serde(rename = "type")]
        #[serde(default = "finish_reason_is")]
        stop_type: String,
        reason: String,
    },
    HasToolCall {
        #[serde(rename = "type")]
        #[serde(default = "has_tool_call")]
        stop_type: String,
        tool_name: String,
    },
    MaxCost {
        #[serde(rename = "type")]
        #[serde(default = "max_cost")]
        stop_type: String,
        max_cost_in_dollars: f32,
    },
    MaxTokensUsed {
        #[serde(rename = "type")]
        #[serde(default = "max_tokens_used")]
        stop_type: String,
        max_tokens: u32,
    },
    StepCountIs {
        #[serde(rename = "type")]
        #[serde(default = "step_count_is")]
        stop_type: String,
        step_count: u32,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ToolChoice {
    Choice(ToolChoiceType),
    Config(ToolChoiceConfig),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ToolChoiceType {
    Auto,
    None,
    Required,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum ToolChoiceConfig {
    ChatNamedToolChoice {
        #[serde(rename = "type")]
        #[serde(default = "function")]
        tool_choice_type: String,
        function: FunctionWithName,
    },
    ChatServerToolChoice {
        #[serde(rename = "type")]
        tool_choice_type: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
struct FunctionWithName {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Function {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strict: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum Tool {
    BaseTool {
        #[serde(rename = "type")]
        #[serde(default = "function")]
        tool_type: String,
        function: Function,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    DatetimeServerTool {
        #[serde(rename = "type")]
        #[serde(default = "openrouter_datetime")]
        tool_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<TimeZone>,
    },
    ImageGenerationServerToolOpenRouter {
        #[serde(rename = "type")]
        #[serde(default = "openrouter_image_generation")]
        tool_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<ModelName>,
    },
    ChatSearchModelsServerTool {
        #[serde(rename = "type")]
        #[serde(default = "openrouter_experimental_search_models")]
        tool_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<MaxResults>,
    },
    WebFetchServerTool {
        #[serde(rename = "type")]
        #[serde(default = "openrouter_web_fetch")]
        tool_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<WebFetchServerToolParam>,
    },
    OpenRouterWebSearchServerTool {
        #[serde(rename = "type")]
        #[serde(default = "openrouter_web_search")]
        tool_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<OpenRouterWebSearchServerToolParam>,
    },
    ChatWebSearchShorthand {
        #[serde(rename = "type")]
        tool_type: ChatWebSearchShorthandType,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        excluded_domains: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_results: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_total_results: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        engine: Option<WebSearchToolEngine>,
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<SearchContextSize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<UserLocation>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<OpenRouterWebSearchServerToolParam>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
struct TimeZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ModelName {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct MaxResults {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
struct WebFetchServerToolParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_content_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engine: Option<WebFetchServerToolEngine>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum WebFetchServerToolEngine {
    Auto,
    Native,
    Openrouter,
    Exa,
    Parallel,
    Firecrawl,
}

#[derive(Deserialize, Serialize, Debug)]
struct OpenRouterWebSearchServerToolParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excluded_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_total_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engine: Option<WebSearchToolEngine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_context_size: Option<SearchContextSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_location: Option<UserLocation>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum WebSearchToolEngine {
    Auto,
    Native,
    Exa,
    Parallel,
    Firecrawl,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum SearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Deserialize, Serialize, Debug)]
enum ChatWebSearchShorthandType {
    #[serde(rename = "web_search")]
    WebSearch,
    #[serde(rename = "web_search_preview")]
    WebSearchPreview,
    #[serde(rename = "web_search_preview_2025_03_11")]
    WebSearchPreview2025_03_11,
    #[serde(rename = "web_search_2025_08_26")]
    WebSearch2025_08_26,
}

#[derive(Deserialize, Serialize, Debug)]
struct Trace {
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_name: Option<String>,
}

func_return_string!(auto_router);
func_return_string!(context_compression);
func_return_string!(file_parser);
func_return_string!(fusion);
func_return_string!(moderation);
func_return_string!(pareto_router);
func_return_string!(response_healing);
func_return_string!(web);
func_return_string!(web_fetch);
func_return_string!(grammar);
func_return_string_!(json_object);
func_return_string_!(json_schema);
func_return_string!(python);
func_return_string!(text);
func_return_string_!(finish_reason_is);
func_return_string_!(has_tool_call);
func_return_string_!(max_cost);
func_return_string_!(max_tokens_used);
func_return_string_!(step_count_is);
func_return_string!(function);
fn openrouter_datetime() -> String {
    "openrouter:datetime".to_string()
}
fn openrouter_image_generation() -> String {
    "openrouter:image_generation".to_string()
}
fn openrouter_experimental_search_models() -> String {
    "openrouter:experimental__search_models".to_string()
}
fn openrouter_web_fetch() -> String {
    "openrouter:web_fetch".to_string()
}
fn openrouter_web_search() -> String {
    "openrouter:web_search".to_string()
}

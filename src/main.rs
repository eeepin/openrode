use clap::Parser;
use dotenv::dotenv;
use futures::StreamExt;
use hillm::ClientBuilder;
use hillm::client::ChatCompletionClient;
use hillm::types::ChatCompletionRequest;
use serde_json::json;
use std::env;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap_or_default();
    let args = Args::parse();
    println!("{}", args.prompt);

    let request: ChatCompletionRequest = serde_json::from_value(json!({
        "messages": [
            {"role": "user", "content": args.prompt}
        ],
        "model": "qwen3.7-plus"
    }))
    .unwrap();

    let client = ClientBuilder::new()
        .api_key(api_key)
        .provider("openai")
        .base_url("https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1")
        .build()?;
    let mut stream = client.chat_stream(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        for choice in chunk.choices {
            if let Some(content) = choice.delta.content {
                print!("{}", content);
            }
        }
    }

    Ok(())
}

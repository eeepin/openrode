use clap::Parser;
use futures::StreamExt;
use hillm::client::Client;
use hillm::config::OpenRouterConfig;
use hillm::types::request::Request;
use serde_json::json;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{}", args.prompt);

    let request: Request = serde_json::from_value(json!({
        "messages": [
            {"role": "user", "content": args.prompt}
        ],
        "model": "qwen3.7-plus"
    }))
    .unwrap();

    let client: Client<OpenRouterConfig> = Client::new();
    let mut stream = client.chat_stream(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        for choice in chunk.choices {
            if let Some(content) = choice.delta.content {
                print!("{}", content);
            }
        }
    }
    println!();

    Ok(())
}

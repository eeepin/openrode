use clap::Parser;
use dotenv::dotenv;
use hillm::ClientBuilder;
use hillm::client::ChatCompletionClient;
use std::env;

mod agent;
mod tool;

#[derive(Parser)]
#[command(author, version, about = "OpenRode - AI 编程智能体")]
struct Args {
    /// 要发送给智能体的提示
    #[arg(short = 'p', long)]
    prompt: Option<String>,

    /// 使用的模型
    #[arg(short, long, default_value = "qwen3.7-plus")]
    model: String,

    /// LLM 提供商
    #[arg(long, default_value = "openai")]
    provider: String,

    /// API base URL
    #[arg(
        long,
        default_value = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
    )]
    base_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap_or_default();
    let args = Args::parse();

    let prompt = match args.prompt {
        Some(p) => p,
        None => {
            eprintln!("错误: 请提供提示 (-p \"your prompt\")");
            std::process::exit(1);
        }
    };

    println!("模型: {}", args.model);
    println!("提示: {}\n", prompt);

    // 构建 LLM 客户端
    let client = ClientBuilder::new()
        .api_key(api_key)
        .provider(&args.provider)
        .base_url(&args.base_url)
        .build()?;

    let boxed_client: Box<dyn ChatCompletionClient> = Box::new(client);

    // 创建并运行代理循环
    let mut loop_agent = agent::AgentLoop::new(boxed_client, args.model);
    loop_agent.run(&prompt).await?;

    Ok(())
}

use clap::Parser;
use dotenv::dotenv;
use hillm::ClientBuilder;
use hillm::client::ChatCompletionClient;
use std::env;

mod agent;
mod prompt;
mod session;
mod storage;
mod tool;

use storage::Storage;

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

    /// 继续最近的会话
    #[arg(short = 'c', long)]
    r#continue: bool,

    /// 恢复指定会话
    #[arg(short = 's', long)]
    session: Option<String>,

    /// 列出所有会话
    #[arg(short = 'l', long)]
    list: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap_or_default();
    let args = Args::parse();

    // 获取当前工作目录
    let cwd = env::current_dir()?;

    // 初始化存储
    let storage = storage::file::FileStorage::default_storage().await?;

    // 列出会话
    if args.list {
        let sessions = storage.list_sessions().await?;
        if sessions.is_empty() {
            println!("没有会话记录");
        } else {
            let header_title = "标题";
            println!("{:<30} {:<24} {:<20} {header_title}", "ID", "时间", "模型");
            println!("{}", "-".repeat(100));
            for s in sessions {
                let time = s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
                let title = if s.title.is_empty() {
                    "(无标题)"
                } else {
                    &s.title
                };
                println!("{:<30} {:<24} {:<20} {}", s.id, time, s.model, title);
            }
        }
        return Ok(());
    }

    let prompt = match args.prompt {
        Some(p) => p,
        None => {
            eprintln!("错误: 请提供提示 (-p \"your prompt\")");
            std::process::exit(1);
        }
    };

    // 构建 LLM 客户端
    let client = ClientBuilder::new()
        .api_key(api_key)
        .provider(&args.provider)
        .base_url(&args.base_url)
        .build()?;

    let boxed_client: Box<dyn ChatCompletionClient> = Box::new(client);
    let boxed_storage: Box<dyn Storage> = Box::new(storage);

    // 决定是新建还是恢复会话
    let mut agent_loop = if let Some(session_id) = args.session {
        // 恢复指定会话
        println!("恢复会话: {session_id}");
        agent::AgentLoop::resume(boxed_client, &session_id, boxed_storage, cwd.clone()).await?
    } else if args.r#continue {
        // 恢复最近会话
        match boxed_storage.latest_session_id().await? {
            Some(id) => {
                println!("继续最近会话: {id}");
                agent::AgentLoop::resume(boxed_client, &id, boxed_storage, cwd.clone()).await?
            }
            None => {
                println!("没有历史会话，创建新会话");
                agent::AgentLoop::new(boxed_client, args.model, boxed_storage, cwd.clone()).await?
            }
        }
    } else {
        // 新建会话
        println!("模型: {}", args.model);
        agent::AgentLoop::new(boxed_client, args.model, boxed_storage, cwd.clone()).await?
    };

    println!("会话 ID: {}", agent_loop.session_id());
    println!("提示: {prompt}\n");

    agent_loop.run(&prompt).await?;

    Ok(())
}

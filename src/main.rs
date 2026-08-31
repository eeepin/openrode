use clap::Parser;
use dotenv::dotenv;
use hillm::ClientBuilder;
use hillm::client::ChatCompletionClient;
use std::env;

mod agent;
mod permission;
mod prompt;
mod provider;
mod server;
mod session;
mod skill;
mod snapshot;
mod storage;
mod tool;
mod tui;

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

    /// LLM 提供商 (openai, anthropic, google, ollama, openrouter, custom)
    /// 如果不指定，会从模型名自动推断
    #[arg(long)]
    provider: Option<String>,

    /// API base URL（覆盖默认值）
    #[arg(long)]
    base_url: Option<String>,

    /// API key（覆盖环境变量）
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,

    /// 继续最近的会话
    #[arg(short = 'c', long)]
    r#continue: bool,

    /// 恢复指定会话
    #[arg(short = 's', long)]
    session: Option<String>,

    /// 列出所有会话
    #[arg(short = 'l', long)]
    list: bool,

    /// 列出可用模型
    #[arg(long)]
    list_models: bool,

    /// 启动 HTTP 服务器
    #[arg(long)]
    serve: bool,

    /// 服务器监听地址
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: String,

    /// 启动 TUI 界面
    #[arg(long)]
    tui: bool,

    /// TUI 连接的服务器地址
    #[arg(long, default_value = "http://127.0.0.1:3000")]
    server_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // 列出模型
    if args.list_models {
        let catalog = provider::models::ModelCatalog::new();
        println!(
            "{:<35} {:<12} {:<12} {:<8} {:<8}",
            "模型", "Provider", "上下文", "工具", "视觉"
        );
        println!("{}", "-".repeat(80));
        for model in catalog.list() {
            let context = format!("{}K", model.context_window / 1000);
            let tools = if model.supports_tools { "✓" } else { "✗" };
            let vision = if model.supports_vision { "✓" } else { "✗" };
            println!(
                "{:<35} {:<12} {:<12} {:<8} {:<8}",
                model.id, model.provider, context, tools, vision
            );
        }
        return Ok(());
    }

    // 获取当前工作目录
    let cwd = env::current_dir()?;

    // 初始化技能注册表
    let skill_dirs = vec![
        cwd.join(".openrode").join("skills"),
        dirs::home_dir()
            .unwrap_or_default()
            .join(".openrode")
            .join("skills"),
    ];
    let skill_registry = skill::create_registry(&skill_dirs).await.ok();

    // 初始化存储
    let storage = storage::file::FileStorage::default_storage().await?;

    // 启动服务器
    if args.serve {
        let storage: std::sync::Arc<dyn Storage> = std::sync::Arc::new(storage);
        return server::start_server(storage, &args.addr).await;
    }

    // 启动 TUI
    if args.tui {
        let tui_client = tui::TuiClient::new(&args.server_url);
        return tui_client.run().await;
    }

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

    // 确定 provider（优先使用命令行参数，否则从模型名推断）
    let provider = args
        .provider
        .unwrap_or_else(|| provider::infer_provider_from_model(&args.model).to_string());

    // 确定 base URL
    let base_url = args
        .base_url
        .unwrap_or_else(|| provider::default_base_url(&provider).to_string());

    // 确定 API key
    let api_key = args.api_key.unwrap_or_else(|| {
        let env_var = provider::default_env_var(&provider);
        if env_var.is_empty() {
            String::new()
        } else {
            env::var(env_var).unwrap_or_default()
        }
    });

    // 对于 Ollama，不需要 API key
    let api_key = if provider == "ollama" && api_key.is_empty() {
        "ollama".to_string()
    } else {
        api_key
    };

    // 构建 LLM 客户端
    let client = ClientBuilder::new()
        .api_key(api_key)
        .provider(&provider)
        .base_url(&base_url)
        .build()?;

    let boxed_client: Box<dyn ChatCompletionClient> = Box::new(client);
    let boxed_storage: Box<dyn Storage> = Box::new(storage);
    let model = args.model;

    println!("模型: {} (provider: {})", model, provider);

    // 决定是新建还是恢复会话
    let mut agent_loop = if let Some(session_id) = args.session {
        // 恢复指定会话
        println!("恢复会话: {session_id}");
        agent::AgentLoop::resume(boxed_client, &session_id, boxed_storage, cwd.clone(), skill_registry).await?
    } else if args.r#continue {
        // 恢复最近会话
        match boxed_storage.latest_session_id().await? {
            Some(id) => {
                println!("继续最近会话: {id}");
                agent::AgentLoop::resume(boxed_client, &id, boxed_storage, cwd.clone(), skill_registry).await?
            }
            None => {
                println!("没有历史会话，创建新会话");
                agent::AgentLoop::new(boxed_client, model, boxed_storage, cwd.clone(), skill_registry).await?
            }
        }
    } else {
        // 新建会话
        agent::AgentLoop::new(boxed_client, model, boxed_storage, cwd.clone(), skill_registry).await?
    };

    println!("会话 ID: {}", agent_loop.session_id());
    println!("提示: {prompt}\n");

    agent_loop.run(&prompt).await?;

    Ok(())
}

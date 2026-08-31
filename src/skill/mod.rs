//! 技能系统
//!
//! 技能是可复用的提示模板，可以包含指令、上下文和工具调用示例。
//! 技能文件使用 Markdown 格式，带有 YAML frontmatter。

mod loader;
mod tool;

pub use loader::{load_skills, Skill};
pub use tool::SkillTool;

use std::sync::Arc;
use tokio::sync::RwLock;

/// 技能注册表
pub type SkillRegistry = Arc<RwLock<Vec<Skill>>>;

/// 创建技能注册表并加载技能
pub async fn create_registry(skill_dirs: &[std::path::PathBuf]) -> anyhow::Result<SkillRegistry> {
    let mut skills = Vec::new();

    for dir in skill_dirs {
        if dir.exists() {
            match load_skills(dir).await {
                Ok(loaded) => skills.extend(loaded),
                Err(e) => eprintln!("加载技能目录 {:?} 失败: {}", dir, e),
            }
        }
    }

    Ok(Arc::new(RwLock::new(skills)))
}

/// 生成技能列表（用于系统提示）
pub async fn skill_list(registry: &SkillRegistry) -> String {
    let skills = registry.read().await;
    if skills.is_empty() {
        return String::new();
    }

    let mut output = String::from("可用技能（使用 /技能名 调用）：\n");
    for skill in skills.iter() {
        output.push_str(&format!("- /{}: {}\n", skill.name, skill.description));
    }
    output
}

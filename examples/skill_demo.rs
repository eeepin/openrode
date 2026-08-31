use openrode::skill::{self, SkillRegistry};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建测试技能目录
    let skill_dir = PathBuf::from(".openrode/skills");
    if !skill_dir.exists() {
        println!("请先创建技能目录并添加技能文件");
        return Ok(());
    }

    // 加载技能
    let registry = skill::create_registry(&[skill_dir]).await?;

    // 显示技能列表
    let skill_list = skill::skill_list(&registry).await;
    println!("已加载的技能：");
    println!("{}", skill_list);

    // 读取技能内容
    let skills = registry.read().await;
    for skill in skills.iter() {
        println!("\n=== 技能: {} ===", skill.name);
        println!("描述: {}", skill.description);
        println!("内容:\n{}", skill.content);
    }

    Ok(())
}

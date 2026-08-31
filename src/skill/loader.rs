//! 技能加载器
//!
//! 从文件系统加载技能文件。技能文件使用 Markdown 格式，
//! 带有 YAML frontmatter 定义元数据。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 技能名称（用于 /技能名 调用）
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 技能内容（Markdown 格式的提示模板）
    pub content: String,
    /// 技能来源文件路径
    #[serde(skip)]
    pub source: Option<String>,
}

/// 技能文件的 frontmatter
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
}

/// 从目录加载所有技能
pub async fn load_skills(dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !dir.exists() {
        return Ok(skills);
    }

    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // 只处理 .md 文件
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match load_skill_file(&path).await {
            Ok(skill) => skills.push(skill),
            Err(e) => eprintln!("加载技能文件 {:?} 失败: {}", path, e),
        }
    }

    Ok(skills)
}

/// 加载单个技能文件
async fn load_skill_file(path: &Path) -> Result<Skill> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("读取文件失败: {:?}", path))?;

    // 解析 frontmatter
    let (frontmatter, body) = parse_frontmatter(&content)
        .with_context(|| format!("解析 frontmatter 失败: {:?}", path))?;

    Ok(Skill {
        name: frontmatter.name,
        description: frontmatter.description,
        content: body.trim().to_string(),
        source: Some(path.to_string_lossy().to_string()),
    })
}

/// 解析 Markdown 文件的 frontmatter
fn parse_frontmatter(content: &str) -> Result<(SkillFrontmatter, &str)> {
    // frontmatter 格式：
    // ---
    // name: xxx
    // description: xxx
    // ---
    // 正文内容

    let content = content.trim_start();

    if !content.starts_with("---") {
        anyhow::bail!("文件不以 --- 开头");
    }

    let after_start = &content[3..];
    let end_pos = after_start
        .find("---")
        .context("找不到 frontmatter 结束标记 ---")?;

    let frontmatter_str = &after_start[..end_pos];
    let body = &after_start[end_pos + 3..];

    // 解析 YAML frontmatter
    let frontmatter: SkillFrontmatter =
        serde_yaml::from_str(frontmatter_str).context("解析 YAML frontmatter 失败")?;

    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_skill_file() {
        let dir = TempDir::new().unwrap();
        let skill_file = dir.path().join("test.md");

        let content = r#"---
name: test-skill
description: A test skill
---

This is the skill content.
It can have multiple lines.
"#;

        tokio::fs::write(&skill_file, content).await.unwrap();

        let skill = load_skill_file(&skill_file).await.unwrap();

        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
        assert!(skill.content.contains("This is the skill content."));
    }

    #[tokio::test]
    async fn test_load_skills_directory() {
        let dir = TempDir::new().unwrap();

        // 创建多个技能文件
        let skill1 = dir.path().join("skill1.md");
        tokio::fs::write(
            &skill1,
            "---\nname: skill1\ndescription: First skill\n---\nContent 1",
        )
        .await
        .unwrap();

        let skill2 = dir.path().join("skill2.md");
        tokio::fs::write(
            &skill2,
            "---\nname: skill2\ndescription: Second skill\n---\nContent 2",
        )
        .await
        .unwrap();

        // 创建一个非 .md 文件（应该被忽略）
        let other = dir.path().join("other.txt");
        tokio::fs::write(&other, "not a skill").await.unwrap();

        let skills = load_skills(dir.path()).await.unwrap();

        assert_eq!(skills.len(), 2);
        assert!(skills.iter().any(|s| s.name == "skill1"));
        assert!(skills.iter().any(|s| s.name == "skill2"));
    }
}

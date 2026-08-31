use std::path::Path;

/// 模型家族
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    Claude,
    Gpt,
    Gemini,
    Qwen,
    Unknown,
}

/// 检测模型家族
pub fn detect_model_family(model: &str) -> ModelFamily {
    let model_lower = model.to_lowercase();
    if model_lower.contains("claude") {
        ModelFamily::Claude
    } else if model_lower.contains("gpt")
        || model_lower.contains("o1")
        || model_lower.contains("o3")
    {
        ModelFamily::Gpt
    } else if model_lower.contains("gemini") {
        ModelFamily::Gemini
    } else if model_lower.contains("qwen") {
        ModelFamily::Qwen
    } else {
        ModelFamily::Unknown
    }
}

/// 获取模型家族的人格模板
fn get_personality_template(family: ModelFamily) -> &'static str {
    match family {
        ModelFamily::Claude => include_str!("templates/claude.txt"),
        ModelFamily::Gpt => include_str!("templates/gpt.txt"),
        _ => include_str!("templates/default.txt"),
    }
}

/// 从运行时工作目录收集 AGENTS.md
///
/// 搜索顺序：
/// 1. 当前工作目录 (cwd)
/// 2. 向上逐级搜索父目录，直到文件系统根目录
///
/// 返回顺序：从最顶层（根目录附近）到最底层（当前目录）
/// 这样项目级指令会覆盖全局指令
pub fn collect_agents_md(cwd: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut current = cwd.to_path_buf();

    // 收集从当前目录到根目录的所有 AGENTS.md
    loop {
        let agents_path = current.join("AGENTS.md");
        if agents_path.exists()
            && let Ok(content) = std::fs::read_to_string(&agents_path)
        {
            files.push((current.clone(), content));
        }

        // 向上一级
        if !current.pop() {
            break;
        }
    }

    // 反转顺序：从最顶层到最底层
    files.reverse();

    // 返回内容，并附上来源路径用于调试
    files.into_iter().map(|(_, content)| content).collect()
}

/// 构建环境信息
fn build_env_info(cwd: &Path, model: &str) -> String {
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 检测 shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());

    // 检测是否在 git 仓库
    let is_git = cwd.join(".git").exists();

    let mut info = format!(
        "<environment>
模型: {model}
工作目录: {}
平台: {platform} ({arch})
日期: {date}
Shell: {shell}
Git 仓库: {}",
        cwd.display(),
        if is_git { "是" } else { "否" }
    );

    // 如果有 AGENTS.md，添加说明
    let agents_files = collect_agents_md(cwd);
    if !agents_files.is_empty() {
        info.push_str(&format!(
            "\n已加载 AGENTS.md: {} 个文件",
            agents_files.len()
        ));
    }

    info.push_str("\n</environment>");
    info
}

/// 构建完整的系统提示
pub fn build_system_prompt(model: &str, cwd: &Path, skill_list: Option<&str>) -> String {
    let family = detect_model_family(model);
    let personality = get_personality_template(family);
    let env_info = build_env_info(cwd, model);
    let agents_instructions = collect_agents_md(cwd);

    let mut prompt = String::new();

    // 1. 人格模板
    prompt.push_str(personality);
    prompt.push_str("\n\n");

    // 2. 环境信息
    prompt.push_str(&env_info);
    prompt.push_str("\n\n");

    // 3. AGENTS.md 指令
    if !agents_instructions.is_empty() {
        prompt.push_str("<project_instructions>\n");
        for (i, content) in agents_instructions.iter().enumerate() {
            if i > 0 {
                prompt.push_str("\n---\n");
            }
            prompt.push_str(content);
        }
        prompt.push_str("\n</project_instructions>");
        prompt.push_str("\n\n");
    }

    // 4. 技能列表
    if let Some(skills) = skill_list {
        if !skills.is_empty() {
            prompt.push_str(skills);
            prompt.push_str("\n\n");
        }
    }

    // 5. 工具使用说明
    prompt.push_str(
        "<tools>\n\
        你可以使用以下工具来完成任务：\n\
        - bash: 执行 shell 命令\n\
        - read: 读取文件内容（带行号）\n\
        - write: 写入文件（覆盖）\n\
        - skill: 读取技能内容（使用 /技能名 调用技能）\n\
        \n\
        使用工具时，先思考再行动。对于破坏性操作（如删除文件、覆盖重要内容），请格外谨慎。\n\
        </tools>",
    );

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_model_family() {
        assert_eq!(detect_model_family("claude-3-opus"), ModelFamily::Claude);
        assert_eq!(detect_model_family("gpt-4-turbo"), ModelFamily::Gpt);
        assert_eq!(detect_model_family("gemini-pro"), ModelFamily::Gemini);
        assert_eq!(detect_model_family("qwen-72b"), ModelFamily::Qwen);
        assert_eq!(detect_model_family("unknown-model"), ModelFamily::Unknown);
    }
}

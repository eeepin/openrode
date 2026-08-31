//! 技能工具
//!
//! 提供一个工具让 LLM 可以按需读取技能内容。

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use super::SkillRegistry;
use crate::tool::{AgentTool, ToolResult};

/// 技能读取工具
pub struct SkillTool {
    registry: SkillRegistry,
}

impl SkillTool {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }
}

#[derive(Deserialize)]
struct SkillInput {
    name: String,
}

const DESCRIPTION: &str = include_str!("prompts/skill.txt");

#[async_trait]
impl AgentTool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "技能名称"
                }
            },
            "required": ["name"]
        })
    }

    async fn run(&self, input: Value) -> ToolResult {
        let SkillInput { name } = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => return ToolResult::error(format!("参数错误: {e}")),
        };

        let skills = self.registry.read().await;

        match skills.iter().find(|s| s.name == name) {
            Some(skill) => ToolResult::success(format!(
                "# {}\n\n{}\n\n---\n\n{}",
                skill.name, skill.description, skill.content
            )),
            None => {
                let available: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
                ToolResult::error(format!(
                    "技能 '{}' 不存在。可用技能: {}",
                    name,
                    if available.is_empty() {
                        "无".to_string()
                    } else {
                        available.join(", ")
                    }
                ))
            }
        }
    }
}

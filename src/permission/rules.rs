//! 规则管理 — 加载、保存、合并权限规则

use super::Rule;
use anyhow::Result;
use std::path::Path;

/// 权限配置文件格式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[allow(dead_code)]
pub struct PermissionConfig {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[allow(dead_code)]
impl PermissionConfig {
    /// 从文件加载配置
    pub async fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// 合并另一个配置（后者的规则优先级更高）
    pub fn merge(&mut self, other: &PermissionConfig) {
        self.rules.extend(other.rules.clone());
    }
}

/// 获取全局配置路径
#[allow(dead_code)]
pub fn global_config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".openrode").join("permissions.json"))
}

/// 获取项目配置路径
#[allow(dead_code)]
pub fn project_config_path(project_dir: &Path) -> std::path::PathBuf {
    project_dir.join(".openrode").join("permissions.json")
}

/// 加载并合并所有配置
#[allow(dead_code)]
pub async fn load_all_rules(project_dir: &Path) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();

    // 1. 加载全局配置
    if let Some(global_path) = global_config_path()
        && let Ok(config) = PermissionConfig::load(&global_path).await
    {
        rules.extend(config.rules);
    }

    // 2. 加载项目配置
    let project_path = project_config_path(project_dir);
    if let Ok(config) = PermissionConfig::load(&project_path).await {
        rules.extend(config.rules);
    }

    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::Action;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("permissions.json");

        let mut config = PermissionConfig::default();
        config.add_rule(Rule::allow("bash", "git *"));
        config.add_rule(Rule::deny("bash", "rm -rf *"));

        config.save(&path).await.unwrap();

        let loaded = PermissionConfig::load(&path).await.unwrap();
        assert_eq!(loaded.rules.len(), 2);
        assert_eq!(loaded.rules[0].action, Action::Allow);
        assert_eq!(loaded.rules[1].action, Action::Deny);
    }

    #[test]
    fn test_config_merge() {
        let mut base = PermissionConfig::default();
        base.add_rule(Rule::allow("bash", "ls"));

        let mut overlay = PermissionConfig::default();
        overlay.add_rule(Rule::deny("bash", "rm *"));

        base.merge(&overlay);
        assert_eq!(base.rules.len(), 2);
    }
}

//! 权限系统 — 控制工具执行的安全性
//!
//! 权限检查流程：
//! 1. 静态安全检查（直接拒绝危险操作）
//! 2. 规则评估（Allow/Deny/Ask）
//! 3. 如果需要 Ask，询问用户
//! 4. 记忆用户选择（会话内）

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod rules;
pub mod safety;

/// 权限动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// 允许执行
    Allow,
    /// 需要询问用户
    Ask,
    /// 拒绝执行
    Deny,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Allow => write!(f, "allow"),
            Action::Ask => write!(f, "ask"),
            Action::Deny => write!(f, "deny"),
        }
    }
}

/// 权限规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// 权限类型: "bash", "read", "write", "*"
    pub permission: String,
    /// 匹配模式: 通配符，如 "git *", "*.env"
    pub pattern: String,
    /// 动作
    pub action: Action,
}

impl Rule {
    pub fn new(permission: impl Into<String>, pattern: impl Into<String>, action: Action) -> Self {
        Self {
            permission: permission.into(),
            pattern: pattern.into(),
            action,
        }
    }

    pub fn allow(permission: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(permission, pattern, Action::Allow)
    }

    #[allow(dead_code)]
    pub fn deny(permission: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(permission, pattern, Action::Deny)
    }

    #[allow(dead_code)]
    pub fn ask(permission: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(permission, pattern, Action::Ask)
    }
}

/// 权限请求
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// 工具名称
    pub tool: String,
    /// 操作描述（用于显示和匹配）
    pub operation: String,
    /// 详细参数
    pub details: HashMap<String, String>,
}

impl PermissionRequest {
    pub fn new(tool: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            operation: operation.into(),
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// 权限决策结果
#[derive(Debug, Clone)]
pub enum PermissionDecision {
    /// 允许执行
    Allowed,
    /// 拒绝执行（带原因）
    Denied(String),
    /// 已询问用户并获得许可
    Asked,
}

/// 简单通配符匹配
///
/// 支持:
/// - `*` 匹配任意字符序列
/// - `?` 匹配单个字符
/// - 其他字符精确匹配
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    fn match_recursive(p: &[char], t: &[char]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }

        match p[0] {
            '*' => {
                // * 可以匹配空序列或任意序列
                match_recursive(&p[1..], t) || (!t.is_empty() && match_recursive(p, &t[1..]))
            }
            '?' => !t.is_empty() && match_recursive(&p[1..], &t[1..]),
            c => !t.is_empty() && t[0] == c && match_recursive(&p[1..], &t[1..]),
        }
    }

    match_recursive(&pattern_chars, &text_chars)
}

/// 评估规则列表
///
/// 规则从后往前匹配，第一个命中的规则决定结果
/// 如果没有命中任何规则，默认返回 Ask
pub fn evaluate(request: &PermissionRequest, rules: &[Rule]) -> Action {
    rules
        .iter()
        .rev()
        .find(|r| {
            wildcard_match(&r.permission, &request.tool)
                && wildcard_match(&r.pattern, &request.operation)
        })
        .map(|r| r.action)
        .unwrap_or(Action::Ask)
}

/// 权限管理器
pub struct PermissionManager {
    /// 全局规则
    global_rules: Vec<Rule>,
    /// 项目规则
    project_rules: Vec<Rule>,
    /// 会话内记忆（已允许的请求）
    session_memory: Arc<RwLock<HashMap<String, Action>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            global_rules: Self::default_global_rules(),
            project_rules: Vec::new(),
            session_memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn with_project_rules(mut self, rules: Vec<Rule>) -> Self {
        self.project_rules = rules;
        self
    }

    /// 默认全局规则
    /// 规则顺序：先写默认规则，后写具体覆盖（后写优先）
    fn default_global_rules() -> Vec<Rule> {
        vec![
            // 默认: 其他操作需要询问（最先写，优先级最低）
            Rule::ask("*", "*"),
            // 读取任何文件允许
            Rule::allow("read", "*"),
            // 只读命令允许
            Rule::allow("bash", "date"),
            Rule::allow("bash", "whoami"),
            Rule::allow("bash", "pwd"),
            Rule::allow("bash", "echo *"),
            Rule::allow("bash", "grep *"),
            Rule::allow("bash", "find *"),
            Rule::allow("bash", "wc *"),
            Rule::allow("bash", "tail *"),
            Rule::allow("bash", "head *"),
            Rule::allow("bash", "cat *"),
            Rule::allow("bash", "ls*"),
            // Git 命令允许（最后写，优先级最高）
            Rule::allow("bash", "git"),
            Rule::allow("bash", "git *"),
        ]
    }

    /// 加载项目级权限配置
    pub async fn load_project_config(&mut self, project_dir: &Path) -> Result<()> {
        let config_path = project_dir.join(".openrode").join("permissions.json");
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            self.project_rules = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// 检查权限
    pub async fn check(&self, request: &PermissionRequest) -> Result<PermissionDecision> {
        // 1. 静态安全检查
        if let Some(reason) = safety::check_safety(request) {
            return Ok(PermissionDecision::Denied(reason));
        }

        // 2. 检查会话记忆
        let memory_key = format!("{}:{}", request.tool, request.operation);
        {
            let memory = self.session_memory.read().await;
            if let Some(action) = memory.get(&memory_key) {
                return Ok(match action {
                    Action::Allow => PermissionDecision::Allowed,
                    Action::Deny => PermissionDecision::Denied("用户已拒绝".to_string()),
                    Action::Ask => unreachable!(),
                });
            }
        }

        // 3. 合并规则评估（项目规则优先于全局规则）
        let mut all_rules = self.global_rules.clone();
        all_rules.extend(self.project_rules.clone());
        let action = evaluate(request, &all_rules);

        match action {
            Action::Allow => Ok(PermissionDecision::Allowed),
            Action::Deny => Ok(PermissionDecision::Denied("规则拒绝".to_string())),
            Action::Ask => {
                // 询问用户
                let allowed = self.ask_user(request).await?;
                if allowed {
                    // 记忆用户选择
                    let mut memory = self.session_memory.write().await;
                    memory.insert(memory_key, Action::Allow);
                    Ok(PermissionDecision::Asked)
                } else {
                    Ok(PermissionDecision::Denied("用户拒绝".to_string()))
                }
            }
        }
    }

    /// 询问用户是否允许
    async fn ask_user(&self, request: &PermissionRequest) -> Result<bool> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let stdout = tokio::io::stdout();
        let mut stdout = stdout;
        let stdin = tokio::io::stdin();
        let mut stdin = tokio::io::BufReader::new(stdin);

        let mut output = String::new();
        output.push('\n');
        output.push_str("⚠️  需要权限确认\n");
        output.push_str(&format!("工具: {}\n", request.tool));
        output.push_str(&format!("操作: {}\n", request.operation));
        for (key, value) in &request.details {
            output.push_str(&format!("  {key}: {value}\n"));
        }
        output.push_str("允许执行? [y/n] ");

        stdout.write_all(output.as_bytes()).await?;
        stdout.flush().await?;

        let mut input = String::new();
        stdin.read_line(&mut input).await?;
        let input = input.trim().to_lowercase();

        Ok(input == "y" || input == "yes")
    }

    /// 获取所有规则的摘要
    #[allow(dead_code)]
    pub fn rules_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("全局规则: {} 条\n", self.global_rules.len()));
        summary.push_str(&format!("项目规则: {} 条\n", self.project_rules.len()));
        summary
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_match() {
        assert!(wildcard_match("*", "anything"));
        assert!(wildcard_match("git *", "git status"));
        assert!(wildcard_match("git *", "git commit -m"));
        assert!(!wildcard_match("git *", "hg status"));
        assert!(wildcard_match("*.txt", "file.txt"));
        assert!(!wildcard_match("*.txt", "file.rs"));
        assert!(wildcard_match("ls*", "ls"));
        assert!(wildcard_match("ls*", "ls -la"));
    }

    #[test]
    fn test_evaluate_rules() {
        // 规则顺序：先写默认规则，后写具体覆盖
        let rules = vec![
            Rule::ask("*", "*"),          // 默认：询问
            Rule::allow("bash", "git *"), // 覆盖：git 命令允许
            Rule::deny("bash", "rm *"),   // 覆盖：rm 命令拒绝
        ];

        let git_req = PermissionRequest::new("bash", "git status");
        assert_eq!(evaluate(&git_req, &rules), Action::Allow);

        let rm_req = PermissionRequest::new("bash", "rm file.txt");
        assert_eq!(evaluate(&rm_req, &rules), Action::Deny);

        let other_req = PermissionRequest::new("bash", "make build");
        assert_eq!(evaluate(&other_req, &rules), Action::Ask);
    }

    #[test]
    fn test_rule_priority() {
        // 后面的规则覆盖前面的
        let rules = vec![
            Rule::deny("bash", "rm *"),
            Rule::allow("bash", "rm -i *"), // 交互式 rm 允许
        ];

        let req = PermissionRequest::new("bash", "rm -i file.txt");
        assert_eq!(evaluate(&req, &rules), Action::Allow);

        let req2 = PermissionRequest::new("bash", "rm file.txt");
        assert_eq!(evaluate(&req2, &rules), Action::Deny);
    }
}

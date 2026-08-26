//! 静态安全检查 — 直接拒绝明显危险的操作
//!
//! 这些检查在规则评估之前执行，无论规则如何配置都会拒绝

use super::PermissionRequest;
use std::path::Path;

/// 检查操作是否安全
///
/// 返回 Some(reason) 表示不安全，None 表示通过静态检查
pub fn check_safety(request: &PermissionRequest) -> Option<String> {
    match request.tool.as_str() {
        "bash" => check_bash_safety(&request.operation),
        "write" => check_write_safety(request),
        _ => None,
    }
}

/// 检查 bash 命令安全性
fn check_bash_safety(command: &str) -> Option<String> {
    let cmd = command.trim();
    let cmd_lower = cmd.to_lowercase();

    // 检查危险模式
    let dangerous_patterns = [
        // 删除根目录或关键目录
        ("rm -rf /", "禁止删除根目录"),
        ("rm -rf /*", "禁止删除根目录下所有文件"),
        ("rm -rf ~", "禁止删除 home 目录"),
        ("rm -rf ~/", "禁止删除 home 目录"),
        ("rm -rf $HOME", "禁止删除 home 目录"),
        // 格式化磁盘
        ("mkfs", "禁止格式化磁盘"),
        ("dd if=", "禁止使用 dd 覆盖设备"),
        // 危险的重定向
        ("> /dev/sda", "禁止写入磁盘设备"),
        ("> /dev/nvme", "禁止写入 NVMe 设备"),
        // 递归权限修改
        ("chmod -r 777 /", "禁止递归修改根目录权限"),
        ("chown -r", "禁止递归修改所有者"),
    ];

    for (pattern, reason) in dangerous_patterns {
        if cmd_lower.contains(pattern) {
            return Some(reason.to_string());
        }
    }

    // 检查是否针对敏感目录的删除
    let sensitive_dirs = [
        "/etc", "/usr", "/bin", "/sbin", "/boot", "/lib", "/var", "/proc", "/sys",
    ];

    if cmd_lower.contains("rm -rf") || cmd_lower.contains("rm -fr") {
        for dir in sensitive_dirs {
            if cmd.contains(dir) {
                return Some(format!("禁止删除系统目录: {}", dir));
            }
        }
    }

    // 检查是否写入敏感位置
    if is_writing_to_sensitive_location(cmd) {
        return Some("禁止写入系统关键位置".to_string());
    }

    None
}

/// 检查 write 工具的安全性
fn check_write_safety(request: &PermissionRequest) -> Option<String> {
    let path = request.details.get("path")?;
    let path = Path::new(path);

    // 检查是否写入敏感目录
    let sensitive_paths = [
        "/etc/", "/usr/", "/bin/", "/sbin/", "/boot/", "/lib/", "/proc/", "/sys/", "/dev/",
    ];

    let path_str = path.to_string_lossy();
    for sensitive in sensitive_paths {
        if path_str.starts_with(sensitive) {
            return Some(format!("禁止写入系统目录: {}", sensitive));
        }
    }

    // 检查是否写入 SSH 配置
    if path_str.contains(".ssh/") {
        return Some("禁止修改 SSH 配置".to_string());
    }

    // 检查是否写入 shell 配置
    let shell_configs = [
        ".bashrc",
        ".bash_profile",
        ".zshrc",
        ".profile",
        ".zprofile",
    ];
    if let Some(filename) = path.file_name().and_then(|f| f.to_str())
        && shell_configs.contains(&filename)
    {
        return Some(format!("禁止修改 shell 配置: {}", filename));
    }

    None
}

/// 检查命令是否写入敏感位置
fn is_writing_to_sensitive_location(cmd: &str) -> bool {
    // 简单的启发式检查
    let sensitive = [
        "/etc/", "/usr/", "/bin/", "/sbin/", ".ssh/", ".bashrc", ".zshrc",
    ];

    // 检查是否有重定向到敏感位置
    for target in sensitive {
        if cmd.contains(&format!("> {}", target)) || cmd.contains(&format!(">> {}", target)) {
            return true;
        }
        // tee 命令
        if cmd.contains("tee ") && cmd.contains(target) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_commands_blocked() {
        assert!(check_bash_safety("rm -rf /").is_some());
        assert!(check_bash_safety("rm -rf /*").is_some());
        assert!(check_bash_safety("rm -rf /etc").is_some());
        assert!(check_bash_safety("mkfs.ext4 /dev/sda1").is_some());
    }

    #[test]
    fn test_safe_commands_allowed() {
        assert!(check_bash_safety("ls -la").is_none());
        assert!(check_bash_safety("git status").is_none());
        assert!(check_bash_safety("cat file.txt").is_none());
        assert!(check_bash_safety("rm file.txt").is_none()); // 单个文件删除是允许的
    }

    #[test]
    fn test_write_safety() {
        let req = PermissionRequest::new("write", "写文件").with_detail("path", "/etc/passwd");
        assert!(check_write_safety(&req).is_some());

        let req2 =
            PermissionRequest::new("write", "写文件").with_detail("path", "/home/user/file.txt");
        assert!(check_write_safety(&req2).is_none());

        let req3 =
            PermissionRequest::new("write", "写文件").with_detail("path", "/home/user/.ssh/config");
        assert!(check_write_safety(&req3).is_some());
    }
}

//! Git 快照系统
//!
//! 在每次工具执行前对工作目录打快照，支持撤销操作。
//! 快照存储在 `~/.openrode/snapshots/<session_id>/` 目录下。

use anyhow::{Context, Result};
use git2::{IndexAddOption, Repository, Signature};
use std::path::{Path, PathBuf};

/// Git 快照管理器
pub struct Snapshot {
    /// 快照仓库路径
    repo_path: PathBuf,
    /// Git 仓库
    repo: Repository,
    /// 工作目录（被监控的目录）
    workdir: PathBuf,
}

impl Snapshot {
    /// 创建新的快照管理器
    pub fn new(session_id: &str, workdir: &Path) -> Result<Self> {
        let repo_path = dirs::home_dir()
            .context("无法获取 home 目录")?
            .join(".openrode")
            .join("snapshots")
            .join(session_id);

        std::fs::create_dir_all(&repo_path)
            .with_context(|| format!("创建快照目录失败: {}", repo_path.display()))?;

        let repo = if repo_path.join(".git").exists() {
            Repository::open(&repo_path)?
        } else {
            Repository::init(&repo_path)?
        };

        Ok(Self {
            repo_path,
            repo,
            workdir: workdir.to_path_buf(),
        })
    }

    /// 在工具执行前打快照
    pub fn before_tool_execution(&self, step_name: &str) -> Result<()> {
        // 同步工作目录到快照仓库
        self.sync_workdir()?;

        let mut index = self.repo.index()?;

        // 添加所有变更
        index.add_all(
            ["*"].iter(),
            IndexAddOption::DEFAULT,
            Some(&mut |path, _| {
                // 排除 .openrode 目录自身
                if path.starts_with(".openrode") { 1 } else { 0 }
            }),
        )?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let sig = Signature::now("openrode", "agent@openrode")?;

        // 创建 commit
        let message = format!("before: {}", step_name);
        match self.repo.head() {
            Ok(head) => {
                let parent = head.peel_to_commit()?;
                self.repo
                    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent])?;
            }
            Err(_) => {
                // 首次 commit
                self.repo
                    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])?;
            }
        }

        Ok(())
    }

    /// 同步工作目录到快照仓库
    fn sync_workdir(&self) -> Result<()> {
        let snapshot_workdir = self.repo_path.join("workdir");

        // 如果 workdir 不存在，创建它
        if !snapshot_workdir.exists() {
            std::fs::create_dir_all(&snapshot_workdir)?;
        }

        // 使用 rsync 或类似方式同步（这里简化为复制）
        // 实际生产环境应该用更高效的方式
        self.copy_dir_recursive(&self.workdir, &snapshot_workdir)?;

        Ok(())
    }

    /// 递归复制目录
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if !dst.exists() {
            std::fs::create_dir_all(dst)?;
        }

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            // 跳过 .git 和 .openrode 目录
            if let Some(name) = src_path.file_name().and_then(|n| n.to_str())
                && (name == ".git" || name == ".openrode" || name == "target")
            {
                continue;
            }

            if src_path.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// 获取快照列表
    #[allow(dead_code)]
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut snapshots = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            snapshots.push(SnapshotInfo {
                id: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
            });
        }

        Ok(snapshots)
    }

    /// 恢复到指定快照
    #[allow(dead_code)]
    pub fn restore(&self, commit_id: &str) -> Result<()> {
        let oid = git2::Oid::from_str(commit_id).context("无效的 commit ID")?;
        let commit = self.repo.find_commit(oid)?;
        let tree = commit.tree()?;

        //  checkout tree 到 workdir
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();

        self.repo
            .checkout_tree(tree.as_object(), Some(&mut checkout_builder))?;

        // 更新 HEAD
        self.repo.set_head_detached(oid)?;

        // 同步回原工作目录
        let snapshot_workdir = self.repo_path.join("workdir");
        self.copy_dir_recursive(&snapshot_workdir, &self.workdir)?;

        Ok(())
    }

    /// 撤销最近 N 步
    #[allow(dead_code)]
    pub fn undo_steps(&self, n: usize) -> Result<()> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let mut current = None;
        for (i, oid) in revwalk.enumerate() {
            if i == n {
                break;
            }
            current = Some(oid?);
        }

        if let Some(target_oid) = current {
            self.restore(&target_oid.to_string())?;
        }

        Ok(())
    }
}

/// 快照信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SnapshotInfo {
    pub id: String,
    pub message: String,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_creation() {
        let dir = tempdir().unwrap();
        let workdir = dir.path();

        // 创建一个测试文件
        std::fs::write(workdir.join("test.txt"), "hello").unwrap();

        let snapshot = Snapshot::new("test-session", workdir).unwrap();
        snapshot.before_tool_execution("test-step").unwrap();

        let snapshots = snapshot.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots[0].message.contains("test-step"));
    }
}

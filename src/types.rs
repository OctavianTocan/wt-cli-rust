use serde_json::de;

/// Represents information about a Git branch, including its name, whether it is a remote branch, and whether it is the current branch.
#[derive(Debug, Clone)]
pub struct GitBranch {
    pub name: String,
    pub is_remote: bool,
    pub current: bool,
}

/// Represents the configuration for a setup process, including installation steps and file copy operations.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
}

/// Represents the status of a Git repository, including whether it has uncommitted changes, how many commits it is ahead or behind its upstream branch, and how many uncommitted files it has.
#[derive(Debug, Clone)]
pub struct GitStatus {
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub uncomitted_files: usize,
    pub upstream: Option<String>,
}

/// Represents information about a Git worktree, including its path, current branch, latest commit, status, last modification time, and whether it is the main branch or the current worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub commit: CommitInfo,
    pub status: GitStatus,
    pub last_modified: String,
    pub is_main: bool,
    pub is_current: bool,
}

/// Represents the source of files to be copied in a setup step, which can be either a single file or multiple files.
impl WorktreeInfo {
    /// Determines if the worktree is clean, meaning it has no uncommitted changes and is not ahead or behind its upstream branch.
    pub fn is_clean(&self) -> bool {
        !self.status.is_dirty && self.status.ahead == 0 && self.status.behind == 0
    }
}

/// Represents different kinds of issues that can be detected during the setup process, such as uncommitted changes, being ahead or behind the upstream branch, stale files, orphaned files, or verification failures.
#[derive(Debug, Clone)]
pub enum IssueKind {
    Dirty,
    Ahead,
    Behind,
    Stale,
    Orphaned,
    Verification,
}

/// Provides a string representation for each kind of issue, which can be used for display purposes or in logs.
impl IssueKind {
    /// Returns a string representation of the issue kind, which can be used for display purposes or in logs.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dirty => "dirty",
            Self::Ahead => "ahead",
            Self::Behind => "behind",
            Self::Stale => "stale",
            Self::Orphaned => "orphaned",
            Self::Verification => "verification",
        }
    }
}

/// Represents an issue detected during the setup process, including its kind and a descriptive message.
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub kind: IssueKind,
    pub message: String,
}

/// Represents a health report for a Git worktree, including the worktree information, any issues detected during the health check, and whether the worktree is considered healthy.
#[derive(Debug, Clone)]
pub struct WorktreeHealthReport {
    pub worktree: WorktreeInfo,
    pub issues: Vec<HealthIssue>,
    pub is_healthy: bool,
}

/// Represents the source of files to be copied in a setup step.
#[derive(Debug, Clone)]
pub enum CopySource {
    Single(String),
    Multiple(Vec<String>),
}

/// Represents a step in the setup process, which can be either an installation or a file copy operation.
#[derive(Debug, Clone)]
pub enum SetupStep {
    Install {
        command: Option<String>,
        optional: Option<bool>,
    },
    Copy {
        from: CopySource,
        to: String,
        exclude: Option<Vec<String>>,
        optional: Option<bool>,
    },
    Run {
        command: String,
        optional: Option<bool>,
    },
    Verify {
        path: String,
        label: Option<String>,
        optional: Option<bool>,
    },
}

/// Represents the overall configuration for the setup process, including the steps to be executed and any lifecycle scripts that should be run at different stages of the process.
#[derive(Debug, Clone)]
pub struct SetupConfig {
    pub steps: Vec<SetupStep>,
}

/// Represents lifecycle scripts that can be executed at different stages of the setup process, such as before or after the main setup steps.
#[derive(Debug, Clone)]
pub struct LifecycleScripts {
    pub postsetup: Option<String>,
    pub preclean: Option<String>,
}

/// Represents lifecycle scripts that can be executed at different stages of the setup process, such as before or after the main setup steps.
#[derive(Debug, Clone)]
pub struct WtConfig {
    pub worktree_dir: String,
    pub main_branch: String,
    pub dev_branch: String,
    pub default_base: String,
    pub remote: String,
    pub auto_setup: bool,
    pub stale_days: u32,
    pub setup: SetupConfig,
    pub lifecycle_scripts: LifecycleScripts,
}

/// Represents a loaded configuration, including the configuration data itself, the source from which it was loaded, and the root path of the configuration.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: WtConfig,
    pub source: String,
    pub root_path: String,
}

/// Represents lifecycle scripts that can be executed at different stages of the setup process, such as before or after the main setup steps.
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub cwd: String,
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_info_fields_are_accessible() {
        // Create a CommitInfo with sample values
        // Assert each field matches what you set
    }

    #[test]
    fn git_status_optional_upstream_is_none() {
        // Create a GitStatus with upstream = None
        // Assert upstream.is_none()
    }

    #[test]
    fn git_status_optional_upstream_is_some() {
        // Create a GitStatus with upstream = Some("origin/main")
        // Assert upstream.unwrap() == "origin/main"
    }

    #[test]
    fn worktree_is_clean_when_no_issues() {
        // Create a WorktreeInfo with is_dirty=false, ahead=0, behind=0
        // Assert is_clean() returns true
    }

    #[test]
    fn worktree_is_not_clean_when_dirty() {
        // Create a WorktreeInfo with is_dirty=true
        // Assert is_clean() returns false
    }

    #[test]
    fn setup_step_enum_variants() {
        // Create one of each SetupStep variant
        // Use match to verify each one has the right type
    }

    #[test]
    fn issue_kind_as_str() {
        // Create each IssueKind variant
        // Assert as_str() returns the expected string for each
    }

    #[test]
    fn copy_source_enum() {
        // Create CopySource::Single("file.txt") and CopySource::Multiple(vec![])
        // Use match to verify the inner values
    }
}

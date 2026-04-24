// ============================================================================
// Git primitives
// ============================================================================

/// A Git branch, local or remote, and whether it is currently checked out.
#[derive(Debug, Clone)]
pub struct GitBranch {
    pub name: String,
    pub is_remote: bool,
    pub current: bool,
}

/// A single commit's identifying metadata.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
}

/// Working-tree and upstream-tracking status for a repository.
#[derive(Debug, Clone)]
pub struct GitStatus {
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub uncomitted_files: usize,
    pub upstream: Option<String>,
}

// ============================================================================
// Worktree model
// ============================================================================

/// A single Git worktree with its branch, latest commit, and status snapshot.
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

impl WorktreeInfo {
    /// True when the worktree has no uncommitted changes and is level with its upstream.
    pub fn is_clean(&self) -> bool {
        !self.status.is_dirty && self.status.ahead == 0 && self.status.behind == 0
    }
}

// ============================================================================
// Health checks
// ============================================================================

/// Category of problem detected by a worktree health check.
#[derive(Debug, Clone)]
pub enum IssueKind {
    Dirty,
    Ahead,
    Behind,
    Stale,
    Orphaned,
    Verification,
}

impl IssueKind {
    /// Stable lowercase label for display and logs.
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

/// A single issue surfaced by a health check, with a human-readable message.
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub kind: IssueKind,
    pub message: String,
}

/// Aggregate health result for a worktree: the worktree plus every issue found.
#[derive(Debug, Clone)]
pub struct WorktreeHealthReport {
    pub worktree: WorktreeInfo,
    pub issues: Vec<HealthIssue>,
    pub is_healthy: bool,
}

// ============================================================================
// Setup configuration
// ============================================================================

/// Source paths for a `Copy` setup step — either one file or many.
#[derive(Debug, Clone)]
pub enum CopySource {
    Single(String),
    Multiple(Vec<String>),
}

/// One action in the setup pipeline run after a worktree is created.
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

/// Ordered list of setup steps to execute.
#[derive(Debug, Clone)]
pub struct SetupConfig {
    pub steps: Vec<SetupStep>,
}

/// Hooks that run around the setup pipeline.
#[derive(Debug, Clone)]
pub struct LifecycleScripts {
    pub postsetup: Option<String>,
    pub preclean: Option<String>,
}

// ============================================================================
// Top-level config
// ============================================================================

/// Full user configuration: layout, branch policy, and setup/lifecycle scripts.
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

/// A resolved `WtConfig` along with where it was loaded from.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: WtConfig,
    pub source: String,
    pub root_path: String,
}

// ============================================================================
// Runtime context
// ============================================================================

/// Per-invocation context passed into commands (working directory, output mode).
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub cwd: String,
    pub json: bool,
}

// ============================================================================
// Tests
// ============================================================================

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

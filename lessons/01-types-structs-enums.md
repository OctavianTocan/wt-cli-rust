# Step 1: Types — Structs and Enums

## Architecture Context

Open `src/types.ts` in the [original wt-cli](https://github.com/OctavianTocan/wt-cli/blob/main/src/types.ts).

This file defines **every data structure** that flows through the rest of the tool. Every command reads these types. Every core module accepts them as parameters and returns them as results. Understanding these shapes is understanding what wt-cli actually works with.

There are 16 TypeScript interfaces/type aliases here. Some are simple data containers (GitBranch, CommitInfo). Some have optional fields (GitStatus, LoadedConfig). Some represent union types (SetupStep = install | copy | run | verify). Some carry behavior (HealthIssue with its kind discriminant).

**Why this is step 1:** Types have no behavior. No I/O, no side effects, no error handling. Just shape. That makes them the perfect place to learn Rust's data definition syntax without any other complexity. Once these are in place, every subsequent step references them.

---

## Original Code (Before)

Here's the actual TypeScript you're translating. Read through it — you already know some of these from using wt-cli:

```typescript
// src/types.ts — the full file

export type InstallCommand = "auto" | string;

export interface SetupInstallStep {
  type: "install";
  command?: InstallCommand;
  optional?: boolean;
}

export interface SetupCopyStep {
  type: "copy";
  from: string | string[];
  to: string;
  exclude?: string[];
  optional?: boolean;
}

export interface SetupRunStep {
  type: "run";
  command: string;
  optional?: boolean;
}

export interface SetupVerifyStep {
  type: "verify";
  path: string;
  label?: string;
  optional?: boolean;
}

export type SetupStep =
  | SetupInstallStep
  | SetupCopyStep
  | SetupRunStep
  | SetupVerifyStep;

export interface LifecycleScripts {
  postsetup?: string;
  preclean?: string;
}

export interface WtConfig {
  worktreeDir: string;
  mainBranch: string;
  devBranch?: string;
  defaultBase: string;
  remote: string;
  autoSetup: boolean;
  staleDays: number;
  setup: {
    steps: SetupStep[];
  };
  lifecycleScripts: LifecycleScripts;
}

export interface LoadedConfig {
  config: WtConfig;
  source: string;
  rootPath: string;
}

export interface GitBranch {
  name: string;
  isRemote: boolean;
  current: boolean;
}

export interface GitStatus {
  isDirty: boolean;
  ahead: number;
  behind: number;
  uncommittedFiles: number;
  upstream?: string;
}

export interface CommitInfo {
  sha: string;
  shortSha: string;
  message: string;
}

export interface WorktreeInfo {
  path: string;
  branch: string;
  commit: CommitInfo;
  status: GitStatus;
  lastModified: Date;
  isMain: boolean;
  isCurrent: boolean;
}

export interface CommandContext {
  cwd: string;
  json: boolean;
}

export interface HealthIssue {
  kind: "dirty" | "ahead" | "behind" | "stale" | "orphaned" | "verification";
  message: string;
}

export interface WorktreeHealthReport {
  worktree: WorktreeInfo;
  issues: HealthIssue[];
  isHealthy: boolean;
}
```

---

## Translation Walkthrough

Let's walk through the key mappings. The goal isn't to memorize rules — it's to understand *why* each Rust choice was made.

### Mapping 1: Simple interface → struct

**TypeScript:**
```typescript
export interface CommitInfo {
  sha: string;
  shortSha: string;
  message: string;
}
```

**Rust:**
```rust
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
}
```

What changed and why:
- `interface` → `struct`. Same idea: named collection of fields.
- `string` → `String`. In TS, `string` is a primitive. In Rust, `String` is heap-allocated text that you own. (There's also `&str` for borrowed text, but for struct fields that store data, use `String`.)
- `export` → `pub`. Both mean "visible outside this file." In Rust, you also put `pub` on each field — without it, fields are private even if the struct is public.
- camelCase → snake_case. Rust convention. `shortSha` becomes `short_sha`.
- `#[derive(Debug, Clone)]` — Rust doesn't auto-generate the ability to print or copy types. `Debug` gives you `println!("{:?}", commit_info)` (like `JSON.stringify`). `Clone` gives you `.clone()`. You'll add these to every struct.

### Mapping 2: Optional field → Option<T>

**TypeScript:**
```typescript
export interface GitStatus {
  isDirty: boolean;
  ahead: number;
  behind: number;
  uncommittedFiles: number;
  upstream?: string;  // might be undefined
}
```

**Rust:**
```rust
#[derive(Debug, Clone)]
pub struct GitStatus {
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub uncommitted_files: usize,
    pub upstream: Option<String>,  // either Some("origin/main") or None
}
```

What changed and why:
- `upstream?: string` → `upstream: Option<String>`. TS uses `?` to mean "might be undefined." Rust uses `Option<T>` — an enum that's either `Some(value)` or `None`. No null, no undefined. The compiler forces you to handle both cases.
- `number` → `usize`. In TS, all numbers are the same type. In Rust, you pick: `u32` (unsigned 32-bit), `i32` (signed), `usize` (pointer-sized, used for counts/indices). For counts of things (ahead, behind, files), `usize` is idiomatic.
- `boolean` → `bool`. Straightforward rename.

### Mapping 3: String literal union → enum

**TypeScript:**
```typescript
export interface HealthIssue {
  kind: "dirty" | "ahead" | "behind" | "stale" | "orphaned" | "verification";
  message: string;
}
```

**Rust:**
```rust
#[derive(Debug, Clone)]
pub enum IssueKind {
    Dirty,
    Ahead,
    Behind,
    Stale,
    Orphaned,
    Verification,
}

#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub kind: IssueKind,
    pub message: String,
}
```

What changed and why:
- TS uses string literal unions (`"dirty" | "ahead" | ...`) for the kind field. In Rust, that's an `enum`. Each variant is its own value — you can't typo it, and the compiler checks you handle every case.
- We split it into two types: `IssueKind` (the enum) and `HealthIssue` (the struct that uses it). This is because Rust enums are standalone types, not inline union annotations.
- Later you'll add `impl IssueKind { fn as_str(&self) -> &'static str { ... } }` to convert back to strings when needed for display.

### Mapping 4: Discriminated union → enum with variants

**TypeScript:**
```typescript
// Four separate interfaces, each with a `type` discriminant:
export interface SetupInstallStep {
  type: "install";
  command?: InstallCommand;
  optional?: boolean;
}

export interface SetupCopyStep {
  type: "copy";
  from: string | string[];
  to: string;
  exclude?: string[];
  optional?: boolean;
}

// ...then a union of all four:
export type SetupStep =
  | SetupInstallStep
  | SetupCopyStep
  | SetupRunStep
  | SetupVerifyStep;
```

**Rust:**
```rust
pub enum CopySource {
    Single(String),
    Multiple(Vec<String>),
}

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
```

What changed and why:
- In TS, the pattern is: separate interfaces + a `type` field as discriminant + a union type. In Rust, **one enum does all of that.** Each variant is a different shape, and the variant name replaces the `type` field. No separate interfaces needed.
- `string | string[]` → `CopySource` enum. TS lets you union any types inline. Rust wants you to name things — so `string | string[]` becomes its own enum `CopySource` with `Single(String)` and `Multiple(Vec<String>)`.
- `string[]` → `Vec<String>`. Vec is Rust's growable array — same as a JS array, but typed.
- Note: `type` is a **reserved keyword** in Rust (used for type aliases), so you can't use it as a field name. The enum variants handle the discriminant instead.
- We're ignoring serde (JSON serialization) for now. When we get to step 3, you'll add `#[serde(tag = "type")]` to handle the JSON mapping.

### Mapping 5: Nested object → nested struct

**TypeScript:**
```typescript
export interface WtConfig {
  worktreeDir: string;
  mainBranch: string;
  // ...
  setup: {
    steps: SetupStep[];
  };
  lifecycleScripts: LifecycleScripts;
}
```

**Rust:**
```rust
pub struct SetupConfig {
    pub steps: Vec<SetupStep>,
}

pub struct WtConfig {
    pub worktree_dir: String,
    pub main_branch: String,
    pub dev_branch: Option<String>,
    pub default_base: String,
    pub remote: String,
    pub auto_setup: bool,
    pub stale_days: u32,
    pub setup: SetupConfig,
    pub lifecycle_scripts: LifecycleScripts,
}
```

What changed and why:
- TS inline object `setup: { steps: SetupStep[] }` → separate `SetupConfig` struct. Rust doesn't have anonymous object types — every shape needs a name. This is actually cleaner because you can add methods to `SetupConfig` later.
- `SetupStep[]` → `Vec<SetupStep>`. Array syntax changes, but it's the same idea: an ordered list of typed items.

---

## Your Task

**File:** `src/types.rs` (create this file)

**First:** Add `mod types;` to `src/main.rs`:
```rust
mod types;

fn main() {
    println!("wt-cli — Rust rewrite in progress");
}
```

Without `mod types;`, Rust won't know your file exists. This is different from TypeScript/JavaScript — Rust doesn't auto-discover files.

**Then** implement these types in `src/types.rs`. Use `#![allow(dead_code)]` at the top (since nothing uses them yet — warnings are expected). Use `#[derive(Debug, Clone)]` on all structs and enums.

Build them in this order (simplest first):

**Round 1 — Simple structs:**
- `CommitInfo` — fields: `sha` (String), `short_sha` (String), `message` (String)
- `GitBranch` — fields: `name` (String), `is_remote` (bool), `current` (bool)
- `CommandContext` — fields: `cwd` (String), `json` (bool)

**Round 2 — Structs with Option:**
- `GitStatus` — fields: `is_dirty` (bool), `ahead` (usize), `behind` (usize), `uncommitted_files` (usize), `upstream` (Option<String>)
- `LifecycleScripts` — fields: `postsetup` (Option<String>), `preclean` (Option<String>)

**Round 3 — Enums:**
- `IssueKind` — variants: `Dirty`, `Ahead`, `Behind`, `Stale`, `Orphaned`, `Verification`
- `CopySource` — variants: `Single(String)`, `Multiple(Vec<String>)`

**Round 4 — Composite types:**
- `HealthIssue` — fields: `kind` (IssueKind), `message` (String)
- `SetupConfig` — field: `steps` (Vec<SetupStep>)
- `WtConfig` — fields: `worktree_dir` (String), `main_branch` (String), `dev_branch` (Option<String>), `default_base` (String), `remote` (String), `auto_setup` (bool), `stale_days` (u32), `setup` (SetupConfig), `lifecycle_scripts` (LifecycleScripts)
- `LoadedConfig` — fields: `config` (WtConfig), `source` (String), `root_path` (String)
- `SetupStep` — enum with variants: `Install`, `Copy`, `Run`, `Verify` (see walkthrough above for field details)
- `WorktreeInfo` — fields: `path` (String), `branch` (String), `commit` (CommitInfo), `status` (GitStatus), `is_main` (bool), `is_current` (bool). Skip `last_modified` for now — we'll add it in a later step when we cover DateTime.
- `WorktreeHealthReport` — fields: `worktree` (WorktreeInfo), `issues` (Vec<HealthIssue>), `is_healthy` (bool)

Add `impl WorktreeInfo` with:
- `pub fn is_clean(&self) -> bool` — returns true if status is not dirty AND ahead/behind are both 0

Add `impl IssueKind` with:
- `pub fn as_str(&self) -> &'static str` — returns "dirty", "ahead", "behind", "stale", "orphaned", or "verification" (use `match`)

---

## Test Skeleton

```rust
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
```

Place the test module at the bottom of `src/types.rs`. Run `cargo test` to check.

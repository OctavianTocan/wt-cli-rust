# Step 1: Types — Structs and Enums

## Architecture Context

Open `src/types.ts` in the [original wt-cli](https://github.com/OctavianTocan/wt-cli/blob/main/src/types.ts).

This file defines **every data structure** that flows through the rest of the tool. Every command reads these types. Every core module accepts them as parameters and returns them as results. Understanding these shapes is understanding what wt-cli actually works with.

There are 16 TypeScript interfaces here. Some are simple data containers (GitBranch, CommitInfo). Some have optional fields (GitStatus, LoadedConfig). Some represent union types (SetupStep = install | copy | run | verify). Some carry behavior (HealthIssue with its kind discriminant).

In TypeScript, these are all `interface` or `type`. In Rust, they'll be `struct` and `enum`. The mapping is direct — the shape doesn't change, just the syntax and the rules around it.

**Why this is step 1:** Types have no behavior. No I/O, no side effects, no error handling. Just shape. That makes them the perfect place to learn Rust's data definition syntax without any other complexity. Once these are in place, every subsequent step references them.

---

## Rust Concepts

### Concept 1: Struct

A struct in Rust is a named collection of fields. Think of it like a TypeScript interface that can also carry methods.

**TypeScript:**
```typescript
interface CommitInfo {
  sha: string;
  shortSha: string;
  message: string;
}
```

**Rust:**
```rust
struct CommitInfo {
    sha: String,
    short_sha: String,
    message: String,
}
```

Key differences:
- Rust uses `String` (heap-allocated, owned) not `string` (which doesn't exist in Rust)
- Field names use snake_case (Rust convention), not camelCase
- The struct and its fields are private by default — but adding `pub` makes them accessible
- No `?` for optional fields — that's handled by `Option<T>` (covered below)

You can add behavior to structs with `impl` blocks:
```rust
impl CommitInfo {
    fn is_empty(&self) -> bool {
        self.sha.is_empty()
    }
}
```

The `&self` means "borrow this struct read-only." Think of it like `this` in a class method, except Rust makes the borrowing explicit.

**Common gotcha:** You'll see `String` and `&str` everywhere and wonder which to use. Rule of thumb: `String` is for owned text (you own the memory). `&str` is for borrowed text (someone else owns it). For struct fields that store data, use `String`. For function parameters that just read data, use `&str`.

### Concept 2: Enum

Rust enums are way more powerful than TypeScript unions. Each variant can carry data.

**TypeScript:**
```typescript
type SetupStep = SetupInstallStep | SetupCopyStep | SetupRunStep | SetupVerifyStep;
// where each is a separate interface with a `type` discriminant field
```

**Rust:**
```rust
enum SetupStep {
    Install { command: Option<String>, optional: Option<bool> },
    Copy { from: CopySource, to: String, exclude: Option<Vec<String>>, optional: Option<bool> },
    Run { command: String, optional: Option<bool> },
    Verify { path: String, label: Option<String>, optional: Option<bool> },
}
```

One type. Four variants. Each variant can carry different data. No need for separate interfaces and a discriminant field — the variant name IS the discriminant.

You access enum data with `match`:
```rust
match step {
    SetupStep::Install { command, .. } => { /* handle install */ }
    SetupStep::Copy { from, to, .. } => { /* handle copy */ }
    _ => {} // catch-all
}
```

The `..` means "ignore the other fields." We'll cover match in depth in step 6.

**Common gotcha:** Enums in Rust are NOT like C enums (just named integers). They're algebraic data types. Each variant is a completely different shape. This is one of Rust's most powerful features — embrace it.

### Concept 3: Derive Macros

Rust can automatically generate common implementations for your types using `#[derive(...)]`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommitInfo {
    sha: String,
    short_sha: String,
    message: String,
}
```

- **Debug** — lets you print the struct with `{:?}` (like JSON.stringify but for debugging)
- **Clone** — lets you make a deep copy with `.clone()`
- **Serialize / Deserialize** — from the `serde` crate, lets you convert to/from JSON

Without derive, you'd have to implement these by hand. With derive, one annotation and you're done. The TS equivalent is roughly "it just works as a plain object."

### Concept 4: Option<T>

TypeScript uses `field?: type` for optional values. Rust uses `Option<T>`.

```typescript
// TypeScript
interface GitStatus {
  upstream?: string;  // might be undefined
}
```

```rust
// Rust
struct GitStatus {
    upstream: Option<String>,  // either Some("origin/main") or None
}
```

`Option<T>` is an enum with two variants: `Some(value)` and `None`. It's never null/undefined — it's always explicitly one or the other. The compiler forces you to handle both cases.

---

## Your Task

**File:** `src/types.rs` (create this file)

Implement these types. Use `pub` on everything (other modules will import them). Derive `Debug, Clone, Serialize, Deserialize` on all structs and enums.

**Core config types:**
- `WtConfig` — fields: `worktree_dir` (String), `main_branch` (String), `dev_branch` (Option<String>), `default_base` (String), `remote` (String), `auto_setup` (bool), `stale_days` (u32), `setup` (SetupConfig), `lifecycle_scripts` (LifecycleScripts)
- `SetupConfig` — field: `steps` (Vec<SetupStep>)
- `LifecycleScripts` — fields: `postsetup` (Option<String>), `preclean` (Option<String>)
- `LoadedConfig` — fields: `config` (WtConfig), `source` (String), `root_path` (String)

**Git types:**
- `GitBranch` — fields: `name` (String), `is_remote` (bool), `current` (bool)
- `GitStatus` — fields: `is_dirty` (bool), `ahead` (usize), `behind` (usize), `uncommitted_files` (usize), `upstream` (Option<String>)
- `CommitInfo` — fields: `sha` (String), `short_sha` (String), `message` (String)

**Worktree types:**
- `WorktreeInfo` — fields: `path` (String), `branch` (String), `commit` (CommitInfo), `status` (GitStatus), `is_main` (bool), `is_current` (bool). Skip `last_modified` for now — we'll add it in a later step when we cover DateTime.

**Setup types:**
- `SetupStep` — enum with variants: `Install { command: Option<String>, optional: Option<bool> }`, `Copy { from: CopySource, to: String, exclude: Option<Vec<String>>, optional: Option<bool> }`, `Run { command: String, optional: Option<bool> }`, `Verify { path: String, label: Option<String>, optional: Option<bool> }`
- `CopySource` — enum: `Single(String)` or `Multiple(Vec<String>)`. You'll need to derive or implement Serialize/Deserialize for this — serde handles enum serialization automatically.

**Health types:**
- `HealthIssue` — fields: `kind` (IssueKind), `message` (String)
- `IssueKind` — enum with variants: `Dirty`, `Ahead`, `Behind`, `Stale`, `Orphaned`, `Verification`
- `WorktreeHealthReport` — fields: `worktree` (WorktreeInfo), `issues` (Vec<HealthIssue>), `is_healthy` (bool)

**Command types:**
- `CommandContext` — fields: `cwd` (String), `json` (bool)

Add `impl WorktreeInfo` with:
- `pub fn is_clean(&self) -> bool` — returns true if status is not dirty AND ahead/behind are both 0

Add `impl IssueKind` with:
- `pub fn as_str(&self) -> &'static str` — returns "dirty", "ahead", "behind", "stale", "orphaned", or "verification"

Then update `src/main.rs` to include the module:
```rust
mod types;

fn main() {
    println!("wt-cli — Rust rewrite in progress");
}
```

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

Place the test module at the bottom of `src/types.rs`. Run `cargo test` (after installing Rust) to check.

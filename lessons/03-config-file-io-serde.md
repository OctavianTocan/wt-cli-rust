# Step 3: Config — File I/O and Serialization

## Architecture Context

Open `src/core/config.ts` in the [original wt-cli](https://github.com/OctavianTocan/wt-cli/blob/main/src/core/config.ts).

config.ts is responsible for loading the project's configuration. It does four things:

1. **Reads** either `wt.config.json` or `package.json#wt` from the project root
2. **Validates** the JSON against a schema (using zod — a runtime type checker)
3. **Applies defaults** for any missing fields (worktreeDir defaults to "tree", staleDays to 30, etc.)
4. **Returns** a `LoadedConfig` with the validated config, the source it came from, and the root path

This module is the first one that combines process I/O (finding the git root via `getCurrentWorktreeRoot`) with file I/O (reading JSON) and structured parsing (validating + defaults). Every command calls `loadConfig()` as one of its first steps.

**Why this is step 3:** You've learned structs/enums (step 1) and process spawning (step 2). Now you combine them: read a file from disk, parse it into the types you defined, handle missing files, apply defaults. This is where Rust's `serde` ecosystem shows its power — JSON parsing in Rust is both safer and faster than in TypeScript.

---

## Rust Concepts

### Concept 1: File I/O with std::fs

Reading a file in Rust:

```rust
use std::fs;
use std::path::Path;

let content = fs::read_to_string("wt.config.json")?;
```

That's it. `read_to_string` returns `Result<String, io::Error>`. The `?` propagates the error if the file doesn't exist or can't be read.

Checking if a file exists:
```rust
let path = Path::new("wt.config.json");
if path.exists() {
    let content = fs::read_to_string(path)?;
}
```

**vs TypeScript:** `fs.readFileSync(path, "utf8")` — similar, but Rust makes the error handling explicit instead of throwing.

**Common gotcha:** `Path::new("file")` creates a path reference but doesn't check if the file exists. `.exists()` does a syscall. Don't check-then-read (race condition) — just read and handle the error. But for this module, we check because we want to try multiple candidate files.

### Concept 2: Serde — Serialization and Deserialization

`serde` is Rust's universal serialization framework. With `#[derive(Deserialize)]` on your types, you can parse JSON directly into them:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct WtConfig {
    worktree_dir: String,
    stale_days: u32,
}

let config: WtConfig = serde_json::from_str(&json_string)?;
```

If the JSON doesn't match the struct, you get a clear error: "missing field `worktree_dir`" or "invalid type: expected u32, got string."

**Defaults with serde:** TypeScript's zod has `.default()`. Serde has `#[serde(default)]`:

```rust
#[derive(Deserialize)]
struct WtConfig {
    #[serde(default = "default_worktree_dir")]
    worktree_dir: String,
    #[serde(default)]
    stale_days: u32,  // uses u32::default() = 0
}

fn default_worktree_dir() -> String { "tree".to_string() }
```

If the field is missing from JSON, serde calls the default function. If you just write `#[serde(default)]`, it uses the type's `Default` trait implementation.

**Common gotcha:** All fields must be deserializable. If you have `Option<String>`, serde treats missing JSON fields as `None` automatically. If you have `Vec<String>`, you need `#[serde(default)]` or the JSON must include the field.

---

## Your Task

**File:** `src/config.rs` (create this file)

**Update `src/types.rs`** — add `#[serde(default)]` annotations to WtConfig fields that have defaults:
- `worktree_dir` defaults to `"tree"`
- `main_branch` defaults to `"main"`
- `default_base` defaults to `"main"`
- `remote` defaults to `"origin"`
- `auto_setup` defaults to `true`
- `stale_days` defaults to `30`
- `setup` defaults to `SetupConfig { steps: vec![] }`
- `lifecycle_scripts` defaults to `LifecycleScripts { postsetup: None, preclean: None }`

Implement `src/config.rs` with these functions:

**`pub fn get_default_config() -> WtConfig`**
- Returns a WtConfig with all default values (matching the TypeScript `getDefaultConfig()`)

**`pub fn load_config(cwd: &str) -> Result<LoadedConfig, String>`**
- First, find the git root by running `git rev-parse --show-toplevel` (use your `process::run_process`)
- Then try reading `wt.config.json` from that root — if it exists, parse it as WtConfig
- If not found, try reading `package.json` — if it exists, parse it and look for a `"wt"` key
- If neither found, return the default config with source = "defaults"
- On any parse error (malformed JSON), return Err with a descriptive message
- Merge defaults with provided values: start with defaults, then overlay parsed fields

Add a helper `fn read_json_file(path: &str) -> Result<serde_json::Value, String>` that reads and parses JSON from a file.

Update `src/main.rs`:
```rust
mod types;
mod process;
mod config;

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
    fn default_config_has_expected_values() {
        // Call get_default_config()
        // Assert worktree_dir == "tree", stale_days == 30, remote == "origin"
    }

    #[test]
    fn parse_minimal_json_uses_defaults() {
        // Create a JSON string: "{}"
        // Parse it into WtConfig with serde_json
        // Assert defaults are applied for missing fields
    }

    #[test]
    fn parse_partial_json_overrides_defaults() {
        // Create JSON: {"worktreeDir": "custom-tree", "staleDays": 14}
        // Parse into WtConfig
        // Assert worktree_dir is "custom-tree" but main_branch is still "main" (default)
    }

    #[test]
    fn invalid_json_returns_error() {
        // Create a string that's not valid JSON: "not json"
        // Try to parse it as WtConfig
        // Assert the result is Err(...)
    }

    #[test]
    fn load_config_returns_default_when_no_files() {
        // In a temp directory with no config files
        // Call load_config(temp_dir)
        // Assert source is "defaults"
    }

    #[test]
    fn load_config_reads_wt_config_json() {
        // In a temp directory with a wt.config.json file
        // Write {"worktreeDir": "my-trees"} to it
        // (This test requires a git repo, so you may need to `git init` in the temp dir)
        // Call load_config(temp_dir)
        // Assert config.worktree_dir == "my-trees"
    }
}
```

For temp directory testing, add `tempfile = "3"` to `[dev-dependencies]` in Cargo.toml (it's already there).

Place the test module at the bottom of `src/config.rs`. Run `cargo test` to check.

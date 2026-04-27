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

## Original Code (Before)

```typescript
// src/core/config.ts — the full file (182 lines)

import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { z } from "zod";
import type { LoadedConfig, SetupStep, WtConfig } from "../types.js";
import { getCurrentWorktreeRoot } from "./git.js";

// Zod schemas for validation (this is the TS equivalent of serde)
const installStepSchema = z.object({
  type: z.literal("install"),
  command: z.string().optional(),
  optional: z.boolean().optional(),
}).strict();

const copyStepSchema = z.object({
  type: z.literal("copy"),
  from: z.union([z.string(), z.array(z.string()).min(1)]),
  to: z.string(),
  exclude: z.array(z.string()).optional(),
  optional: z.boolean().optional(),
}).strict();

// ...more schemas...

const canonicalConfigSchema = z.object({
  worktreeDir: z.string().default("tree"),
  mainBranch: z.string().default("main"),
  devBranch: z.string().optional(),
  defaultBase: z.string().default("main"),
  remote: z.string().default("origin"),
  autoSetup: z.boolean().default(true),
  staleDays: z.number().int().positive().default(30),
  setup: z.object({ steps: z.array(...) }).default({ steps: [] }),
  lifecycleScripts: z.object({ ... }).default({}),
}).strict();

// Default config factory
export function getDefaultConfig(): WtConfig {
  return canonicalConfigSchema.parse({
    setup: {
      steps: [
        { type: "install", command: "auto" },
        { type: "copy", from: ".env*", to: ".", exclude: [".env.example"], optional: true },
        { type: "verify", path: "node_modules", label: "Dependencies installed" },
      ],
    },
    lifecycleScripts: { postsetup: "wt:postsetup", preclean: "wt:preclean" },
  });
}

// Parse with defaults
export function parseConfigData(raw: Partial<WtConfig> = {}): WtConfig {
  const defaults = getDefaultConfig();
  return canonicalConfigSchema.parse({
    ...defaults,
    ...raw,
    setup: { ...defaults.setup, ...raw.setup },
    lifecycleScripts: { ...defaults.lifecycleScripts, ...raw.lifecycleScripts },
  });
}

// Read JSON from file
async function readJsonFile(path: string): Promise<unknown> {
  const content = await readFile(path, "utf8");
  return JSON.parse(content);
}

// Main loader
export async function loadConfig(cwd = process.cwd()): Promise<LoadedConfig> {
  const rootPath = getCurrentWorktreeRoot(cwd);
  const candidates = [
    { source: "wt.config.json", path: join(rootPath, "wt.config.json") },
    { source: "package.json#wt", path: join(rootPath, "package.json") },
  ];

  for (const candidate of candidates) {
    try {
      const parsed = await readJsonFile(candidate.path);
      if (candidate.source === "package.json#wt") {
        const pkg = packageJsonSchema.parse(parsed);
        if (pkg.wt) {
          return {
            config: parseConfigData(pkg.wt as Partial<WtConfig>),
            source: candidate.source,
            rootPath,
          };
        }
        continue;
      }

      return {
        config: parseConfigData(parsed as Partial<WtConfig>),
        source: candidate.source,
        rootPath,
      };
    } catch (error) {
      if (isMissingFileError(error)) continue;
      throw new Error(`Failed to load ${candidate.source}: ${message}`);
    }
  }

  return { config: getDefaultConfig(), source: "defaults", rootPath };
}
```

---

## Translation Walkthrough

### Mapping 1: Zod schemas → serde annotations

**TypeScript (with Zod):**
```typescript
const canonicalConfigSchema = z.object({
  worktreeDir: z.string().default("tree"),
  mainBranch: z.string().default("main"),
  staleDays: z.number().int().positive().default(30),
  autoSetup: z.boolean().default(true),
  devBranch: z.string().optional(),
});
```

**Rust (with serde):**
```rust
#[derive(Deserialize)]
pub struct WtConfig {
    #[serde(default = "default_worktree_dir")]
    pub worktree_dir: String,
    #[serde(default = "default_main_branch")]
    pub main_branch: String,
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    #[serde(default = "default_true")]
    pub auto_setup: bool,
    #[serde(default)]
    pub dev_branch: Option<String>,
}
```

What changed and why:
- **Zod → serde.** Both do the same job: parse + validate + apply defaults. Zod is a runtime library. Serde is a compile-time derive macro — it generates the parsing code at build time.
- `z.string().default("tree")` → `#[serde(default = "default_worktree_dir")]`. You write a function that returns the default value. If the JSON field is missing, serde calls it.
- `z.string().optional()` → `Option<String>` with `#[serde(default)]`. `Option<T>` already defaults to `None`, so `#[serde(default)]` is enough — no custom function needed.
- `z.number().int().positive()` → `u32`. Rust's type system enforces "positive integer" at compile time. No runtime validation needed for that constraint.
- You also need to add `#[serde(rename_all = "camelCase")]` on the struct — this tells serde to map `worktree_dir` (Rust) ↔ `worktreeDir` (JSON). Without it, serde looks for snake_case in the JSON.

### Mapping 2: async readFile → sync std::fs

**TypeScript:**
```typescript
async function readJsonFile(path: string): Promise<unknown> {
  const content = await readFile(path, "utf8");
  return JSON.parse(content);
}
```

**Rust:**
```rust
fn read_json_file(path: &str) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in {}: {}", path, e))
}
```

What changed and why:
- `async/await` → synchronous. The TS version is async because Node's fs.promises are async. In Rust, `std::fs::read_to_string` is sync. For a CLI tool that reads one config file, sync is simpler and fine. (If you needed to read hundreds of files concurrently, you'd use tokio's async fs.)
- `JSON.parse(content)` → `serde_json::from_str(&content)`. Both parse JSON. The difference: `JSON.parse` returns `any` (you hope it's the right shape). `serde_json::from_str` returns a typed `Result<T, Error>` — the compiler verifies the type matches.
- `Promise<unknown>` → `Result<serde_json::Value, String>`. We return `serde_json::Value` (equivalent to `unknown`/`any` in JS) because we don't know the shape yet — we'll parse it into `WtConfig` after checking which file we're reading.

### Mapping 3: loadConfig candidate loop

**TypeScript:**
```typescript
const candidates = [
  { source: "wt.config.json", path: join(rootPath, "wt.config.json") },
  { source: "package.json#wt", path: join(rootPath, "package.json") },
];

for (const candidate of candidates) {
  try {
    const parsed = await readJsonFile(candidate.path);
    // ...handle package.json vs wt.config.json differently
    return { config: parseConfigData(parsed), source: candidate.source, rootPath };
  } catch (error) {
    if (isMissingFileError(error)) continue;
    throw new Error(`Failed to load ${candidate.source}: ${message}`);
  }
}

return { config: getDefaultConfig(), source: "defaults", rootPath };
```

**Rust:**
```rust
let candidates = vec![
    ("wt.config.json", root_path.join("wt.config.json")),
    ("package.json#wt", root_path.join("package.json")),
];

for (source, path) in candidates {
    match read_json_file(path.to_str().unwrap()) {
        Ok(parsed) => {
            if source == "package.json#wt" {
                if let Some(wt) = parsed.get("wt") {
                    let config: WtConfig = serde_json::from_value(wt.clone())
                        .map_err(|e| format!("Invalid config in package.json#wt: {}", e))?;
                    return Ok(LoadedConfig { config, source: source.to_string(), root_path });
                }
                continue;
            }
            let config: WtConfig = serde_json::from_value(parsed)
                .map_err(|e| format!("Invalid config in {}: {}", source, e))?;
            return Ok(LoadedConfig { config, source: source.to_string(), root_path });
        }
        Err(e) if e.contains("No such file") => continue,  // file doesn't exist, try next
        Err(e) => return Err(e),  // malformed JSON, fail
    }
}

Ok(LoadedConfig {
    config: get_default_config(),
    source: "defaults".to_string(),
    root_path,
})
```

What changed and why:
- The overall structure is identical: try candidates in order, skip missing files, fail on malformed JSON, fall back to defaults.
- `try/catch` → `match` on the Result. The `Ok/Err` pattern replaces the try-catch. `Err(e) if e.contains(...)` is like a catch-with-condition.
- `join(rootPath, "wt.config.json")` → `root_path.join("wt.config.json")`. PathBuf's `.join()` does the same thing as Node's `path.join()`.
- `parsed.get("wt")` — serde_json::Value has a `.get()` method that works like accessing a JS object property. Returns `Option<&Value>`.
- The Rust version is more verbose because every fallible operation is explicit. But the logic flow is the same.

---

## Rust Concepts

### Concept 1: Serde — Serialization and Deserialization

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

**Defaults with serde:**
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

### Concept 2: Path and PathBuf

```rust
use std::path::PathBuf;

let root = PathBuf::from("/some/project");
let config_path = root.join("wt.config.json");  // /some/project/wt.config.json
let path_str = config_path.to_str().unwrap();    // convert to &str for fs operations
```

`PathBuf` is like Node's `path.join()` result — an owned, mutable path. `Path` is the borrowed version (like `&str` vs `String`). For this module, `PathBuf` is what you'll use most.

---

## Your Task

**Update `src/types.rs`** — add serde derives and default annotations to WtConfig. You need to add `serde` to the derive list and add `#[serde(default)]` annotations:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WtConfig {
    #[serde(default = "default_tree")]
    pub worktree_dir: String,
    #[serde(default = "default_main")]
    pub main_branch: String,
    #[serde(default)]
    pub dev_branch: Option<String>,
    #[serde(default = "default_main")]
    pub default_base: String,
    #[serde(default = "default_origin")]
    pub remote: String,
    #[serde(default = "default_true")]
    pub auto_setup: bool,
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    #[serde(default)]
    pub setup: SetupConfig,
    #[serde(default)]
    pub lifecycle_scripts: LifecycleScripts,
}
```

Add default functions at the top of the file:
```rust
fn default_tree() -> String { "tree".to_string() }
fn default_main() -> String { "main".to_string() }
fn default_origin() -> String { "origin".to_string() }
fn default_true() -> bool { true }
fn default_stale_days() -> u32 { 30 }
```

Also add `Serialize, Deserialize` to the derives on `SetupStep`, `CopySource`, `LifecycleScripts`, and `SetupConfig`.

**Create `src/config.rs`** with these functions:

**`pub fn get_default_config() -> WtConfig`**
- Returns a WtConfig with all default values (matching the TypeScript `getDefaultConfig()`)

**`pub fn load_config(cwd: &str) -> Result<LoadedConfig, String>`**
- First, find the git root by running `git rev-parse --show-toplevel` (use your `process::run_process`)
- Then try reading `wt.config.json` from that root — if it exists, parse it as WtConfig
- If not found, try reading `package.json` — if it exists, parse it and look for a `"wt"` key
- If neither found, return the default config with source = "defaults"
- On any parse error (malformed JSON), return Err with a descriptive message

Add a helper `fn read_json_file(path: &str) -> Result<serde_json::Value, String>`.

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
}
```

Place the test module at the bottom of `src/config.rs`. Run `cargo test` to check.

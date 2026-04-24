# Step 2: Process Spawning — The I/O Foundation

## Architecture Context

Open `src/core/process.ts` in the [original wt-cli](https://github.com/OctavianTocan/wt-cli/blob/main/src/core/process.ts).

This is the lowest-level module in the project. **Everything** that talks to the outside world goes through here — git commands, shell commands, dependency installs, setup scripts. It wraps Node's `child_process.spawnSync` into two functions:

- `runProcess(command, args, options)` — runs a command with direct arguments (e.g. `git`, `["status", "--porcelain"]`)
- `runShell(command, options)` — runs a command through a shell (e.g. `"npm install"`)

Both return a `ProcessResult` with stdout, stderr, and exit status. Both throw on failure unless `allowFailure: true`.

**Why this is step 2:** Before you can do anything useful (read config, run git, manage worktrees), you need a way to run external commands. This module is small (79 lines) but foundational. git.ts, setup.ts, and shell.ts all depend on it. And it's the perfect place to learn Rust's error handling — every process call can fail, and Rust forces you to acknowledge that.

---

## Original Code (Before)

```typescript
// src/core/process.ts — the full file (79 lines)

import { spawnSync } from "node:child_process";

export interface ProcessOptions {
  cwd?: string;
  allowFailure?: boolean;
  env?: NodeJS.ProcessEnv;
  stdio?: "inherit" | "pipe";
}

export interface ProcessResult {
  stdout: string;
  stderr: string;
  status: number;
}

function formatFailure(
  command: string,
  args: string[],
  result: ProcessResult,
): string {
  const rendered = [command, ...args].join(" ");
  const output = [result.stdout.trim(), result.stderr.trim()]
    .filter(Boolean)
    .join("\n");
  return output ? `${rendered}\n${output}` : rendered;
}

export function runProcess(
  command: string,
  args: string[],
  options: ProcessOptions = {},
): ProcessResult {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: options.env,
    stdio: options.stdio ?? "pipe",
    encoding: "utf8",
  });

  const normalized: ProcessResult = {
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    status: result.status ?? 0,
  };

  if (normalized.status !== 0 && !options.allowFailure) {
    throw new Error(formatFailure(command, args, normalized));
  }

  return normalized;
}

export function runShell(
  command: string,
  options: ProcessOptions = {},
): ProcessResult {
  const result = spawnSync(command, {
    cwd: options.cwd,
    env: options.env,
    stdio: options.stdio ?? "pipe",
    shell: true,
    encoding: "utf8",
  });

  const normalized: ProcessResult = {
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    status: result.status ?? 0,
  };

  if (normalized.status !== 0 && !options.allowFailure) {
    const output = [normalized.stdout.trim(), normalized.stderr.trim()]
      .filter(Boolean)
      .join("\n");
    throw new Error(output ? `${command}\n${output}` : command);
  }

  return normalized;
}
```

---

## Translation Walkthrough

### Mapping 1: Interfaces → structs

**TypeScript:**
```typescript
export interface ProcessOptions {
  cwd?: string;
  allowFailure?: boolean;
  env?: NodeJS.ProcessEnv;
  stdio?: "inherit" | "pipe";
}

export interface ProcessResult {
  stdout: string;
  stderr: string;
  status: number;
}
```

**Rust:**
```rust
pub struct ProcessOptions {
    pub cwd: Option<String>,
    pub allow_failure: bool,
    pub env: Option<Vec<(String, String)>>,
}

pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
```

What changed and why:
- `cwd?: string` → `cwd: Option<String>`. Same pattern as step 1 — `?` becomes `Option<T>`.
- `allowFailure?: boolean` → `allow_failure: bool`. Note: no `Option` here. In the TS version, `allowFailure` defaults to `undefined` (falsy). In Rust, we'll use `Default` to set it to `false` explicitly. No need for `Option` when the default is a plain `false`.
- `env?: NodeJS.ProcessEnv` → `env: Option<Vec<(String, String)>>`. Node's `ProcessEnv` is a string map. Rust doesn't have a built-in env type — we use a list of key-value pairs. (You could also use `HashMap<String, String>` but Vec is simpler for now.)
- `stdio?: "inherit" | "pipe"` → dropped for now. We'll always capture output (equivalent to `"pipe"`). Adding stdio control is a later refinement.
- `status: number` → `status: i32`. Exit codes are signed 32-bit integers. Negative values signal signal-killed processes on Unix.

### Mapping 2: throw → Result<T, E>

**TypeScript:**
```typescript
export function runProcess(
  command: string,
  args: string[],
  options: ProcessOptions = {},
): ProcessResult {
  const result = spawnSync(command, args, { ... });
  
  if (normalized.status !== 0 && !options.allowFailure) {
    throw new Error(formatFailure(command, args, normalized));
  }
  
  return normalized;
}
```

**Rust:**
```rust
pub fn run_process(
    command: &str,
    args: &[&str],
    options: &ProcessOptions,
) -> Result<ProcessResult, String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }
    
    let output = cmd.output().map_err(|e| e.to_string())?;
    let status = output.status.code().unwrap_or(0);
    
    let result = ProcessResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status,
    };
    
    if result.status != 0 && !options.allow_failure {
        return Err(format_failure(command, args, &result));
    }
    
    Ok(result)
}
```

What changed and why:
- `throw new Error(...)` → `return Err(...)`. In TS, any function can throw. In Rust, the return type declares what can go wrong: `Result<ProcessResult, String>` means "either Ok(ProcessResult) or Err(String)." The compiler checks that callers handle both.
- `spawnSync(...)` → `Command::new(command).args(args).output()`. Same idea — run a process, capture output. Different API: Command is a builder pattern.
- `.output()` returns `Result<Output, io::Error>` — two layers of failure. The outer Result is "did the process even start?" The inner exit code is "did it succeed?" In TS, spawnSync returns null-ish values on crash. In Rust, both are explicit.
- `.map_err(|e| e.to_string())?` — converts the io::Error into a plain String (to match our `Result<_, String>`), then `?` returns early if it failed.
- `String::from_utf8_lossy(&output.stdout)` — process output is raw bytes (`Vec<u8>`). This converts to String, replacing invalid UTF-8 with �. TS's `encoding: "utf8"` does the same thing silently.
- `command: string` → `command: &str`. Function parameters that just read text use `&str` (borrowed string) instead of `String` (owned). This lets callers pass either `String` or `&str` without cloning. You'll see `&str` in function signatures and `String` in struct fields — that's the standard pattern.

### Mapping 3: runShell → sh -c

**TypeScript:**
```typescript
export function runShell(command: string, options: ProcessOptions = {}): ProcessResult {
  const result = spawnSync(command, {
    shell: true,
    // ...other options
  });
  // ...same error handling
}
```

**Rust:**
```rust
pub fn run_shell(command: &str, options: &ProcessOptions) -> Result<ProcessResult, String> {
    // Uses: sh -c "command string"
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    // ...same body as run_process
}
```

What changed and why:
- Node's `shell: true` option means "run this through the system shell." Rust's `Command` doesn't have that option — you explicitly run `sh -c <command>`. More verbose, but you see exactly what's happening.
- The rest of the function body is identical to `run_process`. In TS, runProcess and runShell share structure via the options object. In Rust, they share structure by… writing similar code. You could refactor this into a shared helper later, but for learning, keeping them separate is clearer.

### Mapping 4: Default values

**TypeScript:**
```typescript
options: ProcessOptions = {}  // default to empty object
// then: options.cwd ?? undefined, options.allowFailure ?? false
```

**Rust:**
```rust
impl Default for ProcessOptions {
    fn default() -> Self {
        ProcessOptions {
            cwd: None,
            allow_failure: false,
            env: None,
        }
    }
}
```

What changed and why:
- TS uses default parameter values (`= {}`) and nullish coalescing (`??`). Rust uses the `Default` trait — a standard way to say "give me the zero/empty version of this type." Callers write `ProcessOptions::default()` to get one.
- This is more explicit but also more discoverable — `Default` is a convention across the entire Rust ecosystem.

---

## Rust Concepts

### Concept 1: std::process::Command

Rust's standard library has `std::process::Command` for spawning processes.

**TypeScript (Node.js):**
```typescript
import { spawnSync } from "node:child_process";
const result = spawnSync("git", ["status", "--porcelain"], { encoding: "utf8" });
```

**Rust:**
```rust
use std::process::Command;

let output = Command::new("git")
    .args(["status", "--porcelain"])
    .current_dir("/some/path")
    .output()
    .expect("failed to run git");
```

`Command::new("name")` creates a command builder. You chain methods to configure it (args, working directory, environment). `.output()` runs it and captures stdout/stderr. `.status()` runs it and returns just the exit code.

**The key difference:** In Node, spawnSync returns synchronously. In Rust, `.output()` and `.status()` are also synchronous (they block the thread). If you need async, that's `.spawn()` + tokio — but we don't need that here. wt-cli is a CLI tool, blocking is fine.

**Common gotcha:** `.output()` returns `Result<Output, io::Error>`. You must handle the outer Result (did the process even start?) separately from the exit status (did the process succeed?). Two layers of "can go wrong."

### Concept 2: Result<T, E> and the ? Operator

This is Rust's error handling. Every operation that can fail returns `Result<T, E>` — either `Ok(value)` or `Err(error)`.

```rust
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?;  // ? = early return on error
    Ok(content)
}
```

The `?` operator means: "if this is an error, return immediately with that error. If it's success, unwrap the value and keep going."

**TypeScript equivalent:**
```typescript
function readFile(path: string): string {
    const content = fs.readFileSync(path, "utf8"); // throws on error
    return content;
}
```

The difference: in TypeScript, any function can throw anything. In Rust, the function signature **declares** what errors are possible (`Result<T, E>`), and the compiler checks that you handle them. No surprises.

**Common gotcha:** `?` only works in functions that return `Result`. You can't use `?` in `main()` unless main returns `Result<..., ...>`. For now, we'll make our functions return `Result<ProcessResult, String>` so `?` works inside them.

---

## Your Task

**File:** `src/process.rs` (create this file)

Define a `ProcessOptions` struct:
- `cwd: Option<String>` — working directory (None = current dir)
- `allow_failure: bool` — if true, don't error on non-zero exit
- `env: Option<Vec<(String, String)>>` — extra environment variables

Define a `ProcessResult` struct:
- `stdout: String`
- `stderr: String`
- `status: i32` — exit code

Implement `Default` for `ProcessOptions` (cwd = None, allow_failure = false, env = None).

Implement two functions:

**`pub fn run_process(command: &str, args: &[&str], options: &ProcessOptions) -> Result<ProcessResult, String>`**
- Use `std::process::Command::new(command).args(args)` to run the command
- If `options.cwd` is Some, set `.current_dir()`
- Capture output with `.output()`
- If the outer command fails to start (io::Error), return `Err(message)`
- If exit status is non-zero AND `allow_failure` is false, return `Err(formatted_message)`
- Otherwise return `Ok(ProcessResult { stdout, stderr, status })`

**`pub fn run_shell(command: &str, options: &ProcessOptions) -> Result<ProcessResult, String>`**
- Same as run_process but uses `Command::new("sh").arg("-c").arg(command)` to run through a shell
- Same error handling logic

Add a helper: `fn format_failure(command: &str, args: &[&str], result: &ProcessResult) -> String` that formats a nice error message with the command and its output.

Update `src/main.rs` to include the module:
```rust
mod types;
mod process;

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
    fn run_process_echo() {
        // Run: echo "hello"
        // Assert stdout contains "hello"
        // Assert status is 0
    }

    #[test]
    fn run_process_with_cwd() {
        // Run: pwd
        // Set cwd to "/" 
        // Assert stdout contains "/"
    }

    #[test]
    fn run_process_failure_throws() {
        // Run: false (exits with 1)
        // Assert the result is Err(...)
    }

    #[test]
    fn run_process_allow_failure() {
        // Run: false with allow_failure: true
        // Assert the result is Ok(...) with status 1
    }

    #[test]
    fn run_shell_command() {
        // Run: echo "hello from shell" via run_shell
        // Assert stdout contains "hello from shell"
    }

    #[test]
    fn process_options_default() {
        // Create ProcessOptions::default()
        // Assert cwd is None, allow_failure is false, env is None
    }
}
```

Place the test module at the bottom of `src/process.rs`. Run `cargo test` to check.

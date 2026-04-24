# Step 2: Process Spawning — The I/O Foundation

## Architecture Context

Open `src/core/process.ts` in the [original wt-cli](https://github.com/OctavianTocan/wt-cli/blob/main/src/core/process.ts).

This is the lowest-level module in the project. **Everything** that talks to the outside world goes through here — git commands, shell commands, dependency installs, setup scripts. It wraps Node's `child_process.spawnSync` into two functions:

- `runProcess(command, args, options)` — runs a command with direct arguments (e.g. `git`, `["status", "--porcelain"]`)
- `runShell(command, options)` — runs a command through a shell (e.g. `"npm install"`)

Both return a `ProcessResult` with stdout, stderr, and exit status. Both throw on failure unless `allowFailure: true`.

**Why this is step 2:** Before you can do anything useful (read config, run git, manage worktrees), you need a way to run external commands. This module is small (79 lines) but foundational. git.ts, setup.ts, and shell.ts all depend on it. And it's the perfect place to learn Rust's error handling — every process call can fail, and Rust forces you to acknowledge that.

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

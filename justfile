# wt-cli-rust — task runner
# Install just: cargo install just
# Run `just` to see all available recipes

default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode (optimized)
release:
    cargo build --release

# Run the binary
run:
    cargo run

# Run all tests
test:
    cargo test

# Run tests with output printed
test-verbose:
    cargo test -- --nocapture

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format code (writes in place)
fmt:
    cargo fmt

# Check formatting without changing files
fmt-check:
    cargo fmt -- --check

# Full check: format + lint + test
check: fmt-check lint test

# Watch for changes and rerun tests (requires cargo-watch)
watch:
    cargo watch -x test

# Clean build artifacts
clean:
    cargo clean

# AI-powered commit using claude CLI
# Stages all changes, generates a commit message, commits, and pushes.
# Requires: claude CLI (npm i -g @anthropic-ai/claude-code)
commit:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v claude &>/dev/null; then
        echo "Error: claude CLI not found."
        echo "Install with: npm i -g @anthropic-ai/claude-code"
        exit 1
    fi

    if [ -z "$(git status --porcelain)" ]; then
        echo "Nothing to commit."
        exit 0
    fi

    # Stage everything
    git add -A

    # Generate commit message from the diff
    MSG=$(claude -p "Look at the staged git diff and generate ONE concise conventional commit message. \
Only output the message text. Format: type(scope?): summary. Use feat/fix/refactor/test/chore/docs. \
No body, no footer, just the first line. Keep it under 72 chars." \
    --allowedTools "Bash(git diff --cached*),Read" 2>/dev/null)

    if [ -z "$MSG" ]; then
        echo "Error: claude returned empty message. Commiting with fallback."
        git commit -m "chore: update files"
    else
        git commit -m "$MSG"
    fi

    git push
    echo "Pushed: $MSG"

# Quick alias: just ac = stage all + AI commit
ac: commit

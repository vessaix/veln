# Veln Foundation Build Instructions

## What This Is

Build instructions for a FreeBSD virtualization CLI in Rust. Follow every instruction exactly. Do not add code, dependencies, or modules that are not listed here. Do not improvise.

---

## Critical Rules (Read First)

1. Use `edition = "2021"` in Cargo.toml. Not 2024. Not 2018.
2. No `async`, no `tokio`. Everything is synchronous.
3. No `tracing`, no `uuid`, no `mockall`. Only the dependencies listed below.
4. Do NOT create empty modules or placeholder files. Only create files listed here.
5. Crate name in Cargo.toml uses a hyphen: `is-root`, not `is_root`.
6. Every public function returning `Result` must have a `/// # Errors` doc comment. Clippy pedantic requires this.
7. Use `path.display()` for formatting paths, not `path:?` or `{path:?}`. Clippy requires this.
8. Use `map_or_else()` not `map().unwrap_or_else()` on `Result`. Clippy requires this.
9. `main()` returns `miette::Result<()>`. All other code returns `crate::error::Result<()>`. Bridge them with `.map_err(miette::Report::new)` in main.
10. Errors go to stderr (`eprintln!`). Results go to stdout (`println!`).
11. After every code change, run `just qa` (format, check, test). Do not proceed until it passes clean.
12. Track progress in `TODO.md`. Check off each step only after `just qa` passes.

---

## Official Rust References

You MUST follow these official docs. Do not invent patterns.

- Project layout: https://doc.rust-lang.org/cargo/guide/project-layout.html
- CLI project pattern (lib.rs + main.rs split): https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html
- Error handling: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- Modules: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
- Env vars for config: https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html
- stderr for errors: https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html
- Cargo manifest: https://doc.rust-lang.org/cargo/reference/manifest.html
- Cargo lints: https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section
- Clippy lints: https://rust-lang.github.io/rust-clippy/stable/index.html
- rustfmt config: https://rust-lang.github.io/rustfmt/
- clap: https://docs.rs/clap/latest/clap/
- thiserror: https://docs.rs/thiserror/latest/thiserror/
- miette: https://docs.rs/miette/latest/miette/
- serde: https://serde.rs/

---

## Step 1: Initialize Project

Run:

```bash
cargo new veln --bin
cd veln
```

This creates `src/main.rs`, `Cargo.toml`, and `.gitignore`. Do not create any other files yet.

---

## Step 2: Create rustfmt.toml

Create file `rustfmt.toml` in the project root:

```toml
max_width = 100
use_field_init_shorthand = true
```

Do NOT add `edition` to this file. It inherits from Cargo.toml.

---

## Step 3: Create justfile

Create file `justfile` in the project root:

```just
default: check test

check:
    cargo check
    cargo clippy -- -D warnings

test:
    cargo test

qa: fmt check test
    @echo "All checks passed."

run *ARGS:
    sudo cargo run -- {{ARGS}}

fmt:
    cargo fmt --all
```

**Rule:** After writing or modifying any code, always run `just qa` before considering the step done. Do not move to the next step until `just qa` passes with zero errors and zero warnings.

---

## Step 3b: Create TODO.md

Create file `TODO.md` in the project root. Use this to track progress through these build steps:

```markdown
# Veln Foundation — Progress

- [ ] Step 1: Initialize project
- [ ] Step 2: Create rustfmt.toml
- [ ] Step 3: Create justfile
- [ ] Step 4: Replace Cargo.toml
- [ ] Step 5: Create src/error.rs
- [ ] Step 6: Create src/config.rs
- [ ] Step 7: Create src/cli.rs
- [ ] Step 8: Create src/application.rs
- [ ] Step 9: Create src/lib.rs
- [ ] Step 10: Replace src/main.rs
- [ ] Step 11: Verify (`just qa` passes)
```

**Rule:** Check off each step as you complete it. After every code change, run `just qa` and only check off the step if it passes.

---

## Step 4: Replace Cargo.toml

Replace the entire contents of `Cargo.toml` with:

```toml
[package]
name = "veln"
version = "0.1.0"
edition = "2021"
authors = ["Amr Essam <amr@vessaix.com>"]
description = "FreeBSD Virtualization Management CLI"
license = "BSD-3-Clause"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
miette = { version = "7.2", features = ["fancy"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
is-root = "0.1"

[dev-dependencies]
pretty_assertions = "1.4"
```

---

## Step 5: Create src/error.rs

```rust
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum VelnError {
    #[error("Root privileges required")]
    #[diagnostic(code(veln::auth::root_required), help("Try running with sudo."))]
    RootRequired,

    #[error("VM not found: {0}")]
    #[diagnostic(code(veln::vm::not_found))]
    VmNotFound(String),

    #[error("ZFS operation failed: {0}")]
    #[diagnostic(code(veln::infra::zfs))]
    ZfsError(String),

    #[error("Configuration error: {0}")]
    #[diagnostic(code(veln::config::invalid))]
    Config(String),

    #[error(transparent)]
    #[diagnostic(code(veln::io))]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, VelnError>;
```

---

## Step 6: Create src/config.rs

```rust
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

use crate::error::{Result, VelnError};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub zfs_pool: String,
    pub vm_root: String,
}

impl Config {
    /// # Errors
    /// Returns `VelnError::Config` if the file cannot be read or contains invalid TOML.
    pub fn load() -> Result<Self> {
        let path = Self::path();

        let content = fs::read_to_string(&path)
            .map_err(|e| VelnError::Config(format!("Failed to read {}: {e}", path.display())))?;

        toml::from_str(&content)
            .map_err(|e| VelnError::Config(format!("Invalid config: {e}")))
    }

    /// Config file path: `VELN_CONFIG` env var, or `/etc/veln/config.toml`.
    fn path() -> PathBuf {
        env::var("VELN_CONFIG")
            .map_or_else(|_| PathBuf::from("/etc/veln/config.toml"), PathBuf::from)
    }
}
```

---

## Step 7: Create src/cli.rs

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "veln", version, about = "FreeBSD VM Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check system readiness
    Check,

    /// List VMs
    List,

    /// Start a VM
    Start { name: String },
}
```

---

## Step 8: Create src/application.rs

```rust
use crate::cli::Commands;
use crate::config::Config;
use crate::error::{Result, VelnError};

/// # Errors
/// Returns `VelnError::RootRequired` if a privileged command is run without root.
/// Returns `VelnError::Config` if the configuration file is missing or invalid.
pub fn run(command: Commands) -> Result<()> {
    match command {
        Commands::Check => {
            require_root()?;
            let config = Config::load()?;
            eprintln!("Config loaded: pool={}, root={}", config.zfs_pool, config.vm_root);
            println!("System check passed.");
        }

        Commands::List => {
            println!("No VMs configured.");
        }

        Commands::Start { name } => {
            require_root()?;
            let _config = Config::load()?;
            println!("Starting VM: {name}");
        }
    }

    Ok(())
}

fn require_root() -> Result<()> {
    if !is_root::is_root() {
        return Err(VelnError::RootRequired);
    }
    Ok(())
}
```

---

## Step 9: Create src/lib.rs

```rust
pub mod application;
pub mod cli;
pub mod config;
pub mod error;
```

Only these four modules. Do not add `pub mod domain;` or `pub mod infrastructure;` — they do not exist yet.

---

## Step 10: Replace src/main.rs

Replace the entire contents of `src/main.rs` with:

```rust
use clap::Parser;
use veln::cli::Cli;

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    veln::application::run(cli.command).map_err(miette::Report::new)
}
```

No async. No tokio. No tracing. The `.map_err(miette::Report::new)` converts `VelnError` into `miette::Report` so diagnostics render correctly.

---

## Step 11: Verify

All three must pass with zero errors and zero warnings:

```bash
cargo check
cargo clippy -- -D warnings
cargo test
```

Smoke test:

```bash
cargo run -- list
# Expected output: "No VMs configured."

cargo run -- start testvm
# Expected output: RootRequired miette diagnostic (unless run as root)
```

---

## Common Mistakes to Avoid

| Mistake | Why It Breaks |
|---------|---------------|
| Using `edition = "2024"` | Not stable in rustfmt, causes formatting errors |
| Adding `tokio` or `async` | Nothing is async; adds complexity and `Send` bound errors |
| Using `is_root` instead of `is-root` in Cargo.toml | Crate not found on crates.io |
| Missing `/// # Errors` on pub fn returning Result | Clippy pedantic fails with `-D warnings` |
| Using `{path:?}` in format strings | Clippy rejects debug formatting for user-facing output |
| Using `.map().unwrap_or_else()` on Result | Clippy pedantic wants `map_or_else()` |
| Declaring `pub mod domain;` without a domain.rs | Compilation error: file not found |
| Returning `crate::Result` from `main()` | miette diagnostics won't render; must return `miette::Result` |
| Creating `domain.rs` AND `domain/` directory | Rust does not allow both; pick one |

---

## What Gets Added Later (Not Now)

| Dependency | When To Add |
|-----------|-------------|
| `tokio` | First async operation (bhyve process spawning) |
| `tracing` + `tracing-subscriber` | First `tracing::info!()` or `tracing::debug!()` call |
| `uuid` | VM identity system in `domain/` |
| `mockall` | First trait that needs mocking in tests |
| `domain/` module | VM state machine implementation |
| `infrastructure/` module | ZFS/bhyve system calls |

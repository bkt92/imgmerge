# AGENTS.md - imgmerge Development Guide

This document provides guidelines for AI agents working on this codebase.

## Project Overview

`imgmerge` is a fast Rust CLI utility to combine multiple images horizontally, vertically, or in a custom grid. It uses the `image` crate for image I/O, `clap` for CLI argument parsing, and `anyhow` for error handling.

## Build Commands

### Build
```bash
cargo build --release
# Binary: ./target/release/imgmerge
```

### Run Development Build
```bash
cargo run -- [args]
```

### Run Tests
```bash
cargo test
```

### Run Single Test
```bash
cargo test <test_name>
```

### Run Doc Tests
```bash
cargo test --doc
```

### Run Clippy (Linting)
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Check (剧本 and Lint)
```bash
cargo check --all-targets --all-features
```

### Format Code
```bash
cargo fmt
```

### Check Formatting
```bash
cargo fmt -- --check
```

### Full Release Build with Tests
```bash
cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo build --release
```

## Code Style Guidelines

### General Conventions
- Rust edition: 2021
- Use `#![deny(warnings)]` at the crate root
- Follow standard Rust idioms and idiomatic patterns
- Keep functions focused and small

### Naming Conventions
- **Functions/variables**: `snake_case` (e.g., `combine_images`, `parse_hex_color`)
- **Types/enums/structs**: `PascalCase` (e.g., `CombineConfig`, `Layout`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_DIMENSION`)
- **Files**: `snake_case.rs` (e.g., `combiner.rs`)

### Imports Organization
Import order (per Rust standard):
1. Standard library (`std`)
2. External crates (`crate`, `extern`)
3. Local modules (`self`, `super`)

Within each group, sort alphabetically:
```rust
use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use combiner::{combine, parse_hex_color, CombineConfig, Layout};
```

### Error Handling
- Use `anyhow::Result<T>` for functions that can fail in various ways
- Use `bail!("error message")` for early returns on failure
- Use `.with_context(|| format!("Failed to..."))` for context-rich errors
- Use `anyhow::anyhow!("...")` for creating ad-hoc errors

```rust
fn load_images(inputs: &[String]) -> Result<Vec<DynamicImage>> {
    let mut images = Vec::with_capacity(inputs.len());
    for input in inputs {
        let img = image::open(input)
            .with_context(|| format!("Could not open '{}'", input))?;
        images.push(img);
    }
    Ok(images)
}
```

### CLI with Clap
- Derive `Parser` for main CLI struct
- Use `#[derive(Args, Debug)]` for argument structs
- Use `#[derive(Subcommand, Debug)]` for subcommands
- Add doc comments for CLI help text
- Use `#[arg(...)]` attributes for argument configuration

```rust
#[derive(Parser, Debug)]
#[command(name = "imgmerge", version, about = "Combine images...")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Help text shown in CLI
    Horizontal(CommonArgs),
    Vertical(CommonArgs),
    Grid(GridArgs),
}

#[derive(Args, Debug)]
struct CommonArgs {
    #[arg(required = true)]
    inputs: Vec<String>,
    #[arg(short, long)]
    output: String,
}
```

### Type Definitions
- Use `pub enum` with variants for layout types
- Use `pub struct` with named fields for configuration
- Document public fields with doc comments when appropriate

```rust
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    Horizontal,
    Vertical,
    Grid { cols: usize, rows: usize },
}

#[derive(Debug, Clone)]
pub struct CombineConfig {
    pub layout: Layout,
    pub gap: u32,
    pub bg: [u8; 4],
    pub order: Option<Vec<usize>>,
    pub cell_width: Option<u32>,
    pub cell_height: Option<u32>,
}
```

### Derives
Always derive `Debug` on types. Derive `Clone` for types that need to be cloned. Derive `Copy` for small types passed by value.

```rust
#[derive(Debug, Clone, Copy)]
pub enum Layout { ... }

#[derive(Debug, Clone)]
pub struct CombineConfig { ... }
```

### Pattern Guidelines
- Use `match` for enum handling with exhaustive arms
- Use `.unwrap_or_else(|| ...)` for smart defaults
- Use `with_capacity` when collection size is known
- Use iterator methods (`.iter().map(...).collect()`) over manual loops when clearer

### Avoid
- Unused `mut` (let the borrower checker catch this)
- Manual memory management
- `unsafe` unless absolutely necessary
- Panics in library code (return `Result` instead)

## Project Structure

```
imgmerge/
├── Cargo.toml          # Dependencies and metadata
├── src/
│   ├── main.rs         # CLI entry point, argument parsing
│   └── combiner.rs     # Core image combination logic
└── target/             # Build output
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `image` | Image I/O and pixel operations |
| `clap` | CLI argument parsing |
| `anyhow` | Ergonomic error handling |
| `glob` | File glob pattern matching |

## Testing

Add tests using `#[cfg(test)]` module:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color!("RRGGBB"), [r, g, b, 255]);
    }
}
```

Run tests with `cargo test`. Run specific tests with `cargo test <test_name>`.
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stig View is a Rust desktop application for viewing DISA Security Technical Implementation Guides (STIGs). It supports Xylok TOML and XCCDF XML/ZIP formats (v1.1 and v1.2).

## Commands

```bash
# Build
cargo build --release -p stig-view-desktop

# Run (development)
cargo run -p stig-view-desktop

# Tests (core library only — no desktop tests)
cargo test -p stig-view-core

# Build all crates
cargo build
```

## Architecture

Cargo workspace with two crates:

- **`core/`** (`stig-view-core`) — GUI-agnostic business logic, kept separate for potential future reuse (e.g. a web frontend).
- **`desktop/`** (`stig-view-desktop`) — Iced 0.14 application.

`core/stig_dep.rs` and `core/db_dep.rs` are deprecated legacy files kept temporarily. Do not extend them — new work goes in `lib.rs` and `db.rs`.

## TODO

Planned work is tracked in `TODO.md` at the repo root, organized by release version.

## Platform Notes

Currently only builds for Linux. macOS and Windows support is planned — see `TODO.md` for details.

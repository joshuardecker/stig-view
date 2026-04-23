# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stig View is a Rust desktop application for viewing DISA Security Technical Implementation Guides (STIGs). It supports Xylok packed TOML, XCCDF v1.1, CKL, and CKLB formats.

The goal when using AI to develop this application is not for AI to do it alone. Instead AI is used as a tool to improve existing code, and discuss future ideas and how to best implement them.

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
- **`desktop/`** (`stig-view-desktop`) — Iced desktop application.

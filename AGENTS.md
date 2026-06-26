# AGENTS.md — Xylok View

## What This Project Is

A fast, cross-platform desktop viewer for DISA Security Technical Implementation Guides (STIGs). Developed by Joshua Decker for Xylok, it is designed to integrate with the Xylok internal suite and serve as a modern alternative to the official DISA STIG Viewer.

## AI Role & Boundaries

AI is used as a **coding assistant**, not an autonomous developer.

**AI will help with:**
- Planning features and architecture
- Debugging and code review
- Finalizing implementations when explicitly asked
- Keeping the dependencies minimal

**AI will NOT:**
- Add, modify, or delete code unless specifically requested
- Make architectural decisions unilaterally

## Code Style Preferences

When writing or modifying Rust code in this project, follow these conventions:

### Variable Names
- **Never use one-letter variables**, even in closures passed to `.map()`, `.filter()`, `.and_then()`, etc.
- Always use explicit, descriptive names. For example:
  - `|verdict|` instead of `|v|`
  - `|cci|` instead of `|c|`
  - `|checklist|` instead of `|c|`
  - `|error|` instead of `|e|`
  - `|os_str|` instead of `|s|`

### Spacing
- Be **generous with blank lines**.
- Add a blank line after every `let` binding.
- Add a blank line after every `if` / `else` block.
- Add a blank line before every `return` / `continue` / `break`.
- Add a blank line after each match arm in a `match` expression.
- Separate logical blocks within functions with blank lines.

### Error Handling
- Use **`thiserror`** with `#[from]` for error variants.
- Avoid manual `impl From<...> for ...` blocks.
- Example: `SerdeJsonError(#[from] serde_json::Error)` instead of wrapping errors manually.

### Imports
- Order: `std` imports first, then external crates, then `crate::` imports.
- Group `std` imports with `use std::{...};` when possible.

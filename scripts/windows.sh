#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
# No --target flag: windows-latest is x86_64-pc-windows-msvc natively,
# so the binary lands in target/release/ where cargo-wix expects it.
echo "==> Building xylok-view (release)..."
cargo build --release

# ---------------------------------------------------------------------------
# Package (MSI)
# ---------------------------------------------------------------------------
echo "==> Building MSI installer..."
cargo wix --no-build --nocapture

mkdir -p target/wix
cp target/release/xylok-view.exe target/wix/xylok-view.exe

MSI=$(find target/wix -name "*.msi" | head -1)
echo "==> Done: $MSI"
echo "==> Done: target/wix/xylok-view.exe"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

REPO_DIR="$PROJECT_ROOT/flatpak-repo"
BUILD_DIR="$PROJECT_ROOT/flatpak-build"
BUNDLE="$PROJECT_ROOT/stig-view.flatpak"

echo "==> Building stig-view..."
cargo build --release -p stig-view-desktop

echo "==> Assembling Flatpak..."
flatpak run org.flatpak.Builder \
    --repo "$REPO_DIR" \
    "$BUILD_DIR" \
    flatpak_builder.yml \
    --force-clean

echo "==> Bundling Flatpak..."
flatpak build-bundle "$REPO_DIR" "$BUNDLE" io.github.joshuardecker.stig-view

echo "==> Done: $BUNDLE"

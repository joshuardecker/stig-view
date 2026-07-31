#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "==> Building xylok-view..."
cargo build --release

# ---------------------------------------------------------------------------
# Flatpak
# ---------------------------------------------------------------------------
if command -v flatpak-builder &>/dev/null; then
    REPO_DIR="$PROJECT_ROOT/flatpak-repo"
    BUILD_DIR="$PROJECT_ROOT/flatpak-build"
    BUNDLE="$PROJECT_ROOT/xylok-view.flatpak"

    echo "==> Assembling Flatpak..."
    flatpak-builder \
        --repo "$REPO_DIR" \
        "$BUILD_DIR" \
        flatpak_builder.yml \
        --force-clean

    echo "==> Bundling Flatpak..."
    flatpak build-bundle "$REPO_DIR" "$BUNDLE" io.github.joshuardecker.xylok-view

    echo "==> Done: $BUNDLE"
fi

# ---------------------------------------------------------------------------
# AppImage
# Requires: appimagetool on PATH.
# Built against the host glibc — run inside an old-glibc container (e.g.
# AlmaLinux 8 / glibc 2.28) for broad distro compatibility.
# ---------------------------------------------------------------------------
if command -v appimagetool &>/dev/null; then
    APPDIR="$PROJECT_ROOT/AppDir"
    APPIMAGE="$PROJECT_ROOT/xylok-view.AppImage"

    echo "==> Assembling AppDir..."
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR/usr/bin"

    cp target/release/xylok-view "$APPDIR/usr/bin/xylok-view"
    cp assets/io.github.joshuardecker.xylok-view.desktop "$APPDIR/io.github.joshuardecker.xylok-view.desktop"
    cp assets/logo/logo-512.png "$APPDIR/io.github.joshuardecker.xylok-view.png"

    cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE="${SELF%/*}"
exec "$HERE/usr/bin/xylok-view" "$@"
APPRUN
    chmod +x "$APPDIR/AppRun"

    echo "==> Bundling AppImage..."
    # APPIMAGE_EXTRACT_AND_RUN=1 avoids needing FUSE (required in containers).
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 appimagetool "$APPDIR" "$APPIMAGE"

    echo "==> Done: $APPIMAGE"
fi

# ---------------------------------------------------------------------------
# Debian package (.deb)
# Requires: cargo-deb (`cargo install cargo-deb`).
# Built against the host glibc — run on ubuntu-22.04 (glibc 2.35) for
# compatibility with Ubuntu 22.04+ and Debian 12+.
# ---------------------------------------------------------------------------
if command -v cargo-deb &>/dev/null; then
    echo "==> Building .deb..."
    cargo deb --strip

    echo "==> Done: target/debian/*.deb"
fi

# ---------------------------------------------------------------------------
# RPM package (.rpm)
# Requires: cargo-generate-rpm (`cargo install cargo-generate-rpm`).
# Built against the host glibc — run inside an old-glibc container (e.g.
# AlmaLinux 8 / glibc 2.28) for RHEL 8+ and Fedora compatibility.
# cargo-generate-rpm does not strip the binary, so we strip it first.
# ---------------------------------------------------------------------------
if command -v cargo-generate-rpm &>/dev/null; then
    echo "==> Stripping binary..."
    strip -s target/release/xylok-view

    echo "==> Building .rpm..."
    cargo generate-rpm

    echo "==> Done: target/generate-rpm/*.rpm"
fi

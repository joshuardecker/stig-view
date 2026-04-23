#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

VERSION=$(grep '^version' desktop/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
APP_NAME="Stig View"
APP_BUNDLE="StigView.app"
DMG="$PROJECT_ROOT/stig-view.dmg"
STAGING="$PROJECT_ROOT/dmg-staging"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
echo "==> Building stig-view $VERSION..."
cargo build --release -p stig-view-desktop

# ---------------------------------------------------------------------------
# Assemble .app bundle
# ---------------------------------------------------------------------------
echo "==> Assembling $APP_BUNDLE..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp target/release/stig-view "$APP_BUNDLE/Contents/MacOS/stig-view"

# Generate .icns from PNG using built-in macOS tools
ICONSET="$PROJECT_ROOT/StigView.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z $size $size assets/io.github.joshuardecker.stig-view.png \
        --out "$ICONSET/icon_${size}x${size}.png" &>/dev/null
    double=$((size * 2))
    sips -z $double $double assets/io.github.joshuardecker.stig-view.png \
        --out "$ICONSET/icon_${size}x${size}@2x.png" &>/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP_BUNDLE/Contents/Resources/stig-view.icns"
rm -rf "$ICONSET"

cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>stig-view</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.joshuardecker.stig-view</string>
    <key>CFBundleName</key>
    <string>Stig View</string>
    <key>CFBundleDisplayName</key>
    <string>Stig View</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>stig-view</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# ---------------------------------------------------------------------------
# Sign (optional — skipped if credentials are not set)
# ---------------------------------------------------------------------------
if [[ -n "${SIGN_IDENTITY:-}" ]]; then
    echo "==> Signing $APP_BUNDLE..."
    codesign --deep --force --sign "$SIGN_IDENTITY" "$APP_BUNDLE"
else
    echo "==> Skipping signing (SIGN_IDENTITY not set)"
fi

# ---------------------------------------------------------------------------
# Package DMG
# ---------------------------------------------------------------------------
echo "==> Creating DMG..."
rm -rf "$STAGING"
mkdir -p "$STAGING"
cp -r "$APP_BUNDLE" "$STAGING/"
ln -s /Applications "$STAGING/Applications"

hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$STAGING" \
    -ov \
    -format UDZO \
    "$DMG"

rm -rf "$STAGING" "$APP_BUNDLE"

echo "==> Done: $DMG"

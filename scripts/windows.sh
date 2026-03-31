#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# ---------------------------------------------------------------------------
# Locate the VC++ 2022 CRT merge module (.msm) bundled with Visual Studio.
# All path discovery runs inside PowerShell to keep separators clean.
# vswhere.exe is always present on Windows CI runners and VS installations.
# ---------------------------------------------------------------------------
export VCToolsRedistMSMDir=$(powershell -Command "
  \$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
  \$vs = & \$vswhere -latest -products '*' -property installationPath
  \$base = Join-Path \$vs 'VC\Redist\MSVC'
  \$ver = (Get-ChildItem \$base | Sort-Object Name -Descending | Select-Object -First 1).Name
  (Join-Path \$base \"\$ver\MergeModules\") + '\\'
" | tr -d '\r')

echo "==> VC Redist merge modules: $VCToolsRedistMSMDir"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
# No --target flag: windows-latest is x86_64-pc-windows-msvc natively,
# so the binary lands in target/release/ where cargo-wix expects it.
echo "==> Building stig-view (release)..."
cargo build --release -p stig-view-desktop

# ---------------------------------------------------------------------------
# Package (MSI)
# ---------------------------------------------------------------------------
echo "==> Building MSI installer..."
cargo wix --no-build --nocapture -p stig-view-desktop

MSI=$(find target/wix -name "*.msi" | head -1)
echo "==> MSI: $MSI"

# ---------------------------------------------------------------------------
# Sign (optional — skipped if credentials are not set)
# ---------------------------------------------------------------------------
if [[ -n "${SIGN_CERT_PATH:-}" && -n "${SIGN_CERT_PASSWORD:-}" ]]; then
    echo "==> Signing binary..."
    signtool sign \
        /f "$SIGN_CERT_PATH" \
        /p "$SIGN_CERT_PASSWORD" \
        /tr http://timestamp.digicert.com /td sha256 /fd sha256 \
        target/release/stig-view.exe

    echo "==> Signing MSI..."
    signtool sign \
        /f "$SIGN_CERT_PATH" \
        /p "$SIGN_CERT_PASSWORD" \
        /tr http://timestamp.digicert.com /td sha256 /fd sha256 \
        "$MSI"
else
    echo "==> Skipping signing (SIGN_CERT_PATH / SIGN_CERT_PASSWORD not set)"
fi

echo "==> Done: $MSI"

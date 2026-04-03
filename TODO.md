# TODO

## 0.2
- [x] Replace synchronous `std::fs::read_dir` in `desktop/src/app/async_fns.rs` with `tokio::fs::read_dir` so the directory scan task is cancellable on shutdown, fixing the app freeze when quitting during a folder load.
- [x] Add a settings menu, initially supporting theme switching only.
- [x] Add error notification display to the GUI (`ErrNotif` state is already set but never rendered).
- [x] Add window click and drag resizing functionality.
- [x] Remove legacy `/desktop` crate once `/iced` is fully stable and ready to replace it.
- [x] Fix cancelling the file/folder picker incorrectly triggering an error notification.
- [x] Show both the filter icon and bookmark icon simultaneously in the STIG list, rather than replacing the bookmark with the filter icon when a STIG is matched.
- [x] Persist user settings (e.g. theme) between sessions using a `config.toml` stored in `{config_dir}/stig-view/`. Use `dirs::config_dir()` for cross-platform and Flatpak-compatible path resolution. Fall back to defaults on first launch and write the config on save.
- [x] Implement the settings menu (see separate settings menu item above), then remove the `todo!()` panic in `Popup::Settings` match arm in `desktop/src/ui/mod.rs`.

## 0.3
- [x] Add format support, for XccdfV1_1, Xylok, CKL, and CKLB.
- [x] CKL and CKLB files can contain multiple benchmarks — currently only the first is loaded. Update `load_ckl()` and `CKLB::convert()` to return `Vec<Benchmark>` and handle multi-benchmark files.
- [x] Modify UI to show all fields of the new Benchmark type and CKL status.
- [x] After parsing a benchmark, cache it to disk using `rmp-serde` (MessagePack) + `zstd` compression so subsequent loads skip re-parsing the source format. Cache files live in `{cache_dir}/stig-view/`.
- [x] Add a `Popup::SaveBenchmark` variant that prompts the user to save a compressed local copy of a just-opened XCCDF/ZIP benchmark. This lets the user retain a fast-loading copy independent of the original DISA file.
- [x] When a filter is applied, automatically switch the content pane to the first matching result if the currently displayed STIG does not match.
- [x] Move the Flatpak build out of the CI/CD pipeline into `scripts/build-linux.sh`. CI only needs to call the script.

## 0.4 - Animation Support
- [x] Set up a clean pattern for composing multiple time subscriptions in `subscription()`.
- [x] Add fade-in/fade-out transition animations when switching the displayed STIG.
## 0.5 — Windows Support

### Installer
- [x] Build a WiX/MSI installer. The installer must bundle the Visual C++ Redistributable (`vcruntime140.dll`) to resolve the missing runtime error on clean Windows installs.

### Console Window
- [x] Add `#![windows_subsystem = "windows"]` to `main.rs` to suppress the blank terminal window that appears when launching the application on Windows.

### App Icon
- [x] Embed a `.ico` file as a Windows resource via `build.rs`. Without this, the app shows a generic Windows icon in Explorer, the taskbar, and the UAC elevation prompt. Prefer writing the logic directly in `build.rs` (~40 lines: locate `rc.exe`, write a `.rc` resource script, invoke it via `std::process::Command`, emit `cargo:rustc-link-lib`) to avoid a dependency. If that proves brittle, `embed-resource` is the better-maintained crate alternative (`winres` has not been updated in 5 years).

### Code Signing
- [x] If SignPath Foundation is unavailable or insufficient, use **Azure Trusted Signing** (~$10/month) as the next option.

### Validation
- [x] Verify that file/folder path handling works correctly on Windows — confirm no assumptions about `/` separators in path display or regex logic.

## 0.6 — macOS Support

### Prerequisites
- [ ] Enroll in the **Apple Developer Program** ($99/year, standard tier) to obtain Developer ID certificates and notarization access. Create a **Developer ID Application** certificate in the developer portal, export it as a `.p12` file from Keychain Access (includes the private key) for CI use, and generate an **app-specific password** at appleid.apple.com for `notarytool`.

### App Bundle
- [ ] Add a `macos/` directory at the repo root containing `Info.plist` and a minimal `entitlements.plist` (empty `<dict/>` — a pure Rust file-reading app needs no special entitlements; add entries only if notarization rejects with a specific reason). Required `Info.plist` keys: `CFBundleIdentifier` (reverse-DNS), `CFBundleName`, `CFBundleVersion` (integer), `CFBundleShortVersionString` (semver), `CFBundlePackageType` (`APPL`), `CFBundleExecutable`, `NSHighResolutionCapable` (`true`), `LSMinimumSystemVersion`.
- [ ] Create an `.icns` file (generate from a 1024×1024 PNG using `iconutil`) and place it in `Contents/Resources/`. The in-app icon already set in code controls the window title bar at runtime; the `.icns` is a separate thing used by Finder, the Dock, and Spotlight and must be provided in the bundle.

### Build, Signing, and Notarization
- [ ] Produce separate binaries for each architecture: `cargo build --release --target aarch64-apple-darwin` (Apple Silicon) and `cargo build --release --target x86_64-apple-darwin` (Intel).
- [ ] Sign the `.app` with: `codesign --sign "Developer ID Application: ..." --options runtime --entitlements macos/entitlements.plist --timestamp --force StigView.app`. Both `--options runtime` (hardened runtime) and `--timestamp` are required for notarization. If the bundle ever gains nested dylibs (check with `otool -L`), sign each one inside-out before signing the `.app` — do not use `--deep`, it is unreliable.
- [ ] Notarize by zipping (`ditto -c -k --keepParent StigView.app StigView.zip`), submitting (`xcrun notarytool submit StigView.zip --apple-id ... --team-id ... --wait` — typically completes in under 5 minutes), then stapling the ticket (`xcrun stapler staple StigView.app`) so Gatekeeper can verify offline in air-gapped environments.

### Packaging (DMG)
- [ ] Package as a **DMG** for self-service install (`hdiutil create` from the stapled `.app`, sign with the Application cert, notarize, staple). Produce one DMG per architecture.

### Build Script
- [ ] Create `scripts/build-macos.sh` to handle all post-compilation steps for both architectures: assemble `.app` bundles, sign, notarize, staple, and package DMGs. The script reads signing credentials from environment variables. CI only needs to set those variables and call the script.

### CI/CD
- [ ] Add a `macos-15` job to the GitHub Actions workflow that sets signing credentials as environment variables and calls `scripts/build-macos.sh`.

## Backlog

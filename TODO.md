# TODO

## 0.2
- [x] Replace synchronous `std::fs::read_dir` in `iced/src/app/async_fns.rs` with `tokio::fs::read_dir` so the directory scan task is cancellable on shutdown, fixing the app freeze when quitting during a folder load.
- [x] Add a settings menu, initially supporting theme switching only.
- [x] Add error notification display to the GUI (`ErrNotif` state is already set but never rendered).
- [x] Add window click and drag resizing functionality.
- [ ] Remove legacy `/desktop` crate once `/iced` is fully stable and ready to replace it.
- [x] Fix cancelling the file/folder picker incorrectly triggering an error notification.
- [x] When a filter is applied, automatically switch the content pane to the first matching result if the currently displayed STIG does not match.
- [x] Show both the filter icon and bookmark icon simultaneously in the STIG list, rather than replacing the bookmark with the filter icon when a STIG is matched.
- [x] Persist user settings (e.g. theme) between sessions using a `config.toml` stored in `{config_dir}/stig-view/`. Use `dirs::config_dir()` for cross-platform and Flatpak-compatible path resolution. Fall back to defaults on first launch and write the config on save.
- [x] Implement the settings menu (see separate settings menu item above), then remove the `todo!()` panic in `Popup::Settings` match arm in `iced/src/ui/mod.rs`.

## 0.3
- [ ] Refactor the STIG loading pipeline in `/core` — decouple `Stig` parsing from the Xylok format so multiple parsers can feed into the same pipeline cleanly before a second format is added.
- [ ] Add file format detection when loading files/folders so the app can route each file to the correct parser. Currently all files are silently skipped if they fail the Xylok regex, which breaks once a second format exists.
- [ ] Add support in `/core` for loading STIGs downloaded directly from the DISA website, in addition to the existing Xylok internal format.
- [ ] Set up a clean pattern for composing multiple time subscriptions in `subscription()` before animations and loading indicators are both active simultaneously.
- [ ] Add a loading indicator (spinner) when a folder is being loaded or a filter is being processed. Drive via a time subscription active only while loading, using an `is_loading` flag in app state.
- [ ] Add fade-in/fade-out transition animations wherever content changes — switching the displayed STIG, popups appearing, and elements loading. Drive via a time subscription and a transition state machine (e.g. `FadingOut`, `FadingIn`, `Idle`) with an `f32` opacity value applied through widget styles.
- [ ] Move the Flatpak build out of the CI/CD pipeline into `scripts/build-linux.sh`. CI only needs to call the script.

## 0.4 — Windows Support

### Installer
- [ ] Build a WiX/MSI installer. MSI is the expected format for the DISA/enterprise audience and integrates cleanly with enterprise deployment tooling. The installer must bundle the Visual C++ Redistributable (`vcruntime140.dll`) to resolve the missing runtime error on clean Windows installs.

### Console Window
- [ ] Add `#![windows_subsystem = "windows"]` to `main.rs` to suppress the blank terminal window that appears when launching the application on Windows.

### App Icon
- [ ] Embed a `.ico` file as a Windows resource via `build.rs`. Without this, the app shows a generic Windows icon in Explorer, the taskbar, and the UAC elevation prompt. Prefer writing the logic directly in `build.rs` (~40 lines: locate `rc.exe`, write a `.rc` resource script, invoke it via `std::process::Command`, emit `cargo:rustc-link-lib`) to avoid a dependency. If that proves brittle, `embed-resource` is the better-maintained crate alternative (`winres` has not been updated in 5 years).

### Code Signing
- [ ] Investigate **SignPath Foundation** (free for open-source projects on GitHub) as the primary signing path before committing to a paid CA.
- [ ] If SignPath Foundation is unavailable or insufficient, use **Azure Trusted Signing** (~$10/month) as the next option — Microsoft controls both the CA and SmartScreen, ephemeral certs eliminate private key management, and it has native GitHub Actions support. Requires an Azure subscription and identity verification through Microsoft Partner Center.
- [ ] Do not pursue an EV certificate — as of 2024, EV no longer bypasses SmartScreen and the price premium is not worth it. OV is equivalent for SmartScreen purposes.

### SmartScreen Reputation
- [ ] On first release, submit the signed installer to the [Microsoft malware submission portal](https://www.microsoft.com/en-us/wdsi/filesubmission) for review to accelerate SmartScreen clearance. Downloads via browser (from your website or tooling) will trigger SmartScreen until the file accumulates enough download history.
- [ ] Include a note in Windows release instructions explaining that SmartScreen may show a warning on first install ("Click 'More info → Run anyway'") and that this is expected behavior for a new release.

### Build Script
- [ ] Create `scripts/build-windows.sh` to handle all post-compilation steps: sign the binary with `signtool.exe`, build the WiX MSI installer, and sign the installer. The script reads signing credentials from environment variables. CI only needs to set those variables and call the script.

### Validation
- [ ] Verify that file/folder path handling works correctly on Windows — confirm no assumptions about `/` separators in path display or regex logic.

## 0.5 — macOS Support

### Prerequisites
- [ ] Enroll in the **Apple Developer Program** ($99/year, standard tier) to obtain Developer ID certificates and notarization access. Create a **Developer ID Application** certificate in the developer portal, export it as a `.p12` file from Keychain Access (includes the private key) for CI use, and generate an **app-specific password** at appleid.apple.com for `notarytool`.

### App Bundle
- [ ] Add a `macos/` directory at the repo root containing `Info.plist` and a minimal `entitlements.plist` (empty `<dict/>` — a pure Rust file-reading app needs no special entitlements; add entries only if notarization rejects with a specific reason). Required `Info.plist` keys: `CFBundleIdentifier` (reverse-DNS), `CFBundleName`, `CFBundleVersion` (integer), `CFBundleShortVersionString` (semver), `CFBundlePackageType` (`APPL`), `CFBundleExecutable`, `NSHighResolutionCapable` (`true`), `LSMinimumSystemVersion`.
- [ ] Create an `.icns` file (generate from a 1024×1024 PNG using `iconutil`) and place it in `Contents/Resources/`. The in-app icon already set in code controls the window title bar at runtime; the `.icns` is a separate thing used by Finder, the Dock, and Spotlight and must be provided in the bundle.

### Build, Signing, and Notarization
- [ ] Produce separate binaries for each architecture: `cargo build --release --target aarch64-apple-darwin` (Apple Silicon) and `cargo build --release --target x86_64-apple-darwin` (Intel), releasing both as distinct artifacts. Both targets can be built on a single `macos-15` runner via cross-compilation.
- [ ] Sign the `.app` with: `codesign --sign "Developer ID Application: ..." --options runtime --entitlements macos/entitlements.plist --timestamp --force StigView.app`. Both `--options runtime` (hardened runtime) and `--timestamp` are required for notarization. If the bundle ever gains nested dylibs (check with `otool -L`), sign each one inside-out before signing the `.app` — do not use `--deep`, it is unreliable.
- [ ] Notarize by zipping (`ditto -c -k --keepParent StigView.app StigView.zip`), submitting (`xcrun notarytool submit StigView.zip --apple-id ... --team-id ... --wait` — typically completes in under 5 minutes), then stapling the ticket (`xcrun stapler staple StigView.app`) so Gatekeeper can verify offline in air-gapped environments. On failure, inspect the JSON log: `xcrun notarytool log <submission-id>`.

### Packaging (DMG)
- [ ] Package as a **DMG** for self-service install (`hdiutil create` from the stapled `.app`, sign with the Application cert, notarize, staple). Produce one DMG per architecture.

### Build Script
- [ ] Create `scripts/build-macos.sh` to handle all post-compilation steps for both architectures: assemble `.app` bundles, sign, notarize, staple, and package DMGs. The script reads signing credentials from environment variables. CI only needs to set those variables and call the script.

### CI/CD
- [ ] Add a `macos-15` job to the GitHub Actions workflow that sets signing credentials as environment variables and calls `scripts/build-macos.sh`.

## Backlog
- [ ] Write hand-crafted tests for all of the below using known good/bad cases and real sample files as fixtures before reaching for fuzz testing. Add fuzz testing for `Stig::from_xylok_txt()` using `cargo-fuzz` as a one-time hardening step before a release, to catch panics and pathological regex behavior on arbitrary input.
- [ ] Expand unit tests for `core/src/stig.rs` — cover valid Xylok files, files with missing fields, non-Xylok files returning `None`, and edge cases like empty fields or unusual whitespace.
- [ ] Add unit tests for `core/src/db.rs` — verify that `insert` and `clean` keep the `std::sync::RwLock` cache consistent with the underlying tokio `RwLock` data.
- [ ] Add unit tests for `iced/src/app/command.rs` — cover each valid command keyword, invalid input returning the correct error, and regex errors in search terms being handled gracefully.
- [ ] Benchmark folder load performance on large directories to inform loading indicator design.

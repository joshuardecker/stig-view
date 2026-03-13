# TODO

## 0.2
- [ ] Replace synchronous `std::fs::read_dir` in `iced/src/app/async_fns.rs` with `tokio::fs::read_dir` so the directory scan task is cancellable on shutdown, fixing the app freeze when quitting during a folder load.
- [ ] Add a settings menu, initially supporting theme switching only.
- [ ] Add error notification display to the GUI (`ErrNotif` state is already set but never rendered).
- [ ] Add window click and drag resizing functionality.
- [ ] Remove legacy `/desktop` crate once `/iced` is fully stable and ready to replace it.
- [ ] Fix cancelling the file/folder picker incorrectly triggering an error notification.
- [ ] When a filter is applied, automatically switch the content pane to the first matching result if the currently displayed STIG does not match.
- [x] Show both the filter icon and bookmark icon simultaneously in the STIG list, rather than replacing the bookmark with the filter icon when a STIG is matched.
- [ ] Persist user settings (e.g. theme) between sessions using a `config.toml` stored in `{config_dir}/stig-view/`. Use `dirs::config_dir()` for cross-platform and Flatpak-compatible path resolution. Fall back to defaults on first launch and write the config on save.
- [ ] Implement the settings menu (see separate settings menu item above), then remove the `todo!()` panic in `Popup::Settings` match arm in `iced/src/ui/mod.rs`.

## 0.3
- [ ] Add support in `/core` for loading STIGs downloaded directly from the DISA website, in addition to the existing Xylok internal format.
- [ ] Refactor the STIG loading pipeline in `/core` — decouple `Stig` parsing from the Xylok format so multiple parsers can feed into the same pipeline cleanly before a second format is added.
- [ ] Add file format detection when loading files/folders so the app can route each file to the correct parser. Currently all files are silently skipped if they fail the Xylok regex, which breaks once a second format exists.
- [ ] Add a loading indicator (spinner) when a folder is being loaded or a filter is being processed. Drive via a time subscription active only while loading, using an `is_loading` flag in app state.
- [ ] Add fade-in/fade-out transition animations wherever content changes — switching the displayed STIG, popups appearing, and elements loading. Drive via a time subscription and a transition state machine (e.g. `FadingOut`, `FadingIn`, `Idle`) with an `f32` opacity value applied through widget styles.
- [ ] Set up a clean pattern for composing multiple time subscriptions in `subscription()` before animations and loading indicators are both active simultaneously.

## Backlog
- [ ] Benchmark folder load performance on large directories to inform loading indicator design.

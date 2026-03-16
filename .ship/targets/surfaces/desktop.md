+++
title = "Desktop"
owners = ["apps/desktop/"]
profile_hint = "rust-compiler"
status = "frozen"
+++

# Desktop

Tauri-based native app. Currently frozen — web-first is the active strategy. Will return when Studio has a stable foundation worth wrapping.

## When to Unfreeze
- Studio UI is polished and stable
- Account + profile sync is working
- There's a clear native-only capability (system tray, file watching, deeper OS integration)

## Aspirational (when active)
- [ ] Native installer (dmg, msi, deb)
- [ ] System tray — quick profile switch without opening browser
- [ ] File watcher — auto-recompile on `.ship/` change
- [ ] Offline-first — full compiler runs locally with no network
- [ ] Deep OS integration — keychain for credentials, native file pickers

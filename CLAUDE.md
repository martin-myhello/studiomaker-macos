# StudioMaker Native — Claude Code Guide

## IMPORTANT: Read architecture first
**At the start of every prompt, read `/Users/home/Documents/Repositories/Studio/architecture.md`** — it defines the entire system: data model, sync strategy, client architecture, and technology decisions.

## Architecture

This is a **Tauri v2** native shell. The frontend is bundled locally (Vite + React). Data comes from local SQLite via `tauri-plugin-sql`, synced to Supabase Postgres via PowerSync.

## Key Decisions

- **Local-first**: SQLite is the source of truth. All reads are local SQL queries — no network dependency.
- **Bundled frontend**: `frontendDist` points to the local Vite build. Web deploys do NOT update the native app.
- **macOS private API**: Used for `NSColor` background customization via the `cocoa` crate.
- **Capabilities system**: Permissions for IPC are in `capabilities/default.json`. The `remote.urls` field allows the web app at `*.studiomaker.app` to call Tauri commands.
- **PencilKit plugin**: Custom Tauri plugin at `plugins/pencilkit/`. Uses `@_cdecl("init_plugin_pencilkit")` Swift entry point. No-ops on macOS, forwards to Swift on iOS.

## Common Tasks

### Adding a new IPC command

1. Add the Rust command in `src/lib.rs`
2. Register it in `.invoke_handler(tauri::generate_handler![...])`
3. Add JS wrapper in `studio/src/lib/tauri.ts`
4. If needed, add permission in `capabilities/default.json`

### Adding a new menu item

1. Edit `src/menu.rs` — add to the appropriate submenu
2. Add handler in the `menu_event` match block
3. Menu events reach the web app via `emit_menu_action()` → Tauri event `menu-action`

### Updating the auto-updater endpoint

The updater checks `plugins.updater.endpoints` in `tauri.conf.json`. The `latest.json` file is uploaded to GitHub Releases by the CI workflow.

## Build Commands

```bash
cargo check                                    # Type-check
cargo tauri dev                                # Dev mode (macOS)
cargo tauri build --target universal-apple-darwin  # Release (macOS)
cargo tauri ios dev                            # iOS simulator
```

## File Quick Reference

| File | Purpose |
|------|---------|
| `tauri.conf.json` | Window config, bundle settings, plugin config |
| `src/lib.rs` | App entry, commands, window setup |
| `src/menu.rs` | macOS menu bar + keyboard shortcuts |
| `src/updater.rs` | Background update checker |
| `plugins/pencilkit/` | iPad PencilKit overlay plugin |
| `capabilities/default.json` | IPC permissions for remote URLs |
| `gen/apple/project.yml` | Xcode project config (iOS) |

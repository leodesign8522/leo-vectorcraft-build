# Leo Design VectorCraft → official v0.18.0 migration

Status: **Compatibility work not completed. Do not distribute this branch as an enhanced v0.18.0 build.**

## Existing safe baseline
- Repository main: Leo Design VectorCraft Enhanced v0.5.7 (Windows x64) built on official VectorCraft v0.17.0.
- Successful build: https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/40899904329
- Keep the main branch untouched until the v0.18 port builds and passes smoke tests.

## Verified upstream breaking changes
- Official v0.18.0 upgraded to egui 0.37.
- `crates/ui-egui/src/floating.rs` no longer exists in v0.18.0.
- `crates/ui-egui/src/panels/mod.rs` no longer exists in v0.18.0.
- `crates/ui-egui/src/dock.rs` is substantially rewritten.
- `crates/ui-egui/src/menus.rs` is now a smaller module entrypoint.
- `crates/ui-egui/src/panel_docking.rs` changed significantly, including OS floating windows.
- Current patch scripts anchored to v0.17.0 cannot safely be applied unchanged.

## Migration order
1. Establish an isolated official v0.18.0 Windows baseline; keep stable main v0.17.0 build intact.
2. Port core Smart Select and Group Anchor Select logic.
3. Port color settings / ICC persistence and CJK PDF font mapping.
4. Reintegrate Image Pro Export and sidebar using new UI modules.
5. Rebuild Workspace Pro / per-panel resize using v0.18's new shared docking & native floating windows; avoid replacing upstream capabilities.
6. Port Ctrl+Enter text finish and Chinese paragraph sizing.
7. Port Units and Document Color quick panels.
8. Port workspace and shortcut JSON import/export; audit updated workspace schema.
9. Compile and test native Windows x64; test Illustrator clipboard, mixed Chinese/Latin text, CMYK export, workspace save/restore and resizing.
10. Only then promote to stable main.

## User preservation procedure
- Close VectorCraft before copying profile data.
- Portable build: save the whole `VectorCraftData` folder (especially `ui.json`, ICC profiles) beside `portable.txt` and `vectorcraft.exe`.
- Installed build: profile normally resides at `%APPDATA%\\VectorCraft\\ui.json`.
- v0.5.7 supports separate workspace JSON export/import and shortcut JSON export/import.
- Keep old and new executable copies in **different folders**; don't move the new official EXE on top of the Leo Design enhanced EXE.
- The new v0.18 executable must not be treated as compatible with old workspace layout JSON without testing; use exported JSON as a backup.

## Upstream references
- https://github.com/storytold/vectorcraft/releases/tag/v0.18.0
- https://github.com/storytold/vectorcraft/compare/v0.17.0...v0.18.0

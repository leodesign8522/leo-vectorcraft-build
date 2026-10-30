#!/usr/bin/env python3
"""Port the first Leo Design Core Bridge module to VectorCraft v0.18.0.

Kept intentionally small. Break on unknown upstream changes instead of
producing a silently broken Windows EXE.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_core_bridge.py <vectorcraft-v018-source>")
root = Path(sys.argv[1])
module = Path(__file__).resolve().parent / "leo_core_bridge"

def replace(rel, before, after, label):
    file = root / rel
    text = file.read_text(encoding="utf-8")
    count = text.count(before)
    if count != 1:
        raise SystemExit(f"FAIL {label}: {rel}, anchors={count}")
    file.write_text(text.replace(before, after, 1), encoding="utf-8", newline="\n")
    print("OK", label)

replace(
    "crates/ui-egui/src/lib.rs",
    "pub mod workspaces;\n",
    "pub mod workspaces;\npub mod leo_core_bridge;\n",
    "register Leo Design Core Bridge"
)

dest = root / "crates/ui-egui/src/leo_core_bridge"
if dest.exists():
    raise SystemExit("FAIL Core Bridge destination already exists")
dest.mkdir(parents=True, exist_ok=False)
for src in sorted(module.glob("*.rs")):
    (dest / src.name).write_text(src.read_text(encoding="utf-8"), encoding="utf-8", newline="\n")
    print("OK module", src.name)

replace(
    "crates/ui-egui/src/workspaces.rs",
    '    v.push(Item::Cmd("Manage Workspaces…", "window.workspace.manage", Value::Null));\n',
    '''    v.push(Item::Cmd("Manage Workspaces…", "window.workspace.manage", Value::Null));
    v.push(Item::Sep);
    v.push(Item::Cmd("Leo Design · 导出当前工作区…", "window.workspace.export", Value::Null));
    v.push(Item::Cmd("Leo Design · 导入工作区…", "window.workspace.import", Value::Null));
''',
    "add workspace actions to native Window menu and workspace switcher"
)
replace(
    "crates/ui-egui/src/workspaces.rs",
    '        "window.workspace.list" => Ok(json!({"current": app.ui.workspace, "workspaces": names(&app.ui), "custom": app.ui.custom_workspaces})),',
    '''        "window.workspace.list" => Ok(json!({"current": app.ui.workspace, "workspaces": names(&app.ui), "custom": app.ui.custom_workspaces})),
        "window.workspace.export" => crate::leo_core_bridge::workspace_io::export(app, p),
        "window.workspace.import" => crate::leo_core_bridge::workspace_io::import(app, p),''',
    "wire workspace I/O through native UI command dispatcher"
)
replace(
    "crates/ui-egui/src/menus/ui_commands.rs",
    '    ("window.workspace.list", "List Workspaces", "", "{}"),',
    '''    ("window.workspace.list", "List Workspaces", "", "{}"),
    ("window.workspace.export", "Leo Design Export Workspace", "", "{name?, path?}"),
    ("window.workspace.import", "Leo Design Import Workspace", "", "{path? | data?}"),''',
    "register workspace commands for keyboard shortcut editor"
)
replace(
    "crates/ui-egui/src/menus/tree.rs",
    '''                c("Keyboard Shortcuts…", "edit.keyboardShortcuts"),
                c("Preferences…", "edit.preferences"),''',
    '''                c("Keyboard Shortcuts…", "edit.keyboardShortcuts"),
                sub("Leo Design · 快捷键备份", vec![
                    c("导出快捷键 JSON…", "shortcuts.export"),
                    c("导入快捷键 JSON / KYS…", "shortcuts.import"),
                ]),
                c("Preferences…", "edit.preferences"),''',
    "expose official shortcut JSON/KYS I/O as a convenient menu"
)
print("SUCCESS Leo Design Core Bridge / Workspace & Shortcut IO for v0.18")

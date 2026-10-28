#!/usr/bin/env python3
"""Turn Leo Design Image Pro Export into a right-dock icon panel for VectorCraft v0.17.0.

Run AFTER the existing patch_image_pro_export.py. Uses existing native
renderer/export pipeline, and retains the old modal implementation as fallback.
"""
from pathlib import Path
import sys

if len(sys.argv) != 3:
    raise SystemExit("Usage: patch_image_export_sidebar.py VECTORCRAFT_ROOT SIDEBAR_RUST_SOURCE")
root = Path(sys.argv[1])
sidebar = Path(sys.argv[2])


def replace_once(path: Path, before: str, after: str, label: str):
    content = path.read_text(encoding="utf-8")
    found = content.count(before)
    if found != 1:
        raise SystemExit(f"FAIL {label}: expected one anchor, found {found}")
    path.write_text(content.replace(before, after, 1), encoding="utf-8", newline="\n")


state = root / "crates/ui-egui/src/state.rs"
replace_once(
    state,
    '    ("artboards", "Artboards", "dc-artboards"),\n',
    '    ("artboards", "Artboards", "dc-artboards"),\n'
    '    ("leoImageExport", "Leo Design Image Pro Export", "image"),\n',
    "Register right dock Leo Design export panel",
)
replace_once(
    state,
    '    &["artboards"],\n',
    '    &["artboards", "leoImageExport"],\n',
    "Display Leo Design icon alongside artboard tools",
)

panels = root / "crates/ui-egui/src/panels/mod.rs"
replace_once(
    panels,
    '        "artboards" => artboards::show(app, ui),\n',
    '        "artboards" => artboards::show(app, ui),\n'
    '        "leoImageExport" => crate::dialogs::show_leo_export_sidebar(app, ui),\n',
    "Draw live export sidebar content",
)

dialogs = root / "crates/ui-egui/src/dialogs/mod.rs"
replace_once(
    dialogs,
    'pub use leo_image_export::open as open_leo_image_export;\n',
    'pub use leo_image_export::open as open_leo_image_export;\n'
    'pub use leo_image_export::show_sidebar as show_leo_export_sidebar;\n',
    "Export sidebar drawing function",
)

menus = root / "crates/ui-egui/src/menus.rs"
replace_once(
    menus,
    '''        "file.leoImageProExport" => {
            crate::dialogs::open_leo_image_export(app);
            Ok(Value::Null)
        }''',
    '''        "file.leoImageProExport" => {
            // Non-modal dockable sidebar instead of the old popup dialog.
            app.run("window.panel", json!({"panel": "leoImageExport"}))
        }''',
    "Open dock from the File Export menu",
)
replace_once(
    menus,
    '                panel("Asset Export", crate::panels::asset_export::ID),\n',
    '                panel("Asset Export", crate::panels::asset_export::ID),\n'
    '                panel("Leo Design Image Pro Export", "leoImageExport"),\n',
    "Add a Window menu entry for toggling the panel",
)

dialog = root / "crates/ui-egui/src/dialogs/leo_image_export.rs"
if "pub fn show_sidebar" in dialog.read_text(encoding="utf-8"):
    raise SystemExit("Refusing to append Leo Design sidebar twice")
dialog.write_text(dialog.read_text(encoding="utf-8") + "\n" + sidebar.read_text(encoding="utf-8"), encoding="utf-8", newline="\n")

print("SUCCESS: Leo Design Image Pro Export is now a dockable, non-modal right sidebar.")

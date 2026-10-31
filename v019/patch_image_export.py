#!/usr/bin/env python3
"""Leo Design Image Pro Export for VectorCraft v0.19.0.

Restores the v0.5.x nonmodal right dock panel using the native upstream
PNG/JPEG/ICC export services, while keeping the export dialog fallback module.
All hooks are fail-closed against unexpected upstream changes.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_image_export.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])
sources = Path(__file__).resolve().parents[1] / "scripts"

def patch(path, before, after, label):
    p = root / path
    s = p.read_text(encoding="utf-8")
    n = s.count(before)
    if n != 1:
        raise SystemExit(f"FAIL {label}: anchor {n} in {p}")
    p.write_text(s.replace(before, after, 1), encoding="utf-8", newline="\n")
    print("OK", label)

patch(
    "crates/ui-egui/src/dialogs.rs",
    "mod export_for_screens;\n",
    "mod export_for_screens;\nmod leo_image_export;\n",
    "register Image Pro Export module"
)
patch(
    "crates/ui-egui/src/dialogs.rs",
    "pub use export_for_screens::open as open_export_for_screens;\n",
    """pub use export_for_screens::open as open_export_for_screens;
pub use leo_image_export::show_sidebar as show_leo_image_export_sidebar;
""",
    "expose Image Pro Export panel"
)
patch(
    "crates/ui-egui/src/dialogs.rs",
    '    ExportForScreens: ["exportForScreens"] => export_for_screens::SPEC,\n',
    '''    ExportForScreens: ["exportForScreens"] => export_for_screens::SPEC,
    LeoImageExport: [leo_image_export::KIND] => leo_image_export::SPEC,
''',
    "register Image Pro Export dialog fallback"
)
target = root / "crates/ui-egui/src/dialogs/leo_image_export.rs"
if target.exists():
    raise SystemExit("Leo Image Export already installed")
target.write_text(
    (sources / "leo_image_export.rs").read_text(encoding="utf-8")
    + "\n" + (sources / "leo_image_sidebar.rs").read_text(encoding="utf-8"),
    encoding="utf-8", newline="\n",
)

patch(
    "crates/ui-egui/src/state.rs",
    '    ("artboards", "Artboards", "dc-artboards"),\n',
    '''    ("artboards", "Artboards", "dc-artboards"),
    ("leoImageExport", "Leo Design Image Pro Export", "image"),
''',
    "register right-hand Image Pro Export icon"
)
patch(
    "crates/ui-egui/src/state.rs",
    '    &["artboards"],\n',
    '    &["artboards", "leoImageExport"],\n',
    "make Image Pro Export available in default right rail"
)
patch(
    "crates/ui-egui/src/panels.rs",
    '        "artboards" => artboards::show(app, ui),\n',
    '''        "artboards" => artboards::show(app, ui),
        "leoImageExport" => crate::dialogs::show_leo_image_export_sidebar(app, ui),
''',
    "render native export sidebar"
)
patch(
    "crates/ui-egui/src/menus/ui_commands.rs",
    '        "file.exportForScreens",\n        "Export for Screens…",',
    '        "file.exportForScreens",\n        "Export for Screens…",',
    "confirm export command metadata present"
)
patch(
    "crates/ui-egui/src/menus/ui_commands.rs",
    '    ("shortcuts.export", "Export Keyboard Shortcuts…", "", "{path?}"),',
    '''    ("shortcuts.export", "Export Keyboard Shortcuts…", "", "{path?}"),
    ("file.leoImageProExport", "Leo Design 图像专业导出", "", "{}"),
''',
    "register Image Pro Export command"
)
patch(
    "crates/ui-egui/src/menus/run.rs",
    '        "file.exportForScreens" if p.as_object().is_none_or(|o| o.is_empty()) => {',
    '''        "file.leoImageProExport" => app.run("window.panel", json!({"panel": "leoImageExport"})),
        "file.exportForScreens" if p.as_object().is_none_or(|o| o.is_empty()) => {''',
    "wire Image Pro Export menu"
)
patch(
    "crates/ui-egui/src/menus/state.rs",
    '        | "file.exportForScreens"\n',
    '        | "file.exportForScreens"\n        | "file.leoImageProExport"\n',
    "enable Image Pro Export only for an open document"
)
patch(
    "crates/ui-egui/src/menus/tree.rs",
    '                        c("Export for Screens…", "file.exportForScreens"),\n',
    '''                        c("Leo Design · 图像专业导出…", "file.leoImageProExport"),
                        c("Export for Screens…", "file.exportForScreens"),
''',
    "add Image Pro Export to File > Export"
)
patch(
    "crates/ui-egui/src/menus/tree.rs",
    '                panel("Asset Export", crate::panels::asset_export::ID),\n',
    '''                panel("Asset Export", crate::panels::asset_export::ID),
                panel("Leo Design Image Pro Export", "leoImageExport"),
''',
    "add export panel to Window menu"
)
print("SUCCESS Leo Design Image Pro Export v0.19 right-side panel")

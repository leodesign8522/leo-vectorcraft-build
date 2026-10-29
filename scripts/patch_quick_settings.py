#!/usr/bin/env python3
"""Add Leo Design Units and Document Color Mode right-sidebar icons, v0.5.5."""
from pathlib import Path
import sys
if len(sys.argv)!=3: raise SystemExit("Usage: patch_quick_settings.py ROOT RUST_FILE")
root=Path(sys.argv[1]); rust=Path(sys.argv[2]).read_text(encoding="utf-8")
def rep(path,old,new):
 p=root/path;s=p.read_text(encoding="utf-8")
 if s.count(old)!=1:raise SystemExit(f"FAIL {path}: {s.count(old)} anchors")
 p.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
rep("crates/ui-egui/src/state.rs",
'    ("leoImageExport", "Leo Design Image Pro Export", "image"),',
'    ("leoImageExport", "Leo Design Image Pro Export", "image"),\n    ("leoQuickUnits", "Leo Design 单位", "ruler"),\n    ("leoQuickDocColor", "Leo Design 文档颜色模式", "palette"),')
rep("crates/ui-egui/src/state.rs",
'    &["artboards", "leoImageExport"],',
'    &["artboards", "leoImageExport", "leoQuickUnits", "leoQuickDocColor"],')
rep("crates/ui-egui/src/panels/mod.rs",
'pub mod color;',
'pub mod color;\npub mod leo_quick_settings;')
rep("crates/ui-egui/src/panels/mod.rs",
'        "leoImageExport" => crate::dialogs::show_leo_export_sidebar(app, ui),',
'        "leoImageExport" => crate::dialogs::show_leo_export_sidebar(app, ui),\n        "leoQuickUnits" => leo_quick_settings::show_units(app, ui),\n        "leoQuickDocColor" => leo_quick_settings::show_document_color(app, ui),')
rep("crates/ui-egui/src/menus.rs",
'                panel("Leo Design Image Pro Export", "leoImageExport"),',
'                panel("Leo Design Image Pro Export", "leoImageExport"),\n                panel("Leo Design 单位", "leoQuickUnits"),\n                panel("Leo Design 文档颜色模式", "leoQuickDocColor"),')
p=root/"crates/ui-egui/src/panels/leo_quick_settings.rs"
if p.exists():raise SystemExit("Target Rust module already exists")
p.write_text(rust,encoding="utf-8",newline="\n")
print("SUCCESS Leo Design two right-sidebar settings panels")

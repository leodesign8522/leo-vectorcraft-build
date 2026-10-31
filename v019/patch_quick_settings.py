#!/usr/bin/env python3
"""Restore Leo Design v0.5.7 quick settings panels on official VectorCraft v0.19.

Register native dockable Units and Document Colour Mode panels in the
v0.19 shared docking registry. Reuse previously compiled Rust UI module.
"""
from pathlib import Path
import sys

if len(sys.argv)!=2:raise SystemExit("Usage: patch_quick_settings.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])
rust=(Path(__file__).resolve().parents[1]/"scripts/leo_quick_settings.rs").read_text(encoding="utf-8")
def rep(rel,old,new,label):
 f=root/rel;s=f.read_text(encoding="utf-8")
 count=s.count(old)
 if count!=1:raise SystemExit(f"FAIL {label} {rel}: {count}")
 f.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
 print("OK",label)
rep("crates/ui-egui/src/state.rs",
 '    ("leoImageExport", "Leo Design Image Pro Export", "image"),',
 '    ("leoImageExport", "Leo Design Image Pro Export", "image"),\n    ("leoQuickUnits", "Leo Design 单位", "ruler"),\n    ("leoQuickDocColor", "Leo Design 文档颜色模式", "palette"),',
 "register Units and Colour icon panels")
rep("crates/ui-egui/src/state.rs",
 '    &["artboards", "leoImageExport"],',
 '    &["artboards", "leoImageExport", "leoQuickUnits", "leoQuickDocColor"],',
 "default dock controls")
rep("crates/ui-egui/src/panels.rs",
 'pub mod color;',
 'pub mod color;\npub mod leo_quick_settings;',
 "register quick setting panel module")
rep("crates/ui-egui/src/panels.rs",
 '        "leoImageExport" => crate::dialogs::show_leo_image_export_sidebar(app, ui),',
 '        "leoImageExport" => crate::dialogs::show_leo_image_export_sidebar(app, ui),\n        "leoQuickUnits" => leo_quick_settings::show_units(app, ui),\n        "leoQuickDocColor" => leo_quick_settings::show_document_color(app, ui),',
 "render real Units and Colour panels")
rep("crates/ui-egui/src/menus/tree.rs",
 '                panel("Leo Design Image Pro Export", "leoImageExport"),',
 '                panel("Leo Design Image Pro Export", "leoImageExport"),\n                panel("Leo Design 单位", "leoQuickUnits"),\n                panel("Leo Design 文档颜色模式", "leoQuickDocColor"),',
 "list panels in Window menu")
path=root/"crates/ui-egui/src/panels/leo_quick_settings.rs"
if path.exists():raise SystemExit("FAIL duplicate Leo Quick Settings")
path.write_text(rust,encoding="utf-8",newline="\n")
print("SUCCESS v0.19 Units + Document Colour side panels restored")

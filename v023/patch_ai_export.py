"""Add an explicit Illustrator-backed editable AI export without altering native AI saves."""
from pathlib import Path
import sys
root=Path(sys.argv[1]);here=Path(__file__).parent/'ai_export'
def edit(rel,old,new):
    p=root/rel;s=p.read_text(encoding='utf-8');assert s.count(old)==1,(rel,old[:70],s.count(old))
    p.write_text(s.replace(old,new),encoding='utf-8',newline='\n')
dest=root/'crates/ui-egui/src/leo_core_bridge'
for name in ['ai_export.rs','ai_export_tests.rs','illustrator_bridge.jsx']:
    (dest/name).write_text((here/name).read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
edit('crates/ui-egui/src/leo_core_bridge/mod.rs','pub mod ai_import;','pub mod ai_import;\npub mod ai_export;')
edit('crates/ui-egui/src/menus/tree.rs',
     '                        c("Leo Design · 图像专业导出…", "file.leoImageProExport"),',
     '                        c("导出可编辑 AI（通过 Illustrator）…", "file.leoExportEditableAi"),\n                        c("Leo Design · 图像专业导出…", "file.leoImageProExport"),')
edit('crates/ui-egui/src/menus/ui_commands.rs','pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[',
     'pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[\n    ("file.leoExportEditableAi", "导出可编辑 AI（通过 Illustrator）", "", "{path?, wait?: bool} export live point/area text with RGB/CMYK values, layers, groups and artboards through installed Windows Illustrator"),')
edit('crates/ui-egui/src/menus/run.rs','        "file.leoOpenAi" => crate::leo_core_bridge::ai_import::open(app, p),',
     '        "file.leoExportEditableAi" => crate::leo_core_bridge::ai_export::export(app, p),\n        "file.leoOpenAi" => crate::leo_core_bridge::ai_import::open(app, p),')
edit('crates/ui-egui/src/menus/state.rs','        | "file.leoImageProExport"','        | "file.leoExportEditableAi"\n        | "file.leoImageProExport"')
print('Applied editable AI export: installed Illustrator bridge, live text frames, native colour/layer snapshot')

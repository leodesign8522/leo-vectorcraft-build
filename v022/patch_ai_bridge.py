"""Install the two-way selection bridge into the existing Core Bridge recipe."""
from pathlib import Path
import sys
root=Path(sys.argv[1]);here=Path(__file__).parent/'ai_bridge'
def edit(rel,old,new):
    p=root/rel;s=p.read_text(encoding='utf-8')
    assert s.count(old)==1,(rel,old[:90],s.count(old))
    p.write_text(s.replace(old,new),encoding='utf-8',newline='\n')
dest=root/'crates/ui-egui/src/leo_core_bridge'
for name in ['bridge.rs','Send_to_VectorCraft.jsx']:
    (dest/name).write_text((here/name).read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
edit('crates/ui-egui/src/leo_core_bridge/mod.rs','pub mod ai_export;','pub mod ai_export;\npub mod bridge;')
edit('crates/ui-egui/src/app.rs','        background::poll(self);','        background::poll(self);\n        crate::leo_core_bridge::bridge::poll(self,ctx);')
edit('crates/ui-egui/src/menus/tree.rs','                c("Export Selection…", "file.exportSelection"),',
     '                c("发送选中对象到 Illustrator", "file.leoSendToIllustrator"),\n                c("Export Selection…", "file.exportSelection"),')
edit('crates/ui-egui/src/menus/run.rs','        "file.leoExportEditableAi" => crate::leo_core_bridge::ai_export::export(app, p),',
     '        "file.leoSendToIllustrator" => crate::leo_core_bridge::bridge::send(app, p),\n        "file.leoExportEditableAi" => crate::leo_core_bridge::ai_export::export(app, p),')
edit('crates/ui-egui/src/menus/ui_commands.rs','pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[',
     'pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[\n    ("file.leoSendToIllustrator", "发送选中对象到 Illustrator", "", "{wait?: bool} send selected full objects as a new editable Illustrator document; requires installed Illustrator"),')
edit('crates/ui-egui/src/menus/state.rs','        "file.exportSelection" =>',
     '        "file.leoSendToIllustrator" => cfg!(windows) && app.session.active().is_some_and(|d| !d.selection.objects.is_empty()),\n        "file.exportSelection" =>')
print('Applied Leo Design AI Bridge 0.1.0: selected-only editable send and local inbox receive')

"""Expose four real workflows in a native dockable panel, revealed once on upgrade."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
def edit(relative,a,b):
    p=root/relative
    s=p.read_text(encoding='utf-8')
    if s.count(a)!=1:raise SystemExit('Unexpected Path Tools panel anchor: '+relative+' '+a[:70])
    p.write_text(s.replace(a,b,1),encoding='utf-8',newline='\n')
edit('crates/tools/src/direct.rs','        self.mesh.options()', '''        let mut options=self.mesh.options();
        if !options.is_object() {options=json!({});}
        options["leoHandles"]=json!(self.leo_handles.iter().map(|((id,si,ai,out),p)|json!({"id":id.0,"subpath":si,"anchor":ai,"which":if *out {"out"} else {"in"},"x":p.x,"y":p.y})).collect::<Vec<_>>());
        options["leoHandleRevision"]=json!(self.leo_handle_revision);
        options''')
edit('crates/tools/src/direct.rs','    fn notify(&mut self, cx: &ToolContext, what: &str) {','''    fn notify(&mut self, cx: &ToolContext, what: &str) {
        if what=="leo.handlesRefresh" {
            self.leo_handles=self.leo_handles.iter().filter_map(|(key,_)|leo_handle_position(cx,*key).map(|p|(*key,p))).collect();
            self.leo_handle_revision=cx.revision.1;
            self.leo_handle_hover=None;
        }''')
edit('crates/ui-egui/src/state.rs','    pub leo_lock_cmyk_display: bool,','    pub leo_lock_cmyk_display: bool,\n    #[serde(default)]\n    pub leo_path_tools_seen: bool,')
edit('crates/ui-egui/src/state.rs','            leo_lock_cmyk_display: false,','            leo_lock_cmyk_display: false,\n            leo_path_tools_seen: false,')
edit('crates/ui-egui/src/state.rs','    ("leoImageExport", "Leo Design Image Pro Export", "image"),','    ("leoImageExport", "Leo Design Image Pro Export", "image"),\n    ("leoPathTools", "Leo Design Path Tools", "spline-pointer"),')
edit('crates/ui-egui/src/state.rs','    &["variables"],','    &["variables"],\n    &["leoPathTools"],')
edit('crates/ui-egui/src/panels.rs','pub mod leo_quick_settings;','pub mod leo_quick_settings;\npub mod leo_path_tools;')
edit('crates/ui-egui/src/panels.rs','        "leoImageExport" => crate::dialogs::show_leo_image_export_sidebar(app, ui),','        "leoImageExport" => crate::dialogs::show_leo_image_export_sidebar(app, ui),\n        "leoPathTools" => leo_path_tools::show(app, ui),')
edit('crates/ui-egui/src/menus/tree.rs','                panel("Leo Design Image Pro Export", "leoImageExport"),','                panel("Leo Design Image Pro Export", "leoImageExport"),\n                panel("Leo Design Path Tools", "leoPathTools"),')
edit('crates/ui-egui/src/app.rs','        panels::library_panel::tidy(self);','        panels::library_panel::tidy(self);\n        panels::leo_path_tools::reveal_once(self);')
(root/'crates/ui-egui/src/panels/leo_path_tools.rs').write_text((Path(__file__).parent/'panel.rs').read_text(encoding='utf-8'),encoding='utf-8',newline='\n')

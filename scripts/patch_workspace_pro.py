#!/usr/bin/env python3
"""Leo Design Workspace Pro v0.5.0: wider, freely composable panel workspace.

Preserves upstream docking, floating panels, workspace serialization, and the
Image Pro Export sidebar. Adds explicit move-to-tab / split-to-side commands
from every panel menu, including the legacy dock.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python scripts/patch_workspace_pro.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def replace_once(path, before, after, name):
    source = path.read_text(encoding="utf-8")
    n = source.count(before)
    if n != 1:
        raise SystemExit(f"FAIL: {name}: expected one anchor, found {n}")
    path.write_text(source.replace(before, after, 1), encoding="utf-8", newline="\n")

panels = root / "crates/ui-egui/src/panels/mod.rs"
layout_menu = '''/// Leo Design Workspace Pro: arrange any open panel without finicky drag-and-drop.
///
/// Pick a real destination panel, then combine as tabs or split into a new left,
/// right, upper, or lower section. The existing docking engine persists this
/// layout in the user's named workspaces; no separate state or file is needed.
pub fn leo_workspace_layout_menu(app: &mut VectorcraftApp, ui: &mut Ui, panel: &str) {
    let mut targets: Vec<String> = app.ui.docking.as_ref()
        .map(|layout| layout.panels().into_iter().cloned().collect())
        .unwrap_or_else(|| vec!["properties".into(), "layers".into(), "libraries".into()]);
    // Native floating groups are kept by VectorCraft's own layout model. Include
    // only the targets present in the shared layout, avoiding invalid moves.
    targets.retain(|target| target != panel);
    targets.sort_unstable();
    targets.dedup();

    ui.menu_button("Leo Design · 自由组合", |ui| {
        ui.label(egui::RichText::new("选择目标板块及排列方式").small().weak());
        if targets.is_empty() {
            ui.label("请先打开其他面板");
            return;
        }
        for (title, zone) in [
            ("合并为标签页", "tab"),
            ("拆分到目标左侧", "left"),
            ("拆分到目标右侧", "right"),
            ("拆分到目标上方", "top"),
            ("拆分到目标下方", "bottom"),
        ] {
            ui.menu_button(title, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("leo-workspace-targets", panel, zone))
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for anchor in &targets {
                            let label = crate::state::all_panels()
                                .find(|(id, _)| *id == anchor.as_str())
                                .map(|(_, label)| label)
                                .unwrap_or(anchor.as_str());
                            if ui.button(label).clicked() {
                                if let Err(error) = app.run(
                                    "window.panel.move",
                                    json!({"panel": panel, "anchor": anchor, "zone": zone}),
                                ) {
                                    app.ui.status = error;
                                }
                                ui.close();
                            }
                        }
                    });
            });
        }
        ui.separator();
        if ui.button("本板块独立浮动").clicked() {
            if let Err(error) = app.run("window.panel.float", json!({"panel": panel})) {
                app.ui.status = error;
            }
            ui.close();
        }
        ui.label(egui::RichText::new("布局可通过「窗口 → 工作区 → 新建工作区」保存").small().weak());
    });
}

'''
replace_once(
    panels,
    "/// Items of a panel's (≡) menu. Unimplemented Illustrator items are listed disabled.",
    layout_menu + "/// Items of a panel's (≡) menu. Unimplemented Illustrator items are listed disabled.",
    "Insert Workspace Pro panel menu",
)
replace_once(
    panels,
    '''            // The panel floats out of the dock or goes back in, as dragging its tab does.''',
    '''            leo_workspace_layout_menu(app, ui, id);
            ui.separator();
            // The panel floats out of the dock or goes back in, as dragging its tab does.''',
    "Put Workspace Pro into the right panel hamburger menu",
)

docking = root / "crates/ui-egui/src/panel_docking.rs"
replace_once(
    docking,
    '''    fn panel_menu(&mut self, ui: &mut egui::Ui, panel: &String) {
        crate::panels::panel_menu_items(self.app, ui, panel);
    }''',
    '''    fn panel_menu(&mut self, ui: &mut egui::Ui, panel: &String) {
        if crate::panels::panel_menu_items(self.app, ui, panel) {
            ui.separator();
        }
        crate::panels::leo_workspace_layout_menu(self.app, ui, panel);
    }''',
    "Add Workspace Pro options to shared-dock tab context menu",
)
replace_once(
    docking,
    '''    response.context_menu(|ui| {
        if ui.button(tl!("Float Panel")).clicked() {''',
    '''    response.context_menu(|ui| {
        crate::panels::leo_workspace_layout_menu(app, ui, panel);
        ui.separator();
        if ui.button(tl!("Float Panel")).clicked() {''',
    "Add Workspace Pro options to legacy tab context menu",
)
replace_once(
    docking,
    "PanelLimits { min: egui::vec2(220.0, if short.contains(id) { 0.0 } else { 80.0 }),",
    "PanelLimits { min: egui::vec2(190.0, if short.contains(id) { 0.0 } else { 80.0 }),",
    "Allow narrower multi-column panels",
)
replace_once(
    docking,
    ".size_range(220.0..=720.0)",
    ".size_range(200.0..=1200.0)",
    "Widen the resizable multi-column dock",
)

dock = root / "crates/ui-egui/src/dock.rs"
replace_once(dock, "const SHARED_DOCK_MIN: f32 = 220.0;", "const SHARED_DOCK_MIN: f32 = 200.0;", "Shared dock min width")
replace_once(dock, "const SHARED_DOCK_MAX: f32 = 720.0;", "const SHARED_DOCK_MAX: f32 = 1200.0;", "Shared dock maximum width")

print("SUCCESS: Workspace Pro: tab, horizontal and vertical split choices and 1200pt dock.")

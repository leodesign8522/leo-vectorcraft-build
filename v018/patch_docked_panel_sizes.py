#!/usr/bin/env python3
"""Persist direct resized iconic flyouts in VectorCraft v0.18 Workspace settings.

Craft-ui (pinned git dependency) draws the resize grips; this host bridge
persists the resulting per-panel width/height into UiState JSON. This keeps
the flyout anchored at the right dock and does not require Float Panel.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2: raise SystemExit("Usage: patch_docked_panel_sizes.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])
def patch(rel,a,b,label):
 p=root/rel;s=p.read_text(encoding="utf-8")
 if s.count(a)!=1:raise SystemExit(f"FAIL {label}: anchor count={s.count(a)}")
 p.write_text(s.replace(a,b,1),encoding="utf-8",newline="\n")
 print("OK",label)
patch("crates/ui-egui/src/state.rs",
 '''    pub open_panel: Option<String>,
    pub control_bar: bool,''',
 '''    pub open_panel: Option<String>,
    /// Leo Design: the draggable sizes of individual right-dock icon flyouts.
    /// These are view settings, saved with the workspace/preferences, not art.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub leo_icon_flyout_sizes: std::collections::BTreeMap<String, [f32; 2]>,
    pub control_bar: bool,''',
 "add persisted per-panel icon flyout size preferences")
patch("crates/ui-egui/src/state.rs",
 '''            open_panel: None,
            control_bar: false,''',
 '''            open_panel: None,
            leo_icon_flyout_sizes: Default::default(),
            control_bar: false,''',
 "provide default sizes compatible with existing ui.json")
patch("crates/ui-egui/src/panel_docking.rs",
 '''    let mut output = dock.show_content(ui, &layout, &style, &mut PanelContent { app: &mut *app });
    app.dock_rt.space = craft_ui::docking::float_space(&ctx, area);''',
 '''    // Leo Design: synchronize each icon's last user-dragged size. A cached
    // size is only reinserted if the flyout is open. The actual drag happens
    // within craft-ui's resize handles, not via Float Panel.
    let before = app.ui.open_panel.clone();
    if let Some(id) = before.as_ref() {
        if let Some([w,h]) = app.ui.leo_icon_flyout_sizes.get(id).copied() {
            ctx.data_mut(|d| d.insert_temp(area.with(("leo-icon-size", id.clone())), egui::vec2(w,h)));
        }
    }
    let mut output = dock.show_content(ui, &layout, &style, &mut PanelContent { app: &mut *app });
    if let Some(id) = before {
        if let Some(size) = ctx.data(|d| d.get_temp::<egui::Vec2>(area.with(("leo-icon-size", id.clone())))) {
            let saved = [size.x.clamp(220.0, 900.0), size.y.clamp(160.0, 1200.0)];
            if saved.iter().all(|n| n.is_finite())
                && app.ui.leo_icon_flyout_sizes.get(&id).copied() != Some(saved) {
                app.ui.leo_icon_flyout_sizes.insert(id, saved);
                app.prefs_changed = true;
            }
        }
    }
    app.dock_rt.space = craft_ui::docking::float_space(&ctx, area);''',
 "sync resized iconic flyout dimensions with persisted UI state")
print("SUCCESS Leo Design docked icon flyout resizing host bridge")

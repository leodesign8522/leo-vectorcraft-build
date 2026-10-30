#!/usr/bin/env python3
"""Include icon flyout widths/heights when saving, switching or exporting workspaces.

UiState stores live flyout sizes; Workspace owns the named snapshots. This
ensures a saved 'leo' layout has its own panel dimensions rather than sharing
them with all workspaces. Missing keys deserialize to normal auto-fit defaults.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2:raise SystemExit("Usage: patch_workspace_flyout_sizes.py VECTORCRAFT_ROOT")
p=Path(sys.argv[1])/"crates/ui-egui/src/workspaces.rs"
s=p.read_text(encoding="utf-8")
def edit(a,b,label):
 global s
 n=s.count(a)
 if n!=1:raise SystemExit(f"FAIL {label}: anchor count={n}")
 s=s.replace(a,b,1)
 print("OK",label)
edit(
 '''    pub open_panel: Option<String>,
    pub status_bar: bool,''',
 '''    pub open_panel: Option<String>,
    /// Leo Design: each workspace remembers its iconic flyout sizes.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub leo_icon_flyout_sizes: std::collections::BTreeMap<String, [f32; 2]>,
    pub status_bar: bool,''',
 "workspace schema stores per-panel flyout sizes")
edit(
 '''        open_panel: ui.open_panel.clone(),
        status_bar: ui.status_bar,''',
 '''        open_panel: ui.open_panel.clone(),
        leo_icon_flyout_sizes: ui.leo_icon_flyout_sizes.clone(),
        status_bar: ui.status_bar,''',
 "save current flyout sizes in named workspace")
edit(
 '''    ui.open_panel = w.open_panel.clone();
    ui.status_bar = w.status_bar;''',
 '''    ui.open_panel = w.open_panel.clone();
    ui.leo_icon_flyout_sizes = w.leo_icon_flyout_sizes.clone();
    ui.status_bar = w.status_bar;''',
 "restore flyout sizes when switching workspaces")
p.write_text(s,encoding="utf-8",newline="\n")
# On workspace switch, craft-ui must not reuse another workspace's
# temporary egui sizes. Reload the new workspace's saved dimensions instead.
d = Path(sys.argv[1])/"crates/ui-egui/src/panel_docking.rs"
dsrc=d.read_text(encoding="utf-8")
anchor='''    if changed_workspace {
        craft_ui::docking::cancel_drag::<String>(&ctx, area);
    }'''
replacement='''    if changed_workspace {
        craft_ui::docking::cancel_drag::<String>(&ctx, area);
        // Leo Design: each named workspace has its own per-panel sizes.
        ctx.data_mut(|data| {
            for (id, _) in crate::state::all_panels() {
                data.remove::<egui::Vec2>(area.with(("leo-icon-size", id.to_owned())));
            }
        });
    }'''
if dsrc.count(anchor)!=1:raise SystemExit(f"FAIL workspace switch hook: {dsrc.count(anchor)}")
d.write_text(dsrc.replace(anchor,replacement,1),encoding="utf-8",newline="\n")
print("SUCCESS named workspaces include Leo Design docked icon flyout geometry")

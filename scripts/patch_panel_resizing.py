#!/usr/bin/env python3
"""Leo Design Workspace Pro v0.5.6: remembered custom panel dimensions.

Icon flyouts resize from their bottom-left corner (right edge is attached to
the toolbar); floating legacy panels resize from bottom-right. A docked panel
still fills its allotted dock region: float it to customize independent height.
Sizes live in UI preferences, never in the artwork document.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_panel_resizing.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def rep(relative, old, new, what):
    f = root / relative
    s = f.read_text(encoding="utf-8")
    if s.count(old) != 1:
        raise SystemExit(f"FAIL {what}: anchor count {s.count(old)} in {relative}")
    f.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("OK", what)

state = "crates/ui-egui/src/state.rs"
rep(state,
    '    pub open_panel: Option<String>,',
    '''    pub open_panel: Option<String>,
    /// Leo Design: independent icon-flyout/floating panel [width, height] (screen points).
    /// Height 0 means natural/content-fit height until resized.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub leo_panel_sizes: std::collections::BTreeMap<String, [f32; 2]>,''',
    "remember independent panel sizes")
rep(state,
    '            open_panel: None,',
    '''            open_panel: None,
            leo_panel_sizes: Default::default(),''',
    "default panel sizes")

panels = "crates/ui-egui/src/panels/mod.rs"
rep(panels,
    '    ui.menu_button("Leo Design · 自由组合", |ui| {',
    '''    ui.menu_button("Leo Design · 面板宽高", |ui| {
        ui.set_min_width(270.0);
        let current = app.ui.leo_panel_sizes.get(panel).copied()
            .unwrap_or([crate::dock::panel_width(panel), 0.0]);
        let mut width = current[0].clamp(220.0, 900.0);
        let mut height = if current[1] > 0.0 { current[1].clamp(160.0, 950.0) } else { 430.0 };
        let mut change = false;
        ui.horizontal(|ui| {
            ui.label("宽度");
            change |= ui.add(egui::DragValue::new(&mut width).speed(1.0)
                .range(220.0..=900.0).suffix(" pt")).changed();
            ui.label("高度");
            change |= ui.add(egui::DragValue::new(&mut height).speed(1.0)
                .range(160.0..=950.0).suffix(" pt")).changed();
        });
        if change {
            app.ui.leo_panel_sizes.insert(panel.to_string(), [width, height]);
            if let Some(layout) = app.ui.docking.as_mut() {
                if let Some(group) = layout.floating.iter_mut().find(|g| g.panels.iter().any(|id| id == panel)) {
                    group.rect[2] = width;
                    group.rect[3] = height;
                }
            }
            app.prefs_changed = true;
        }
        ui.horizontal(|ui| {
            for (label, w, h) in [
                ("紧凑", 300.0, 340.0),
                ("标准", 390.0, 550.0),
                ("宽大", 560.0, 750.0),
            ] {
                if ui.button(label).clicked() {
                    app.ui.leo_panel_sizes.insert(panel.to_string(), [w, h]);
                    if let Some(layout) = app.ui.docking.as_mut() {
                        if let Some(group) = layout.floating.iter_mut().find(|g| g.panels.iter().any(|id| id == panel)) {
                            group.rect[2] = w;
                            group.rect[3] = h;
                        }
                    }
                    app.prefs_changed = true;
                }
            }
        });
        if ui.button("恢复自动内容高度").clicked() {
            app.ui.leo_panel_sizes.remove(panel);
            app.prefs_changed = true;
        }
        ui.separator();
        let popped = app.ui.open_panel.as_deref() == Some(panel);
        let legacy_float = crate::floating::group_of(&app.ui, panel).is_some();
        if popped {
            ui.label("图标弹出面板：拖动左下角调整宽高");
        } else if legacy_float {
            ui.label("独立浮动面板：拖动右下角调整宽高");
        } else {
            ui.label("停靠面板由布局填满高度；拖动分隔线可改变宽度。");
            if ui.button("独立浮动后调整宽高").clicked() {
                if let Err(e) = app.run("window.panel.float", json!({"panel": panel})) {
                    app.ui.status = e;
                }
                ui.close();
            }
        }
        ui.label(egui::RichText::new("尺寸保存在首选项，不影响原始设计文件。").small().weak());
    });
    ui.menu_button("Leo Design · 自由组合", |ui| {''',
    "add per-panel size controls to all panel menus")

dock = "crates/ui-egui/src/dock.rs"
rep(dock,
    '''    let width = panel_width(id);
    let tall = (screen.height() - FLYOUT_TOP - 26.0 - 20.0 - 40.0).clamp(160.0, 560.0);''',
    '''    // Leo Design: every icon remembers its own width and optional fixed height.
    let saved_size = app.ui.leo_panel_sizes.get(id).copied();
    let width = saved_size.map_or(panel_width(id), |size| size[0]).clamp(220.0, 900.0)
        .min((screen.width() - ICON_COL - 16.0).max(220.0));
    let tall = (screen.height() - FLYOUT_TOP - 26.0 - 20.0 - 40.0).clamp(160.0, 560.0);''',
    "use each icon panel's remembered width")

rep(dock,
    '''            panel_body(app, ui, id, width, tall);
        });
    });
    // User Interface › Auto-Collapse Iconic Panels''',
    '''            if let Some([_, height]) = saved_size.filter(|size| size[1] > 0.0) {
                let body_height = (height - 46.0).clamp(120.0, screen.height().max(160.0));
                widgets::panel_scroll(
                    ui,
                    egui::ScrollArea::vertical().id_salt(("leo-panel-size", id))
                        .max_height(body_height).min_scrolled_height(body_height)
                        .auto_shrink([false, false]),
                    |ui| panel_body(app, ui, id, width, body_height),
                );
            } else {
                panel_body(app, ui, id, width, tall);
            }
            // The flyout is anchored by its right edge: bottom-LEFT is the
            // natural grip, so dragging left expands it towards the artwork.
            let bounds = ui.min_rect();
            let corner = Rect::from_min_size(bounds.left_bottom() - vec2(0.0, 18.0), vec2(18.0, 18.0));
            let grip = ui.interact(corner, ui.id().with(("leo-resize-flyout", id)), Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeNeSw)
                .on_hover_text("Leo Design：拖动调整面板宽高");
            ui.painter().line_segment(
                [corner.left_bottom() + vec2(3.0, -4.0), corner.left_bottom() + vec2(14.0, -15.0)],
                Stroke::new(1.5, t.text_dim),
            );
            if grip.dragged() {
                let delta = ui.input(|i| i.pointer.delta());
                let before = app.ui.leo_panel_sizes.get(id).copied()
                    .unwrap_or([width, bounds.height()]);
                let updated = [
                    (before[0] - delta.x).clamp(220.0, 900.0),
                    (before[1] + delta.y).clamp(160.0, 950.0),
                ];
                if updated != before {
                    app.ui.leo_panel_sizes.insert(id.to_string(), updated);
                    app.prefs_changed = true;
                }
            }
        });
    });
    // User Interface › Auto-Collapse Iconic Panels''',
    "add resizable corner and body-height scrolling to popout panels")

floating = "crates/ui-egui/src/floating.rs"
rep(floating,
    '''    let given = set.iter().filter_map(|&i| app.ui.floating_panels.get(i)?.width).reduce(f32::max).map(|w| w.min(screen.width() - 16.0));
    let area = area_id(first);''',
    '''    let given = set.iter().filter_map(|&i| app.ui.floating_panels.get(i)?.width).reduce(f32::max).map(|w| w.min(screen.width() - 16.0));
    let custom_size = app.ui.leo_panel_sizes.get(lead.active).copied();
    let area = area_id(first);''',
    "remember floating panel custom dimensions")

rep(floating,
    '''                .fold(given.unwrap_or(0.0), f32::max);
            ui.set_width(width);''',
    '''                .fold(given.unwrap_or(0.0), f32::max)
                .max(custom_size.map_or(0.0, |size| size[0].clamp(220.0, 900.0)));
            ui.set_width(width);''',
    "set floating panel width")

rep(floating,
    '''            let alone = (groups.len() == 1).then_some(pos);
            draw_groups(app, ui, &groups, width, tall, bar.top(), alone, action);
        });
    });
}

/// Draw `groups` top to bottom''',
    '''            let alone = (groups.len() == 1).then_some(pos);
            if let Some([_, height]) = custom_size.filter(|size| size[1] > 0.0) {
                let available = (height - TITLE - 24.0).max(120.0);
                widgets::panel_scroll(ui,
                    egui::ScrollArea::vertical().id_salt(("leo-floating-size", first))
                        .max_height(available).min_scrolled_height(available)
                        .auto_shrink([false, false]),
                    |ui| draw_groups(app, ui, &groups, width, tall, bar.top(), alone, action));
            } else {
                draw_groups(app, ui, &groups, width, tall, bar.top(), alone, action);
            }
            let bounds = ui.min_rect();
            let corner = Rect::from_min_size(bounds.right_bottom() - vec2(18.0, 18.0), vec2(18.0, 18.0));
            let grip = ui.interact(corner, ui.id().with(("leo-resize-float", first)), Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe)
                .on_hover_text("Leo Design：拖动调整面板宽高");
            ui.painter().line_segment(
                [corner.right_bottom() - vec2(14.0, 3.0), corner.right_bottom() - vec2(3.0, 14.0)],
                Stroke::new(1.5, t.text_dim),
            );
            if grip.dragged() {
                let delta = ui.input(|i| i.pointer.delta());
                let before = app.ui.leo_panel_sizes.get(lead_active).copied()
                    .unwrap_or([width, bounds.height()]);
                let updated = [
                    (before[0] + delta.x).clamp(220.0, 900.0),
                    (before[1] + delta.y).clamp(160.0, 950.0),
                ];
                if updated != before {
                    app.ui.leo_panel_sizes.insert(lead_active.to_string(), updated);
                    app.prefs_changed = true;
                }
            }
        });
    });
}

/// Draw `groups` top to bottom''',
    "allow drag resizing legacy floating groups")

print("SUCCESS: Leo Design Workspace Pro v0.5.6 resizable icon and floating panel sizes")

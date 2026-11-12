#!/usr/bin/env python3
"""Leo Design panel size presets and export sidebar for v0.24.

- Image Pro Export: avoid nested scroll areas, expose a one-click large floating panel.
- All icon-panel hamburger menus: small/medium/large/custom floating sizes.
- Use shared craft-ui Floating layout, so the OS window can be resized with its border
  and the new rectangle persists as part of the Workspace JSON.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_panel_usability.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])

def replace(rel,original,new,label):
    path=root/rel
    s=path.read_text(encoding="utf-8")
    c=s.count(original)
    if c != 1:
        raise SystemExit(f"FAIL {label}: {rel} expected one anchor, got {c}")
    path.write_text(s.replace(original,new,1),encoding="utf-8",newline="\n")
    print(f"OK {label}")

replace(
    "crates/ui-egui/src/panel_docking.rs",
    '    match panel {\n        "properties" | "layers" | "libraries" => egui::vec2(300.0, 420.0),',
    '''    match panel {
        // Leo Design: the Image Pro Export workflow needs the full settings column.
        "leoImageExport" => egui::vec2(430.0, 690.0),
        "properties" | "layers" | "libraries" => egui::vec2(300.0, 420.0),''',
    "larger default floating size for Image Pro Export"
)

insert= '''
/// Leo Design: set a specific panel's size without changing the canvas size.
/// Use the native docking layout for both icon flyouts and floating windows.
pub(crate) fn leo_size_panel(app: &mut VectorcraftApp, raw: &str, width: f32, height: f32) -> Result<(), String> {
    let panel = normalize(raw).ok_or("unknown panel")?.to_owned();
    if !width.is_finite() || !height.is_finite() {
        return Err("panel width and height must be finite".to_owned());
    }
    let width = width.clamp(240.0, 900.0);
    let height = height.clamp(240.0, 1200.0);
    let space = float_space(app);
    let mut txn = Transaction::begin(app)?;
    if in_iconic_column(&txn.workspace.layout, &panel) {
        txn.apply(Action::ResizeFlyoutWidth { panel: panel.clone(), width: Some(width) })?;
        txn.apply(Action::ResizeGroup { panel: panel.clone(), height: Some(height) })?;
        txn.commit(app, Vec::new(), Some(panel));
        return Ok(());
    }
    if !txn.workspace.layout.contains(&panel) {
        txn.float_new(&panel, &space)?;
    }
    let floating = floating_window(&txn.workspace.layout, &panel);
    let own_window = floating.is_some_and(|w| w.panels().count() == 1);
    let old_rect = floating.map(|w| w.rect);
    let at = old_rect.map_or(
        [space.area.min.x + 32.0, space.area.min.y + 32.0],
        |rect| [rect[0], rect[1]],
    );
    let rect = [at[0], at[1], width, height];
    let action = if own_window {
        Action::MoveFloating { panel: panel.clone(), rect }
    } else {
        Action::Float { panel: panel.clone(), rect }
    };
    txn.apply(action)?;
    txn.commit(app, Vec::new(), Some(panel.clone()));
    app.ui.open_panel = None;
    Ok(())
}

'''
replace(
    "crates/ui-egui/src/panel_docking.rs",
    '/// A change to the layout in use, committed only when every step succeeded.\nstruct Transaction {',
    insert + '/// A change to the layout in use, committed only when every step succeeded.\nstruct Transaction {',
    "native resizable floating panel transaction"
)
replace(
    "crates/ui-egui/src/panels.rs",
    '''            // The panel floats out of the dock or goes back in, as dragging its tab does.
            let (label, cmd) = if crate::panel_docking::floats(&app.ui, id) {''',
    '''            // Leo Design: every panel can become a user-resizable native
            // floating window. Shared docking stores its width and height in
            // the workspace, not in the document.
            ui.menu_button("Leo Design · 面板尺寸", |ui| {
                for (name, w, h) in [
                    ("紧凑 320 × 420", 320.0, 420.0),
                    ("标准 430 × 690", 430.0, 690.0),
                    ("宽大 600 × 850", 600.0, 850.0),
                ] {
                    if ui.button(name).clicked() {
                        if let Err(e) = crate::panel_docking::leo_size_panel(app, id, w, h) {
                            app.ui.status = e;
                        }
                        ui.close();
                    }
                }
            });
            ui.separator();
            // The panel floats out of the dock or goes back in, as dragging its tab does.
            let (label, cmd) = if crate::panel_docking::floats(&app.ui, id) {''',
    "add resizable size presets to every panel menu"
)

# The outer shared docking shell already owns one scrolling viewport.
# Nested independent ScrollAreas were using the same wheel/height budget, so
# the image export form visually stopped after "JPEG quality" in a short flyout.
replace(
    "crates/ui-egui/src/dialogs/leo_image_export.rs",
    '''    egui::ScrollArea::vertical().id_salt("leo-image-export-scroll").auto_shrink([false, true]).show(ui, |ui| {
        ui.set_min_width(190.0);''',
    '''    // The outer dock already scrolls: do not create a second scroll area
    // that traps wheel input or lets egui shrink the flyout to only 360 pt.
    ui.vertical(|ui| {
        ui.set_min_width(190.0);
        ui.horizontal(|ui| {
            if ui.small_button("⛶ 展开面板").clicked() {
                if let Err(e) = crate::panel_docking::leo_size_panel(app, "leoImageExport", 430.0, 690.0) {
                    app.ui.status = e;
                }
            }
        });
        ui.add_space(4.0);''',
    "remove double scrolling and provide one-click large export panel"
)
print("SUCCESS Leo Design panel presets use native v0.24 docking")


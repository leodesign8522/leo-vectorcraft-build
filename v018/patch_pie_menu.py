#!/usr/bin/env python3
"""Integrate Leo Design Pie Menu v0.1.0 in official VectorCraft v0.18.0.

Patch applied after modular Core Bridge feature migrations. Fail closed if
upstream changes any insertion point. Native canvas/keyboard input; configs
in existing UI prefs, no global Windows hooks.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2:raise SystemExit("Usage: patch_pie_menu.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1]); here=Path(__file__).resolve().parent
def change(rel,old,new,label):
 p=root/rel;s=p.read_text(encoding="utf-8")
 count=s.count(old)
 if count!=1: raise SystemExit(f"FAIL {label}: {rel}, anchor count={count}")
 p.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
 print("OK",label)

dest=root/"crates/ui-egui/src/leo_pie_menu.rs"
if dest.exists():raise SystemExit("FAIL Pie Menu module already exists")
dest.write_text((here/"pie-menu/leo_pie_menu.rs").read_text(encoding="utf-8"),encoding="utf-8",newline="\n")
print("OK native Pie Menu module")

change("crates/ui-egui/src/lib.rs",
 '''pub mod layout_migration;
pub mod menus;''',
 '''pub mod layout_migration;
pub mod leo_pie_menu;
pub mod menus;''',
 "register radial menu module")
change("crates/ui-egui/src/state.rs",
 '''    pub palette_open: bool,
    pub palette_query: String,
    /// Screen mode:''',
 '''    pub palette_open: bool,
    pub palette_query: String,
    /// Leo Design: native editable radial menu bindings/slots.
    #[serde(default = "crate::leo_pie_menu::defaults")]
    pub leo_pies: Vec<crate::leo_pie_menu::PieConfig>,
    /// Runtime state is not saved to any artwork or UI preferences file.
    #[serde(skip)]
    pub leo_pie_open: Option<crate::leo_pie_menu::OpenPie>,
    #[serde(skip)]
    pub leo_pie_editor: bool,
    #[serde(skip)]
    pub leo_pie_selected: usize,
    #[serde(skip)]
    pub leo_pie_slot: usize,
    #[serde(skip)]
    pub leo_pie_query: String,
    #[serde(skip)]
    pub leo_pie_recording: String,
    #[serde(skip)]
    pub leo_pie_import: String,
    #[serde(skip)]
    pub leo_pie_message: String,
    /// Screen mode:''',
 "save user-defined menu configuration, keep open state transient")
change("crates/ui-egui/src/state.rs",
 '''            palette_open: false,
            palette_query: String::new(),
            screen_mode: 0,''',
 '''            palette_open: false,
            palette_query: String::new(),
            leo_pies: crate::leo_pie_menu::defaults(),
            leo_pie_open: None,
            leo_pie_editor: false,
            leo_pie_selected: 0,
            leo_pie_slot: 0,
            leo_pie_query: String::new(),
            leo_pie_recording: String::new(),
            leo_pie_import: String::new(),
            leo_pie_message: String::new(),
            screen_mode: 0,''',
 "default config on fresh installations and migration")
change("crates/ui-egui/src/shortcuts.rs",
 '''    ctx.input_mut(unclip_chords);
    // Tab is ours when no field has the keyboard''',
 '''    // Leo Design: user-chosen Pie Menu keyboard shortcut takes precedence
    // when no editor, IME or dialog owns the keyboard.
    if crate::leo_pie_menu::keyboard_frame(app, ctx) {
        return;
    }
    ctx.input_mut(unclip_chords);
    // Tab is ours when no field has the keyboard''',
 "intercept user-configured Shift+Space and other keyboard chords")
change("crates/ui-egui/src/canvas.rs",
 '''    let resp = ui.interact(rect, egui::Id::unique("canvas"), Sense::click_and_drag());
    // A press on the canvas''',
 '''    let resp = ui.interact(rect, egui::Id::unique("canvas"), Sense::click_and_drag());
    // Leo Design: modifier+right-click belongs to the pie before the normal
    // context menu sees the event; ordinary right click is unchanged.
    let leo_pie_owns_canvas = crate::leo_pie_menu::mouse_frame(app, ui, &resp);
    // A press on the canvas''',
 "detect a radial menu mouse gesture from canvas pointer")
change("crates/ui-egui/src/canvas.rs",
 '''    } else {
        handle_input(app, ui, &resp, rect);
        // `hover_doc`''',
 '''    } else if leo_pie_owns_canvas {
        // Suppress normal tool pan/selection while the radial menu owns input.
    } else {
        handle_input(app, ui, &resp, rect);
        // `hover_doc`''',
 "avoid changing artwork while selecting a pie menu direction")
change("crates/ui-egui/src/canvas.rs",
 '''    if app.ui.screen_mode < 3 {
        context_menu(app, &resp, &xf);
    }''',
 '''    if app.ui.screen_mode < 3 && !leo_pie_owns_canvas {
        context_menu(app, &resp, &xf);
    }''',
 "keep normal RMB while avoiding duplicate menus for custom RMB chords")
change("crates/ui-egui/src/canvas.rs",
 '''        ui.ctx().set_cursor_icon(cur);
    }
}

#[cfg(test)]
mod tests;''',
 '''        ui.ctx().set_cursor_icon(cur);
    }
    // Draw the real radial command menu above artwork and tool overlays.
    crate::leo_pie_menu::paint(ui, app);
    let ctx = ui.ctx().clone();
    crate::leo_pie_menu::editor(app, &ctx);
}

#[cfg(test)]
mod tests;''',
 "render radial overlay and native searchable menu configuration window")
change("crates/ui-egui/src/menus/tree.rs",
 '''                c("Control", "window.control"),
                c("Contextual Task Bar", "window.taskBar"),''',
 '''                c("Leo Design 饼菜单设置…", "ui.leoPieSettings"),
                Sep,
                c("Control", "window.control"),
                c("Contextual Task Bar", "window.taskBar"),''',
 "Window menu opens Leo Design Pie Menu editor")
change("crates/ui-egui/src/menus/ui_commands.rs",
 '''pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[
''',
 '''pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[
    ("ui.leoPieSettings", "Leo Design Pie Menu Settings", "", "{} open native radial command menu editor"),
''',
 "register UI command for native menu editor")
change("crates/ui-egui/src/menus/run.rs",
 '''    // The Window menu's panel items toggle like its other checked items''',
 '''    if id == "ui.leoPieSettings" {
        app.ui.leo_pie_editor = true;
        return Some(Ok(json!({"open": "leoPieEditor"})));
    }
    // The Window menu's panel items toggle like its other checked items''',
 "route the UI command directly to the editor")

print("SUCCESS Leo Design Pie Menu v0.1.0 integrated, including self-configurable shortcuts and searchable command binding")

#!/usr/bin/env python3
"""Leo Design v0.5.4: persistent CMYK-only DISPLAY for Color panel.

The lock changes slider display only. Selecting RGB art must not change
stored color channels or the document's color space.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2: raise SystemExit("Usage: patch_color_panel_lock.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])
def replace(file,old,new):
    p=root/file;s=p.read_text(encoding="utf-8")
    if s.count(old)!=1: raise SystemExit(f"FAIL {file}: anchor count {s.count(old)}")
    p.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
replace("crates/ui-egui/src/state.rs",
"""    pub color_mode: Option<crate::panels::color::Mode>,
    /// Color panel: Hide Options""",
"""    pub color_mode: Option<crate::panels::color::Mode>,
    /// Leo Design: show CMYK values consistently without converting selected art.
    #[serde(default)]
    pub leo_lock_cmyk_display: bool,
    /// Color panel: Hide Options""")
replace("crates/ui-egui/src/state.rs",
"""            color_mode: None,
            color_hide_options: false,""",
"""            color_mode: None,
            leo_lock_cmyk_display: false,
            color_hide_options: false,""")
replace("crates/ui-egui/src/panels/color.rs",
"""fn shown_mode(app: &VectorcraftApp, color: Option<&Color>) -> Mode {
    let defaults = app.session.active().is_none_or(|st| st.selection.subjects().is_empty());""",
"""fn shown_mode(app: &VectorcraftApp, color: Option<&Color>) -> Mode {
    // Leo Design: presentation-only lock. RGB/CMYK values stored on objects
    // remain unchanged. Tint swatches still use their dedicated tint row.
    if app.ui.leo_lock_cmyk_display {
        return Mode::Cmyk;
    }
    let defaults = app.session.active().is_none_or(|st| st.selection.subjects().is_empty());""")
replace("crates/ui-egui/src/panels/color.rs",
"""fn set_mode(app: &mut VectorcraftApp, tgt: &Target, mode: Mode) {
    app.ui.color_mode = Some(mode);""",
"""fn set_mode(app: &mut VectorcraftApp, tgt: &Target, mode: Mode) {
    // Explicitly choosing a native mode exits the display-only CMYK lock.
    app.ui.leo_lock_cmyk_display = false;
    app.ui.color_mode = Some(mode);""")
replace("crates/ui-egui/src/panels/color.rs",
"""    ui.separator();
    let tgt = target(app, ui);
    let color = tgt.color();
    // Tint mode checks no colour mode""",
"""    ui.separator();
    let locked = app.ui.leo_lock_cmyk_display;
    if menu_item(ui, "Leo Design · 固定显示 CMYK（不转换原色）", true, locked) {
        app.ui.leo_lock_cmyk_display = !locked;
        app.prefs_changed = true;
    }
    ui.separator();
    let tgt = target(app, ui);
    let color = tgt.color();
    // Tint mode checks no colour mode""")
print("SUCCESS: Color panel CMYK display lock persists across restarts")

//! Leo Design Pie Menu v0.5.1 — native VectorCraft canvas radial commands.
//! Multiple configurable menus, keyboard and modifier+RMB bindings, a searchable
//! native command picker, and JSON import/export. No OS-global mouse hook.
use crate::VectorcraftApp;
use egui::{Color32, Key, Modifiers, PointerButton, Pos2, Rect, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PieSlot {
    pub name: String,
    pub command: String,
    /// Parameters captured from a native menu item, defaulting for old configs.
    pub params: serde_json::Value,
    /// Index in the exported menu collection; renames do not break links.
    pub submenu: Option<usize>,
}
impl Default for PieSlot {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            params: serde_json::Value::Null,
            submenu: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PieConfig {
    pub name: String,
    pub keyboard: String,
    pub mouse: String,
    /// Explicit consent before overriding a built-in keyboard/nav shortcut.
    pub allow_keyboard_override: bool,
    /// Mouse travel (in screen points) required before a gesture is valid.
    pub flick_threshold: f32,
    /// Radial label layout size, independent of gesture distance.
    pub menu_radius: f32,
    /// Blender-style hold/flick/release mode for keyboard shortcuts.
    pub keyboard_release_select: bool,
    pub slots: Vec<PieSlot>,
}
impl Default for PieConfig {
    fn default() -> Self {
        Self {
            name: "我的常用设计".into(),
            keyboard: String::new(),
            mouse: "Shift+Alt+RMB".into(),
            allow_keyboard_override: false,
            flick_threshold: 24.0,
            menu_radius: 106.0,
            keyboard_release_select: false,
            slots: vec![PieSlot::default(); 8],
        }
    }
}
pub fn defaults() -> Vec<PieConfig> {
    let mut a = PieConfig::default();
    for (i, (name, cmd)) in [
        ("编组", "object.group"),
        ("拆组", "object.ungroup"),
        ("镜像", "object.reflect"),
        ("连接路径", "path.join"),
        ("取消选择", "select.none"),
        ("对齐", "object.align"),
        ("直接选择", "tool:directSelection"),
        ("选择", "tool:selection"),
    ]
    .iter()
    .enumerate()
    {
        a.slots[i] = PieSlot {
            name: (*name).into(),
            command: (*cmd).into(),
            ..PieSlot::default()
        };
    }
    let b = PieConfig {
        name: "快捷路径工具".into(),
        mouse: "Shift+Ctrl+RMB".into(),
        ..PieConfig::default()
    };
    vec![a, b]
}
#[derive(Clone, Debug)]
pub struct OpenPie {
    pub menu: usize,
    pub origin: Pos2,
    pub mouse: bool,
    pub key: Option<Key>,
    pub last_sector: Option<usize>,
    pub history: Vec<(usize, Pos2)>,
}
impl OpenPie {
    pub fn sector(&self, at: Pos2, threshold: f32) -> Option<usize> {
        let d = at - self.origin;
        // Blender-like flick selection: only a small inner deadzone;
        // NEVER impose a maximum distance (fast gestures often overshoot).
        if d.length_sq() < threshold.clamp(8.0, 64.0).powi(2) {
            return None;
        }
        let theta = d.y.atan2(d.x);
        Some(
            ((theta + std::f32::consts::FRAC_PI_8).rem_euclid(std::f32::consts::TAU)
                / std::f32::consts::FRAC_PI_4)
                .floor() as usize
                % 8,
        )
    }
}
fn normalize(s: &str) -> String {
    s.replace(' ', "").to_ascii_uppercase()
}
fn mouse_binding(s: &str, m: Modifiers) -> bool {
    let s = normalize(s);
    if s.is_empty() {
        return false;
    }
    let alt = s.contains("ALT+");
    let shift = s.contains("SHIFT+");
    let ctrl = s.contains("CTRL+");
    s.ends_with("RMB")
        && (alt || shift || ctrl)
        && m.alt == alt
        && m.shift == shift
        && m.ctrl == ctrl
        && !m.mac_cmd
}
fn keyboard_binding(s: &str, key: Key, m: Modifiers) -> bool {
    let s = normalize(s);
    if s.is_empty() || s.contains("RMB") {
        return false;
    }
    let alt = s.contains("ALT+");
    let shift = s.contains("SHIFT+");
    let ctrl = s.contains("CTRL+");
    let name = s.rsplit('+').next().unwrap_or("");
    let current = format!("{key:?}").to_ascii_uppercase();
    (current == name || (name == "SPACE" && key == Key::Space))
        && m.alt == alt
        && m.shift == shift
        && m.ctrl == ctrl
        && !m.mac_cmd
}
fn keyboard_text(key: Key, m: Modifiers) -> String {
    let mut v = String::new();
    if m.ctrl {
        v.push_str("Ctrl+");
    }
    if m.alt {
        v.push_str("Alt+");
    }
    if m.shift {
        v.push_str("Shift+");
    }
    v.push_str(&format!("{key:?}"));
    v
}
fn mouse_text(m: Modifiers) -> String {
    let mut v = String::new();
    if m.ctrl {
        v.push_str("Ctrl+");
    }
    if m.alt {
        v.push_str("Alt+");
    }
    if m.shift {
        v.push_str("Shift+");
    }
    v.push_str("RMB");
    v
}
fn valid_binding(s: &str, mouse: bool) -> bool {
    if s.trim().is_empty() {
        return true;
    }
    let s = normalize(s);
    let parts: Vec<_> = s.split('+').collect();
    if parts.is_empty() {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    for part in &parts[..parts.len() - 1] {
        if !matches!(*part, "CTRL" | "ALT" | "SHIFT") || !seen.insert(*part) {
            return false;
        }
    }
    if mouse {
        parts.last() == Some(&"RMB") && parts.len() > 1
    } else {
        // There is no hardcoded list of allowed letters, Fn keys or special
        // keys. Accept any keyboard key exposed by this version of egui.
        // That includes Shift+D, Ctrl+Shift+D and combinations with arrows,
        // punctuation and function keys. Dedicated modifier keys alone are
        // still rejected.
        let name = *parts.last().unwrap_or(&"");
        !name.is_empty()
            && egui::Key::ALL.iter().any(|key| {
                // Modifier-key press/release events themselves cannot be the
                // terminal shortcut key; they made Ctrl+ControlLeft appear valid.
                let k = format!("{key:?}");
                !matches!(
                    *key,
                    Key::ShiftLeft
                        | Key::ShiftRight
                        | Key::ControlLeft
                        | Key::ControlRight
                        | Key::AltLeft
                        | Key::AltRight
                        | Key::SuperLeft
                        | Key::SuperRight
                ) && k.eq_ignore_ascii_case(name)
            })
    }
}
fn validate(configs: &[PieConfig]) -> Result<(), String> {
    if configs.is_empty() || configs.len() > 24 {
        return Err("饼菜单数量必须在 1–24 之间".into());
    }
    let mut bindings = std::collections::HashSet::new();
    for config in configs {
        if config.slots.len() != 8 {
            return Err(format!("「{}」必须恰好有 8 个方向", config.name));
        }
        if !config.flick_threshold.is_finite()
            || !(8.0..=64.0).contains(&config.flick_threshold)
            || !config.menu_radius.is_finite()
            || !(70.0..=180.0).contains(&config.menu_radius)
        {
            return Err(format!("「{}」的手势距离或菜单尺寸无效", config.name));
        }
        if !valid_binding(&config.keyboard, false) || !valid_binding(&config.mouse, true) {
            return Err(format!("「{}」的快捷键格式无效", config.name));
        }
        for binding in [&config.keyboard, &config.mouse] {
            if !binding.is_empty() && !bindings.insert(normalize(binding)) {
                return Err(format!("重复快捷键：{binding}"));
            }
        }
    }
    let mut depths = vec![None; configs.len()];
    for index in 0..configs.len() {
        check_submenus(configs, index, &mut Vec::new(), &mut depths)?;
    }
    Ok(())
}
fn check_submenus(
    configs: &[PieConfig],
    menu: usize,
    path: &mut Vec<usize>,
    depths: &mut [Option<usize>],
) -> Result<usize, String> {
    if path.contains(&menu) {
        return Err("下级菜单不能形成循环".into());
    }
    if let Some(depth) = depths.get(menu).copied().flatten() {
        return Ok(depth);
    }
    let config = configs.get(menu).ok_or("下级菜单目标不存在")?;
    path.push(menu);
    let mut depth = 1;
    for target in config.slots.iter().filter_map(|s| s.submenu) {
        depth = depth.max(1 + check_submenus(configs, target, path, depths)?);
        if depth > 8 {
            return Err("下级菜单最多 8 层".into());
        }
    }
    path.pop();
    if let Some(value) = depths.get_mut(menu) {
        *value = Some(depth);
    }
    Ok(depth)
}
fn remove_menu(menus: &mut Vec<PieConfig>, index: usize) {
    menus.remove(index);
    for menu in menus {
        for slot in &mut menu.slots {
            if let Some(target) = slot.submenu {
                if target == index {
                    *slot = PieSlot::default();
                } else if target > index {
                    slot.submenu = Some(target - 1);
                }
            }
        }
    }
}
fn select_slot(app: &mut VectorcraftApp, mut open: OpenPie, sector: usize, at: Pos2) {
    let target = app
        .ui
        .leo_pies
        .get(open.menu)
        .and_then(|m| m.slots.get(sector))
        .and_then(|s| s.submenu);
    if let Some(target) = target {
        if target >= app.ui.leo_pies.len()
            || target == open.menu
            || open.history.iter().any(|(m, _)| *m == target)
            || open.history.len() >= 7
        {
            app.ui.leo_pie_open = None;
            app.status("下级菜单目标无效、循环或超过 8 层，请检查配置。");
            return;
        }
        open.history.push((open.menu, open.origin));
        open.menu = target;
        open.origin = app
            .canvas_rect
            .map(|clip| visual_center(clip, at, app.ui.leo_pies[target].menu_radius))
            .unwrap_or(at);
        // Entered on release: child stays open for a deliberate click, with no extra shortcut.
        open.mouse = false;
        open.key = None;
        open.last_sector = None;
        app.ui.leo_pie_open = Some(open);
    } else {
        app.ui.leo_pie_open = None;
        run_slot(app, open.menu, sector);
    }
}
fn back(app: &mut VectorcraftApp, mut open: OpenPie) {
    if let Some((menu, origin)) = open.history.pop() {
        open.menu = menu;
        open.origin = app
            .canvas_rect
            .map(|clip| {
                visual_center(
                    clip,
                    origin,
                    app.ui.leo_pies.get(menu).map_or(106.0, |m| m.menu_radius),
                )
            })
            .unwrap_or(origin);
        open.mouse = false;
        open.key = None;
        open.last_sector = None;
        app.ui.leo_pie_open = Some(open);
    } else {
        app.ui.leo_pie_open = None;
    }
}
fn run_slot(app: &mut VectorcraftApp, selected: usize, sector: usize) {
    let Some(slot) = app
        .ui
        .leo_pies
        .get(selected)
        .and_then(|m| m.slots.get(sector))
        .cloned()
    else {
        return;
    };
    if slot.submenu.is_some() {
        let origin = app
            .canvas_rect
            .map(|r| r.center())
            .unwrap_or(Pos2::new(400.0, 300.0));
        select_slot(
            app,
            OpenPie {
                menu: selected,
                origin,
                mouse: false,
                key: None,
                last_sector: None,
                history: vec![],
            },
            sector,
            origin,
        );
        return;
    }
    if slot.command.is_empty() {
        return;
    }
    app.ui.status.clear();
    if slot.command == "object.align"
        && slot.params.get("horizontal").is_none()
        && slot.params.get("vertical").is_none()
    {
        crate::menus::invoke(app, "window.panel", json!({"panel":"align"}));
        app.ui.leo_pie_message = "已打开对齐面板；可搜索左对齐、右对齐等具体功能直接添加".into();
        return;
    }
    if !slot.command.starts_with("tool:")
        && !slot.command.starts_with("panel:")
        && !crate::menus::enabled(app, &slot.command)
    {
        app.ui.leo_pie_message = format!(
            "「{}」当前不可用，请检查是否打开文档、选中所需对象或处于正确工具状态",
            slot.name
        );
        app.status(app.ui.leo_pie_message.clone());
        return;
    }
    if let Some(tool) = slot.command.strip_prefix("tool:") {
        crate::menus::invoke(app, "tool.select", json!({"tool":tool}));
    } else if let Some(panel) = slot.command.strip_prefix("panel:") {
        crate::menus::invoke(app, "window.panel", json!({"panel":panel}));
    } else {
        crate::menus::invoke(
            app,
            &slot.command,
            if slot.params.is_null() {
                json!({})
            } else {
                slot.params.clone()
            },
        );
    }
    if !app.ui.status.is_empty() {
        app.ui.leo_pie_message = app.ui.status.clone();
    } else if let Some(on) = crate::menus::checked(app, &slot.command, &slot.params) {
        let scope = if slot.command == "perspective.grid.snap" {
            "（仅用于透视网格）"
        } else {
            ""
        };
        app.ui.leo_pie_message = format!(
            "{}：{}{}",
            slot.name,
            if on { "已开启" } else { "已关闭" },
            scope
        );
        app.status(app.ui.leo_pie_message.clone());
    } else {
        app.ui.leo_pie_message = format!("已调用：{}", slot.name);
    }
}

/// Keyboard events are intercepted *before* VectorCraft's regular keymap.
/// Never trigger over a focused editor, dialog, IME or command palette.
pub fn keyboard_frame(app: &mut VectorcraftApp, ctx: &egui::Context) -> bool {
    // An open Pie Menu editor is a NON-MODAL configuration panel: the user
    // must be able to try gestures on the document without closing it.
    // The focus guard below still prevents shortcut activation while the
    // user types into any of its name, command, search or JSON fields.
    if app.ui.dialog.is_some()
        || app.ui.palette_open
        || ctx.egui_wants_keyboard_input()
        || app.session.tool_wants_text()
        || app.session.tool_composing()
    {
        return false;
    }
    if let Some(open) = app.ui.leo_pie_open.clone() {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            app.ui.leo_pie_open = None;
            return true;
        }
        if ctx.input(|i| i.key_pressed(Key::Backspace)) {
            back(app, open);
            return true;
        }
        // Optional Blender-style keyboard chord: hold the key, move the
        // cursor toward a sector, release the same key to execute.
        if let Some(key) = open.key {
            let release_enabled = app
                .ui
                .leo_pies
                .get(open.menu)
                .is_some_and(|m| m.keyboard_release_select);
            if release_enabled && ctx.input(|i| i.key_released(key)) {
                let at = ctx.input(|i| i.pointer.hover_pos());
                let threshold = app.ui.leo_pies[open.menu].flick_threshold;
                let slot = at
                    .and_then(|p| open.sector(p, threshold))
                    .or(open.last_sector);
                if let Some(slot) = slot {
                    select_slot(app, open.clone(), slot, at.unwrap_or(open.origin));
                } else {
                    app.ui.leo_pie_open = None;
                }
                return true;
            }
        }
        // While the menu is open, don't let tool-specific keyboard bindings
        // alter the active document under the menu.
        return true;
    }
    let found = ctx.input(|i| {
        i.events.iter().find_map(|e| {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = e
            {
                app.ui
                    .leo_pies
                    .iter()
                    .enumerate()
                    .find(|(_, p)| {
                        keyboard_binding(&p.keyboard, *key, *modifiers)
                            && (p.allow_keyboard_override
                                || (normalize(&p.keyboard) != "SHIFT+SPACE"
                                    && crate::shortcut_editor::conflicts_with(
                                        &app.ui.shortcut_overrides,
                                        &p.keyboard,
                                        "ui.leoPieSettings",
                                    )
                                    .is_empty()))
                    })
                    .map(|(idx, _)| (idx, *key, *modifiers))
            } else {
                None
            }
        })
    });
    if let Some((menu, key, mods)) = found {
        let origin = ctx
            .input(|i| i.pointer.hover_pos())
            .or_else(|| app.canvas_rect.map(|r| r.center()))
            .unwrap_or(Pos2::new(400.0, 300.0));
        app.ui.leo_pie_open = Some(OpenPie {
            menu,
            origin,
            mouse: false,
            key: Some(key),
            last_sector: None,
            history: vec![],
        });
        ctx.input_mut(|i| {
            i.consume_key(mods, key);
        });
        return true;
    }
    false
}

/// Returns true when the radial overlay owns the canvas and must suppress
/// the ordinary right-click popup and document tool pointer handling.
pub fn mouse_frame(app: &mut VectorcraftApp, ui: &Ui, resp: &egui::Response) -> bool {
    // Don't block canvas mouse shortcuts just because the (non-modal)
    // editor is open. resp.hovered() gates activation to the actual canvas,
    // so editing the form or using its dropdowns never triggers a pie.
    if app.ui.dialog.is_some()
        || app.ui.screen_mode >= 3
        || app.session.tool_wants_text()
        || app.session.tool_composing()
    {
        return false;
    }
    let input = ui.input(|i| (i.pointer.clone(), i.modifiers, i.key_pressed(Key::Escape)));
    let (pointer, mods, escape) = input;
    if let Some(mut active) = app.ui.leo_pie_open.clone() {
        if escape {
            app.ui.leo_pie_open = None;
            return true;
        }
        let threshold = app
            .ui
            .leo_pies
            .get(active.menu)
            .map_or(24.0, |m| m.flick_threshold);
        // hover_pos remains available even after button release; interact_pos
        // may vanish on the release frame, which formerly lost fast flicks.
        let at = pointer.hover_pos().or_else(|| pointer.interact_pos());
        if let Some(p) = at {
            active.last_sector = active.sector(p, threshold);
        }
        if active.mouse && pointer.button_released(PointerButton::Secondary) {
            let slot = active.last_sector;
            if let Some(slot) = slot {
                select_slot(app, active.clone(), slot, at.unwrap_or(active.origin));
            } else {
                app.ui.leo_pie_open = None;
            }
            return true;
        }
        if !active.mouse && pointer.button_pressed(PointerButton::Primary) {
            let radius = app
                .ui
                .leo_pies
                .get(active.menu)
                .map_or(106.0, |m| m.menu_radius);
            if !active.history.is_empty()
                && at.is_some_and(|p| {
                    p.distance(visual_center(ui.clip_rect(), active.origin, radius)) <= 31.0
                })
            {
                back(app, active);
                return true;
            }
            let slot = active.last_sector;
            if let Some(slot) = slot {
                select_slot(app, active.clone(), slot, at.unwrap_or(active.origin));
            } else {
                back(app, active);
            }
            return true;
        }
        if active.mouse && !pointer.button_down(PointerButton::Secondary) {
            app.ui.leo_pie_open = None; // lost focus: never leave a stuck menu
            return true;
        }
        app.ui.leo_pie_open = Some(active);
        return true;
    }
    if !resp.hovered() || !pointer.button_pressed(PointerButton::Secondary) {
        return false;
    }
    if let Some((menu, _)) = app
        .ui
        .leo_pies
        .iter()
        .enumerate()
        .find(|(_, p)| mouse_binding(&p.mouse, mods))
    {
        if let Some(pos) = pointer.interact_pos() {
            app.ui.leo_pie_open = Some(OpenPie {
                menu,
                origin: pos,
                mouse: true,
                key: None,
                last_sector: None,
                history: vec![],
            });
            return true;
        }
    }
    false
}

fn visual_center(clip: Rect, origin: Pos2, radius: f32) -> Pos2 {
    let margin = radius.clamp(70.0, 180.0) + 46.0;
    Pos2::new(
        if clip.width() > 2.0 * margin {
            origin.x.clamp(clip.min.x + margin, clip.max.x - margin)
        } else {
            clip.center().x
        },
        if clip.height() > 2.0 * margin {
            origin.y.clamp(clip.min.y + margin, clip.max.y - margin)
        } else {
            clip.center().y
        },
    )
}

/// Draw above the canvas using egui primitives, without replacing the native
/// application context menu on a normal (unmodified) RMB click.
pub fn paint(ui: &Ui, app: &VectorcraftApp) {
    let Some(open) = &app.ui.leo_pie_open else {
        return;
    };
    let Some(cfg) = app.ui.leo_pies.get(open.menu) else {
        return;
    };
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let picked = pointer.and_then(|p| open.sector(p, cfg.flick_threshold));
    let radius = cfg.menu_radius.clamp(70.0, 180.0);
    let painter = ui.painter();
    // Preserve the actual gesture origin; only the VISUAL layout is clamped
    // away from viewport edges so all eight options stay visible.
    let center = visual_center(ui.clip_rect(), open.origin, radius);
    // Keep the visual weight low. Distinct command labels and a fast
    // directional highlight matter more than a giant opaque 8-slice disc.
    painter.circle_filled(
        center,
        31.0,
        Color32::from_rgba_unmultiplied(24, 30, 40, 226),
    );
    painter.circle_stroke(
        center,
        31.0,
        Stroke::new(1.4, Color32::from_rgb(112, 138, 170)),
    );
    let active_color = Color32::from_rgb(115, 182, 250);
    for sector in 0..8 {
        let a = sector as f32 * std::f32::consts::FRAC_PI_4;
        let direction = Vec2::angled(a);
        let at = center + direction * radius;
        let is_active = picked == Some(sector);
        let name = cfg.slots.get(sector).map(|x| x.name.as_str()).unwrap_or("");
        let is_empty = name.trim().is_empty();
        let submenu = cfg.slots.get(sector).is_some_and(|s| s.submenu.is_some());
        let display = if submenu {
            format!("{name} ▸")
        } else {
            name.to_owned()
        };
        let label = if is_empty {
            "＋ 添加"
        } else {
            display.as_str()
        };
        let text_size = if radius < 88.0 { 11.0 } else { 13.0 };
        // Compact, elongated pills mimic Blender's readable option layout.
        let text_width =
            (label.chars().count() as f32 * text_size * 0.77 + 22.0).clamp(66.0, 148.0);
        let box_rect = Rect::from_center_size(at, Vec2::new(text_width, 29.0));
        let fill = if is_active {
            Color32::from_rgb(65, 106, 157)
        } else {
            Color32::from_rgba_unmultiplied(38, 46, 58, 235)
        };
        painter.rect_filled(box_rect, egui::CornerRadius::same(7), fill);
        painter.rect_stroke(
            box_rect,
            egui::CornerRadius::same(7),
            Stroke::new(
                if is_active { 1.8 } else { 0.7 },
                if is_active {
                    active_color
                } else {
                    Color32::from_rgb(69, 82, 100)
                },
            ),
            egui::StrokeKind::Inside,
        );
        if is_active {
            painter.line_segment(
                [
                    center + direction * 34.0,
                    at - direction * (text_width * 0.5 + 4.0),
                ],
                Stroke::new(2.0, active_color),
            );
        }
        painter.text(
            at,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(text_size),
            if is_empty {
                Color32::from_rgb(150, 162, 178)
            } else {
                Color32::WHITE
            },
        );
    }
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        if open.history.is_empty() {
            "Leo"
        } else {
            "↩ 返回"
        },
        egui::FontId::proportional(12.5),
        Color32::WHITE,
    );
    // No unconditional request_repaint(). Native pointer input already
    // requests another frame; forcing redraw every frame wastes CPU/GPU.
}

fn preset_definitions() -> [(&'static str, [&'static str; 8]); 3] {
    [
        (
            "视图与网格",
            [
                "view.grid",
                "view.snapToGrid",
                "view.smartGuides",
                "view.guides",
                "view.rulers",
                "view.transparencyGrid",
                "view.zoomIn",
                "view.zoomOut",
            ],
        ),
        (
            "编组与排列",
            [
                "object.group",
                "object.ungroup",
                "object.arrange.bringToFront",
                "object.arrange.bringForward",
                "object.arrange.sendBackward",
                "object.arrange.sendToBack",
                "select.none",
                "tool:selection",
            ],
        ),
        (
            "常用编辑工具",
            [
                "edit.undo",
                "edit.redo",
                "edit.cut",
                "edit.copy",
                "edit.paste",
                "edit.pasteInPlace",
                "tool:directSelection",
                "tool:selection",
            ],
        ),
    ]
}

/// Match familiar Chinese verbs as well as the official catalog terminology.
fn search_term(s: &str) -> String {
    s.to_lowercase()
        .replace("制作", "创建")
        .replace("建立", "创建")
        .replace("剪裁蒙版", "剪切蒙版")
        .replace("裁剪蒙版", "剪切蒙版")
        .replace("遮罩", "蒙版")
        .replace("解除", "释放")
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(*c, '›' | '/' | '·' | '…'))
        .collect()
}
fn command_matches(label: &str, id: &str, query: &str) -> bool {
    let aliases = match id {
        "object.clippingMask.make" => {
            "制作剪切蒙版 创建剪切蒙版 建立剪切蒙版 剪裁蒙版 clipping mask make"
        }
        "object.clippingMask.release" => {
            "释放剪切蒙版 解除剪切蒙版 取消剪切蒙版 clipping mask release"
        }
        "object.clippingMask.editContents" => "编辑蒙版内容 编辑内容",
        "object.clippingMask.editMask" => "编辑剪切路径 编辑蒙版",
        "view.snapToGrid" => "对齐网格 网格吸附 吸附到网格 普通网格",
        "perspective.grid.snap" => "透视网格吸附 透视对齐网格",
        "object.ungroup" => "取消编组 解除编组 拆组",
        _ => "",
    };
    let haystack = search_term(&format!("{label} {id} {aliases}"));
    query
        .split_whitespace()
        .all(|word| haystack.contains(&search_term(word)))
}

fn builtin_commands() -> Vec<(String, String)> {
    let mut v: Vec<_> = crate::palette::items()
        .into_iter()
        .map(|(label, id, _)| (label, id))
        .collect();
    for (id, label) in crate::state::all_panels() {
        v.push((format!("面板 › {label}"), format!("panel:{id}")));
    }
    // Use the official Simplified Chinese catalog independently of app language.
    let lang = crate::i18n::Lang::from_code("zh-hans").unwrap_or(crate::i18n::current());
    for (label, id) in &mut v {
        let english = label.clone();
        let translated = label
            .split(" › ")
            .map(|part| crate::i18n::tr(lang, part))
            .collect::<Vec<_>>()
            .join(" › ");
        let chinese = match id.as_str() {
            "object.clippingMask.make" => "创建剪切蒙版",
            "object.clippingMask.release" => "释放剪切蒙版",
            "view.grid" => "显示网格",
            "view.snapToGrid" => "普通网格吸附（开关）",
            "perspective.grid.snap" => "透视网格吸附（开关）",
            "perspective.grid.show" => "显示透视网格",
            "perspective.grid.rulers" => "显示透视网格标尺",
            "view.smartGuides" => "智能参考线",
            "view.guides" => "显示／隐藏参考线",
            "view.rulers" => "显示标尺",
            "object.group" => "编组",
            "object.ungroup" => "取消编组",
            _ => translated.as_str(),
        };
        *label = if chinese == english {
            english
        } else {
            format!("{chinese} / {english}")
        };
    }
    // Deduplicate by command ID before sorting by the visible name.
    let mut seen = std::collections::HashSet::new();
    v.retain(|(_, id)| seen.insert(id.clone()));
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.dedup_by(|a, b| a.1 == b.1);
    v
}
/// Preserve native menu parameters (alignment direction, document color mode,
/// effects etc.), rather than silently binding all variants to an empty object.
fn menu_variants() -> Vec<(String, String, serde_json::Value)> {
    fn collect(
        items: &[crate::menus::Item],
        path: &str,
        out: &mut Vec<(String, String, serde_json::Value)>,
    ) {
        let lang = crate::i18n::Lang::from_code("zh-hans").unwrap_or(crate::i18n::current());
        for item in items {
            match item {
                crate::menus::Item::Sub(label, children) => {
                    collect(
                        children,
                        &format!("{path} › {}", crate::i18n::tr(lang, label)),
                        out,
                    );
                }
                crate::menus::Item::Cmd(label, id, params)
                    if params.as_object().is_some_and(|p| !p.is_empty()) =>
                {
                    let (command, params) = crate::menus::click_target(label, id, params);
                    let translated = crate::i18n::tr(lang, label);
                    out.push((format!("{path} › {translated} / {label}"), command, params));
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    let lang = crate::i18n::Lang::from_code("zh-hans").unwrap_or(crate::i18n::current());
    for (title, items) in crate::menus::menu_tree() {
        collect(&items, crate::i18n::tr(lang, title), &mut out);
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|(_, id, p)| seen.insert((id.clone(), p.to_string())));
    out
}

/// Geometry of the eight slots in the same order as the *runtime* pie:
/// Right, Down-Right, Down, Down-Left, Left, Up-Left, Up, Up-Right.
/// egui's Y grows downward; this is the exact angle convention of OpenPie.
fn editor_slot_position(center: Pos2, radius: f32, slot: usize) -> Pos2 {
    center + Vec2::angled(slot as f32 * std::f32::consts::FRAC_PI_4) * radius
}

/// Real, clickable 8-way layout (not a text-only representation).
/// All operations still edit the very same PieConfig/slots as list mode.
fn editor_radial_slots(
    ui: &mut Ui,
    config: &PieConfig,
    selected: &mut usize,
    test_slot: &mut Option<usize>,
) {
    let width = ui.available_width().clamp(282.0, 432.0);
    let (area, _) = ui.allocate_exact_size(Vec2::new(width, 326.0), egui::Sense::hover());
    let center = area.center();
    let radius = (width * 0.312).clamp(96.0, 132.0);
    let pill_width = (width * 0.275).clamp(84.0, 119.0);
    let painter = ui.painter();
    painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_gray(77)));
    // Full disk background is intentionally avoided: clarity and resemblance
    // to the lightweight radial command menu matter more than decoration.
    let directions = ["→", "↘", "↓", "↙", "←", "↖", "↑", "↗"];
    for i in 0..8 {
        let at = editor_slot_position(center, radius, i);
        let rect = Rect::from_center_size(at, Vec2::new(pill_width, 37.0));
        let response = ui.interact(
            rect,
            ui.id().with(("leo-pie-editor-slot", i)),
            egui::Sense::click(),
        );
        if response.clicked() {
            *selected = i;
        }
        let active = *selected == i;
        let is_empty =
            config.slots[i].command.trim().is_empty() && config.slots[i].submenu.is_none();
        let fill = if active {
            Color32::from_rgb(143, 75, 65)
        } else if response.hovered() {
            Color32::from_rgb(64, 82, 105)
        } else {
            Color32::from_rgb(44, 53, 67)
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), fill);
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(8),
            Stroke::new(
                if active { 1.6 } else { 0.8 },
                if active {
                    Color32::from_rgb(231, 162, 140)
                } else {
                    Color32::from_rgb(79, 94, 110)
                },
            ),
            egui::StrokeKind::Inside,
        );
        let name: String = if is_empty {
            "＋ 添加".to_owned()
        } else if config.slots[i].name.trim().is_empty() {
            "未命名".to_owned()
        } else {
            let name = config.slots[i].name.trim();
            let text: String = name.chars().take(5).collect();
            if name.chars().count() > 5 {
                format!("{text}…")
            } else {
                text
            }
        };
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} {}", directions[i], name),
            egui::FontId::proportional(12.2),
            if is_empty {
                Color32::from_rgb(160, 177, 193)
            } else {
                Color32::WHITE
            },
        );
        response.on_hover_text(format!(
            "方向：{} · {}\n命令：{}\n点击选择此方向",
            directions[i],
            if config.slots[i].name.trim().is_empty() {
                "空槽位"
            } else {
                &config.slots[i].name
            },
            if is_empty {
                "未绑定"
            } else {
                &config.slots[i].command
            }
        ));
    }
    // Center previews the active configuration; click explicitly tests
    // the currently selected direction, without closing this editor.
    let middle = Rect::from_center_size(center, Vec2::new(148.0, 74.0));
    let response = ui.interact(
        middle,
        ui.id().with("leo-pie-center-test"),
        egui::Sense::click(),
    );
    ui.painter().rect_filled(
        middle,
        egui::CornerRadius::same(14),
        if response.hovered() {
            Color32::from_rgb(48, 72, 96)
        } else {
            Color32::from_rgb(30, 39, 52)
        },
    );
    ui.painter().rect_stroke(
        middle,
        egui::CornerRadius::same(14),
        Stroke::new(1.2, Color32::from_rgb(110, 141, 175)),
        egui::StrokeKind::Inside,
    );
    let name: String = config.name.chars().take(9).collect();
    ui.painter().text(
        center + Vec2::new(0.0, -19.0),
        egui::Align2::CENTER_CENTER,
        if name.is_empty() { "Leo Design" } else { &name },
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    ui.painter().text(
        center + Vec2::new(0.0, 2.0),
        egui::Align2::CENTER_CENTER,
        format!("已选方向 {}", *selected + 1),
        egui::FontId::proportional(11.0),
        Color32::from_rgb(168, 187, 213),
    );
    ui.painter().text(
        center + Vec2::new(0.0, 22.0),
        egui::Align2::CENTER_CENTER,
        "▶ 测试当前功能",
        egui::FontId::proportional(11.5),
        Color32::from_rgb(153, 212, 252),
    );
    if response.clicked() && !config.slots[*selected].command.trim().is_empty() {
        *test_slot = Some(*selected);
    }
    response.on_hover_text("点击运行当前选中的槽位命令；不会关闭编辑器");
}

const MAX_CONFIG_BYTES: usize = 2_000_000;
fn decode_config_file(bytes: &[u8]) -> Result<Vec<PieConfig>, String> {
    if bytes.len() > MAX_CONFIG_BYTES {
        return Err("配置文件超过 2MB".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| format!("配置文件不是 UTF-8：{e}"))?;
    let menus: Vec<PieConfig> = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("配置文件格式错误：{e}"))?;
    validate(&menus)?;
    Ok(menus)
}
fn export_config_file(
    app: &mut VectorcraftApp,
    menus: &[PieConfig],
) -> Result<Option<String>, String> {
    validate(menus)?;
    let bytes = serde_json::to_vec_pretty(menus).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_CONFIG_BYTES {
        return Err("配置文件超过 2MB，无法导出".into());
    }
    let pick = crate::FilePick {
        filters: vec![crate::FilePick::filter("Leo Design 饼菜单配置", &["json"])],
        ..crate::FilePick::named("Leo_Design_Pie_Menu.json")
    };
    let Some(path) = crate::picks::save(app, &pick) else {
        return Ok(None);
    };
    let write = app.services.write.as_mut().ok_or("无文件写入服务")?;
    write(&path, &bytes)?;
    Ok(Some(format!("已导出 {} 个饼菜单：{path}", menus.len())))
}
fn import_config_file(app: &mut VectorcraftApp) -> Result<Option<String>, String> {
    let pick = crate::FilePick {
        filters: vec![crate::FilePick::filter("Leo Design 饼菜单配置", &["json"])],
        ..Default::default()
    };
    let Some(path) = crate::picks::open(app, &pick) else {
        return Ok(None);
    };
    let read = app.services.read.as_ref().ok_or("无文件读取服务")?;
    let bytes = read(&path)?;
    // Do not replace any preferences until the complete file validates.
    let menus = decode_config_file(&bytes)?;
    let count = menus.len();
    app.ui.leo_pies = menus;
    app.ui.leo_pie_selected = 0;
    app.ui.leo_pie_slot = 0;
    app.ui.leo_pie_open = None;
    app.ui.leo_pie_recording.clear();
    app.prefs_changed = true;
    Ok(Some(format!("已导入 {count} 个饼菜单，快捷键和方向已恢复")))
}
fn file_result(app: &mut VectorcraftApp, result: Result<Option<String>, String>) {
    match result {
        Ok(Some(message)) => app.ui.leo_pie_message = message,
        Ok(None) => {} // Cancelled or asynchronous dialog still open.
        Err(error) => app.ui.leo_pie_message = format!("配置操作失败：{error}"),
    }
}

pub fn editor(app: &mut VectorcraftApp, ctx: &egui::Context) {
    if !app.ui.leo_pie_editor {
        return;
    }
    let mut menus = app.ui.leo_pies.clone();
    let mut index = app.ui.leo_pie_selected.min(menus.len().saturating_sub(1));
    let mut sector = app.ui.leo_pie_slot.min(7);
    let mut query = app.ui.leo_pie_query.clone();
    let mut recording = app.ui.leo_pie_recording.clone();
    let mut import = app.ui.leo_pie_import.clone();
    let mut status = app.ui.leo_pie_message.clone();
    let mut open = true;
    let mut changed = false;
    let mut test_slot = None;
    let mut file_action = 0;
    let commands = builtin_commands();
    let variants = menu_variants();
    egui::Window::new("Leo Design 饼菜单编辑器")
        .id(egui::Id::new("leo-pie-config-window"))
        .open(&mut open).default_pos([60.0,70.0]).default_size([710.0,540.0])
        .resizable(true).vscroll(true).show(ctx,|ui|{
        ui.horizontal_wrapped(|ui|{
            ui.label("饼菜单");
            egui::ComboBox::from_id_salt("leo-pie-current").selected_text(&menus[index].name).show_ui(ui,|ui|{
                for (i,m) in menus.iter().enumerate(){ui.selectable_value(&mut index,i,&m.name);}
            });
            if ui.button("＋ 新建菜单").clicked()&&menus.len()<24 {
                let mut m=PieConfig::default();
                m.name=format!("自定义菜单 {}",menus.len()+1);
                m.mouse.clear();
                menus.push(m);index=menus.len()-1;changed=true;
            }
            if ui.button("删除当前菜单").clicked()&&menus.len()>1 {
                remove_menu(&mut menus,index);index=0;changed=true;
            }
        });
        ui.menu_button("＋ 从常用预设新建", |ui| {
            ui.label("新建独立菜单，保留现有快捷键和槽位");
            for (title, ids) in preset_definitions() {
                if ui.add_enabled(menus.len() < 24, egui::Button::new(title)).clicked() {
                    let mut preset = PieConfig::default();
                    preset.name = title.into();
                    preset.mouse.clear();
                    for (slot, id) in preset.slots.iter_mut().zip(ids) {
                        if let Some((label, command)) = commands.iter().find(|(_, command)| command == id) {
                            slot.name = label.split(" / ").next().unwrap_or(label).rsplit(" › ").next().unwrap_or(label).into();
                            slot.command = command.clone();
                        }
                    }
                    menus.push(preset);
                    index = menus.len() - 1;
                    sector = 0;
                    changed = true;
                    status = "预设已新建，请设置触发快捷键；原菜单已保留".into();
                    ui.close();
                }
            }
        });
        if ui.add_enabled(menus.len()<24, egui::Button::new("＋ 为当前方向新建下级菜单")).clicked() {
            let child = menus.len();
            let name = format!("下级菜单 {}",child+1);
            menus[index].slots[sector] = PieSlot { name:name.clone(), submenu:Some(child), ..Default::default() };
            menus.push(PieConfig { name, mouse:String::new(), ..Default::default() });
            index=child; sector=0; changed=true;
        }
        let menu_names: Vec<_> = menus.iter().enumerate().map(|(i,m)|(i,m.name.clone())).collect();
        let menu=&mut menus[index];
        ui.horizontal(|ui|{
            ui.label("菜单名称");
            changed|=ui.text_edit_singleline(&mut menu.name).changed();
        });
        ui.separator();
        ui.horizontal_wrapped(|ui|{
            ui.label(egui::RichText::new("实时编辑：无需关闭此窗口，在画布上直接测试快捷键/甩动。").strong());
            ui.menu_button("单击测试命令",|ui|{
                for (i,slot) in menu.slots.iter().enumerate() {
                    if (!slot.command.is_empty() || slot.submenu.is_some())
                        && ui.button(format!("{}  {}",i+1,slot.name)).clicked() {
                        test_slot=Some(i);
                        ui.close();
                    }
                }
            });
        });
        ui.label(egui::RichText::new("若正在输入文字，先点击画布空白处解除焦点，再按键盘快捷键。").small().weak());
        ui.label("触发快捷键（可以输入，也可以录制，不限制常用字母与功能键）");
        ui.horizontal(|ui|{
            ui.label("键盘");
            changed|=ui.text_edit_singleline(&mut menu.keyboard).changed();
            if ui.button(if recording=="keyboard"{"正在录制… Esc 取消"}else{"录制键盘"}).clicked(){recording="keyboard".into();}
        });
        ui.horizontal(|ui|{
            ui.label("右键");
            changed|=ui.text_edit_singleline(&mut menu.mouse).changed();
            if ui.button(if recording=="mouse"{"正在录制… Esc 取消"}else{"录制鼠标"}).clicked(){recording="mouse".into();}
        });
        if recording=="keyboard" {
            if let Some((key,mods))=ctx.input(|i|i.events.iter().find_map(|e|{
                match e { egui::Event::Key{key,pressed:true,modifiers,..}=>Some((*key,*modifiers)),_=>None }
            })) {
                let k=format!("{key:?}");
                if key==Key::Escape {recording.clear();}
                else if !(k.starts_with("Control") || k.starts_with("Shift")
                    || k.starts_with("Alt") || k.starts_with("Meta")
                    || k.starts_with("Super") || k.starts_with("Command")) {
                    menu.keyboard=keyboard_text(key,mods);recording.clear();changed=true;
                }
            }
        }
        if recording=="mouse" && ctx.input(|i|i.pointer.button_pressed(PointerButton::Secondary)) {
            let mods=ctx.input(|i|i.modifiers);
            if mods.shift||mods.alt||mods.ctrl {
                menu.mouse=mouse_text(mods);changed=true;
            }else {status="普通右键保留原有菜单，请加 Shift、Alt 或 Ctrl".into();}
            recording.clear();
        }
        ui.label(egui::RichText::new("例如 Shift+Space、Shift+Alt+RMB、Shift+Ctrl+RMB；普通右键不改变。").small().weak());
        ui.separator();
        ui.label("Blender 式快捷手势");
        ui.horizontal(|ui|{
            ui.label("甩动灵敏度");
            changed |= ui.add(egui::Slider::new(&mut menu.flick_threshold,8.0..=64.0)
                .suffix(" pt")).changed();
            ui.label("菜单紧凑度");
            changed |= ui.add(egui::Slider::new(&mut menu.menu_radius,70.0..=180.0)
                .suffix(" pt")).changed();
        });
        changed |= ui.checkbox(&mut menu.keyboard_release_select,
            "键盘快捷键：按住 → 甩动 → 松开执行（Blender 模式）").changed();
        ui.label(egui::RichText::new("右键手势始终为按住 → 甩动 → 松开，越过灵敏度距离就能选中；无需甩到菜单文字上。").small().weak());
        if !menu.keyboard.trim().is_empty() {
            if !valid_binding(&menu.keyboard,false) {
                ui.colored_label(Color32::from_rgb(250,153,109),"键盘快捷键格式错误，请使用如 Shift+D、Ctrl+Shift+D 或 Shift+Space");
            } else {
                ui.label(egui::RichText::new(format!("✓ 支持的键盘组合：{}",menu.keyboard)).color(Color32::from_rgb(134,212,158)));
            }
        }
        let key_conflicts=crate::shortcut_editor::conflicts_with(&app.ui.shortcut_overrides,&menu.keyboard,"ui.leoPieSettings");
        let conflict=normalize(&menu.keyboard)=="SHIFT+SPACE" || !key_conflicts.is_empty();
        if conflict {
            ui.colored_label(Color32::from_rgb(250,176,93),
                format!("可能与原生导航/快捷键冲突：{}",
                    if key_conflicts.is_empty() {"画布空格导航".to_owned()}else{key_conflicts.join(", ")}));
            changed|=ui.checkbox(&mut menu.allow_keyboard_override,"允许饼菜单覆盖此原生键位").changed();
        }
        ui.separator();
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new("八方向功能").strong());
            ui.separator();
            ui.selectable_value(&mut app.ui.leo_pie_list_view, false, "◉ 圆形视图");
            ui.selectable_value(&mut app.ui.leo_pie_list_view, true, "☷ 列表视图");
            ui.label(egui::RichText::new("点击方向选择，再从右侧添加功能").small().weak());
        });
        ui.columns(2,|cols|{
            cols[0].vertical(|ui|{
                if app.ui.leo_pie_list_view {
                    ui.label("八个方向（列表）");
                    for (i,label) in ["→ 右","↘ 右下","↓ 下","↙ 左下","← 左","↖ 左上","↑ 上","↗ 右上"].iter().enumerate() {
                        let title=format!("{}  {}",label,if menu.slots[i].command.is_empty() && menu.slots[i].submenu.is_none(){"＋ 添加"}else{&menu.slots[i].name});
                        ui.selectable_value(&mut sector,i,title);
                    }
                } else {
                    editor_radial_slots(ui, menu, &mut sector, &mut test_slot);
                }
                ui.separator();
                ui.label(egui::RichText::new(format!("编辑当前方向：{}",["→ 右","↘ 右下","↓ 下","↙ 左下","← 左","↖ 左上","↑ 上","↗ 右上"][sector])).strong());
                ui.label("显示名称");
                changed|=ui.text_edit_singleline(&mut menu.slots[sector].name).changed();
                let old_target=menu.slots[sector].submenu;
                egui::ComboBox::from_id_salt("leo-pie-submenu-target")
                    .selected_text(old_target.and_then(|t|menu_names.get(t)).map_or("执行功能".to_owned(),|(_,n)|format!("下级：{n}")))
                    .show_ui(ui,|ui|{
                        ui.selectable_value(&mut menu.slots[sector].submenu,None,"执行功能");
                        for (target,name) in &menu_names {
                            if *target!=index { ui.selectable_value(&mut menu.slots[sector].submenu,Some(*target),format!("打开下级：{name}")); }
                        }
                    });
                if old_target!=menu.slots[sector].submenu {
                    if let Some(target)=menu.slots[sector].submenu {
                        menu.slots[sector].name=menu_names.get(target).map(|(_,n)|n.clone()).unwrap_or_default();
                        menu.slots[sector].command.clear(); menu.slots[sector].params=serde_json::Value::Null;
                    }
                    changed=true;
                }
                ui.label("下级菜单：进入后单击选择；点中心或 Backspace 返回，Esc 关闭。");
                ui.label("命令 ID（高级）");
                if ui.text_edit_singleline(&mut menu.slots[sector].command).changed() {
                    menu.slots[sector].params = serde_json::Value::Null;
                    menu.slots[sector].submenu = None;
                    changed = true;
                }
                if menu.slots[sector].command == "perspective.grid.snap" {
                    ui.label("这是透视网格吸附，不影响普通网格。");
                    if ui.button("改为普通网格吸附").clicked() {
                        menu.slots[sector] = PieSlot { name:"普通网格吸附".into(), command:"view.snapToGrid".into(), ..Default::default() };
                        changed = true;
                    }
                }
                if ui.button("清空此方向").clicked() {
                    menu.slots[sector]=PieSlot::default();changed=true;
                }
            });
            cols[1].vertical(|ui|{
                ui.label("中文功能列表 · 点击添加到当前方向");
                ui.label(egui::RichText::new("支持中文、英文和命令 ID；例如：网格 / grid").small().weak());
                ui.text_edit_singleline(&mut query);
                let matches: Vec<_> = commands.iter()
                    .filter(|(name,id)| !variants.iter().any(|(_,variant,_)| variant == id) && command_matches(name,id,&query)).collect();
                let parameter_matches: Vec<_> = variants.iter()
                    .filter(|(name,id,_)| command_matches(name,id,&query)).collect();
                ui.label(egui::RichText::new(format!("找到 {} 个功能；点击 ＋ 替换当前选中方向", matches.len()+parameter_matches.len())).small().weak());
                if matches.is_empty() && parameter_matches.is_empty() {
                    ui.label("没有匹配的功能。试试更短的关键词，例如：剪切蒙版、网格。");
                    if ui.button("清空搜索，显示全部功能").clicked() { query.clear(); }
                }
                egui::ScrollArea::vertical().max_height(340.0).show(ui,|ui|{
                    for (label,id,params) in parameter_matches {
                        if ui.button(format!("＋ {label}")).on_hover_text(format!("{id} {params}")).clicked() {
                            menu.slots[sector] = PieSlot { name:label.split(" / ").next().unwrap_or(label).rsplit(" › ").next().unwrap_or(label).into(), command:id.clone(), params:params.clone(), ..Default::default() };
                            changed = true;
                        }
                    }
                    for (label,id) in matches {
                        if ui.button(format!("＋ {label}")).on_hover_text(id).clicked(){
                            menu.slots[sector]=PieSlot{name:label.split(" / ").next().unwrap_or(label).rsplit(" › ").next().unwrap_or(label).to_owned(),command:id.clone(), ..PieSlot::default()};
                            changed=true;
                        }
                    }
                });
            });
        });
        ui.separator();
        ui.horizontal_wrapped(|ui|{
            if ui.button("导出配置文件…").on_hover_text("保存全部饼菜单、方向、功能、参数和触发快捷键到 JSON 文件").clicked() { file_action=1; }
            if ui.button("导入配置文件…").on_hover_text("选择 JSON 配置文件；验证通过后替换全部饼菜单设置").clicked() { file_action=2; }
        });
        ui.label(egui::RichText::new("备份全部饼菜单及快捷键；导入会替换当前饼菜单配置。").small().weak());
        ui.collapsing("高级：复制／粘贴 JSON 配置", |ui| {
        ui.horizontal(|ui|{
            if ui.button("复制 JSON 配置").clicked(){
                if let Ok(raw)=serde_json::to_string_pretty(&menus){ctx.copy_text(raw);status="已将配置复制到剪贴板".into();}
            }
            if ui.button("恢复默认菜单").clicked(){menus=defaults();index=0;changed=true;}
        });
        ui.label("导入 JSON：将导出的配置粘贴到下方，再点导入");
        ui.add(egui::TextEdit::multiline(&mut import).desired_rows(2).hint_text("粘贴 JSON 配置"));
        if ui.button("验证并导入 JSON").clicked() {
            if import.trim().is_empty() {
                status="请先粘贴导出的 JSON 配置".into();
            } else {
            match serde_json::from_str::<Vec<PieConfig>>(&import)
                .map_err(|e|e.to_string()).and_then(|x|{validate(&x)?;Ok(x)}) {
                Ok(v)=>{menus=v;index=0;changed=true;status="导入成功".into();}
                Err(e)=>status=format!("导入失败：{e}"),
            }
            }
        }
        });
        if let Err(e)=validate(&menus){ui.colored_label(Color32::from_rgb(255,130,100),format!("配置冲突：{e}"));}
        if !status.is_empty(){ui.label(status.as_str());}
    });
    if changed {
        // Keep in-progress text edits (including temporarily invalid strings),
        // but validate before interpreting any gesture or importing JSON.
        app.ui.leo_pie_open = None;
        app.ui.leo_pies = menus;
        app.prefs_changed = true;
    }
    app.ui.leo_pie_selected = index;
    app.ui.leo_pie_slot = sector;
    app.ui.leo_pie_query = query;
    app.ui.leo_pie_recording = recording;
    app.ui.leo_pie_import = import;
    app.ui.leo_pie_message = status;
    app.ui.leo_pie_editor = open;
    // Native async file pickers replay this callback after a path arrives.
    // Dispatch after committing live edits to avoid replacing imported state.
    if file_action == 1 {
        let snapshot = app.ui.leo_pies.clone();
        crate::picks::button(app, move |app| {
            let result = export_config_file(app, &snapshot);
            file_result(app, result);
        });
    } else if file_action == 2 {
        crate::picks::button(app, |app| {
            let result = import_config_file(app);
            file_result(app, result);
        });
    }
    // A command test is performed only after the live edits above have been
    // committed to UiState; the editor stays open during the action.
    if let Some(slot) = test_slot {
        run_slot(app, index, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submenu_navigation_back_and_leaf_execution() {
        let mut app = test_app();
        app.ui.leo_pies[0].slots[0] = PieSlot {
            name: "视图".into(),
            submenu: Some(1),
            ..Default::default()
        };
        app.ui.leo_pies[1].slots[0] = PieSlot {
            name: "网格".into(),
            command: "view.snapToGrid".into(),
            ..Default::default()
        };
        let origin = Pos2::new(300.0, 200.0);
        let root = OpenPie {
            menu: 0,
            origin,
            mouse: true,
            key: None,
            last_sector: Some(0),
            history: vec![],
        };
        let before = app.ui.view.snap_to_grid;
        select_slot(&mut app, root.clone(), 0, origin);
        let child = app.ui.leo_pie_open.clone().unwrap();
        assert_eq!(child.menu, 1);
        assert!(!child.mouse && child.key.is_none());
        assert_eq!(app.ui.view.snap_to_grid, before);
        back(&mut app, child.clone());
        assert_eq!(app.ui.leo_pie_open.as_ref().unwrap().menu, 0);
        select_slot(&mut app, child, 0, origin);
        assert!(app.ui.leo_pie_open.is_none());
        assert_ne!(app.ui.view.snap_to_grid, before);
    }
    #[test]
    fn submenu_config_roundtrip_cycles_and_delete() {
        let mut menus = defaults();
        menus[0].slots[0].submenu = Some(1);
        let raw = serde_json::to_string(&menus).unwrap();
        let back: Vec<PieConfig> = serde_json::from_str(&raw).unwrap();
        assert_eq!(menus, back);
        validate(&back).unwrap();
        menus[1].slots[0].submenu = Some(0);
        assert!(validate(&menus).is_err());
        menus[1].slots[0].submenu = None;
        remove_menu(&mut menus, 1);
        assert_eq!(menus[0].slots[0], PieSlot::default());
        validate(&menus).unwrap();
        menus[0].slots[0].submenu = Some(5);
        assert!(validate(&menus).is_err());
    }
    #[test]
    fn submenu_survives_mouse_release_then_click_executes_leaf() {
        let mut app = test_app();
        app.ui.leo_pies[0].slots[0] = PieSlot {
            name: "子菜单".into(),
            submenu: Some(1),
            ..Default::default()
        };
        app.ui.leo_pies[1].slots[0] = PieSlot {
            name: "网格".into(),
            command: "view.snapToGrid".into(),
            ..Default::default()
        };
        let ctx = egui::Context::default();
        let origin = Pos2::new(300.0, 300.0);
        app.ui.leo_pie_open = Some(OpenPie {
            menu: 0,
            origin,
            mouse: true,
            key: None,
            last_sector: None,
            history: vec![],
        });
        let frame = |app: &mut VectorcraftApp, events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 900.0));
                    let response = ui.allocate_rect(rect, egui::Sense::hover());
                    assert!(mouse_frame(app, ui, &response));
                    paint(ui, app);
                },
            );
            output.platform_output.commands.clear();
            output.textures_delta.clear();
        };
        let at = origin + Vec2::new(100.0, 0.0);
        let button = |pos, button, pressed| egui::Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: Modifiers::default(),
        };
        let before = app.ui.view.snap_to_grid;
        frame(
            &mut app,
            vec![
                egui::Event::PointerMoved(at),
                button(at, PointerButton::Secondary, true),
            ],
        );
        frame(&mut app, vec![button(at, PointerButton::Secondary, false)]);
        assert_eq!(app.ui.leo_pie_open.as_ref().unwrap().menu, 1);
        frame(&mut app, vec![]);
        assert_eq!(
            app.ui.view.snap_to_grid, before,
            "release must only open the child"
        );
        let leaf = at + Vec2::new(100.0, 0.0);
        frame(
            &mut app,
            vec![
                egui::Event::PointerMoved(leaf),
                button(leaf, PointerButton::Primary, true),
            ],
        );
        assert!(app.ui.leo_pie_open.is_none());
        assert_ne!(app.ui.view.snap_to_grid, before);
    }

    fn test_app() -> VectorcraftApp {
        let mut app = VectorcraftApp::new(
            vectorcraft_engine::Session::new(),
            crate::Services::default(),
        );
        app.run("file.new", json!({"width":200,"height":200}))
            .unwrap();
        app
    }
    fn execute_test_slot(app: &mut VectorcraftApp, id: &str, params: serde_json::Value) {
        app.ui.leo_pies[0].slots[0] = PieSlot {
            name: id.into(),
            command: id.into(),
            params,
            ..Default::default()
        };
        run_slot(app, 0, 0);
    }
    #[test]
    fn grid_toggles_are_independent_and_report_state() {
        let mut app = test_app();
        let original = app.ui.view.snap_to_grid;
        execute_test_slot(&mut app, "view.snapToGrid", json!({}));
        assert_ne!(app.ui.view.snap_to_grid, original);
        assert!(!app.ui.leo_pie_message.is_empty());
        let normal = app.ui.view.snap_to_grid;
        let perspective = crate::menus::checked(&app, "perspective.grid.snap", &json!({}));
        execute_test_slot(&mut app, "perspective.grid.snap", json!({}));
        assert_ne!(
            crate::menus::checked(&app, "perspective.grid.snap", &json!({})),
            perspective
        );
        assert_eq!(app.ui.view.snap_to_grid, normal);
        let shown = app.ui.view.grid;
        execute_test_slot(&mut app, "view.grid", json!({}));
        assert_ne!(app.ui.view.grid, shown);
        execute_test_slot(&mut app, "view.snapToGrid", json!({}));
        assert_eq!(app.ui.view.snap_to_grid, original);
    }
    #[test]
    fn menu_alignment_preserves_direction_and_moves_objects() {
        let mut app = test_app();
        let a = app
            .run(
                "shape.rectangle",
                json!({"x":10,"y":10,"width":10,"height":10}),
            )
            .unwrap()["id"]
            .clone();
        let b = app
            .run(
                "shape.rectangle",
                json!({"x":70,"y":40,"width":20,"height":10}),
            )
            .unwrap()["id"]
            .clone();
        app.run("select.set", json!({"ids":[a,b]})).unwrap();
        for direction in ["left", "right"] {
            let (_, id, params) = menu_variants()
                .into_iter()
                .find(|(_, id, p)| id == "object.align" && p["horizontal"] == direction)
                .unwrap();
            execute_test_slot(&mut app, &id, params);
            let st = app.session.active().unwrap();
            let bounds = |id: &serde_json::Value| {
                st.doc
                    .bounds_of(&[vectorcraft_doc::NodeId(id.as_u64().unwrap())], false)
                    .unwrap()
            };
            let (aa, bb) = (bounds(&a), bounds(&b));
            if direction == "left" {
                assert!((aa.x0 - bb.x0).abs() < 1e-6);
            } else {
                assert!((aa.x1 - bb.x1).abs() < 1e-6);
            }
        }
    }
    #[test]
    fn grouping_and_missing_selection_feedback() {
        let mut app = test_app();
        let a = app
            .run(
                "shape.rectangle",
                json!({"x":0,"y":0,"width":10,"height":10}),
            )
            .unwrap()["id"]
            .clone();
        let b = app
            .run(
                "shape.rectangle",
                json!({"x":20,"y":0,"width":10,"height":10}),
            )
            .unwrap()["id"]
            .clone();
        app.run("select.set", json!({"ids":[a,b]})).unwrap();
        execute_test_slot(&mut app, "object.group", json!({}));
        assert_eq!(app.session.active().unwrap().selection.objects.len(), 1);
        execute_test_slot(&mut app, "object.ungroup", json!({}));
        assert_eq!(app.session.active().unwrap().selection.objects.len(), 2);
        execute_test_slot(&mut app, "select.none", json!({}));
        execute_test_slot(&mut app, "object.group", json!({}));
        assert!(app.ui.leo_pie_message.contains("当前不可用"));
    }
    #[test]
    fn config_files_roundtrip_and_reject_invalid_without_installing() {
        let menus = defaults();
        let raw = serde_json::to_vec_pretty(&menus).unwrap();
        assert_eq!(decode_config_file(&raw).unwrap(), menus);
        let bom = [b"\xef\xbb\xbf".as_slice(), raw.as_slice()].concat();
        assert_eq!(decode_config_file(&bom).unwrap(), menus);
        assert!(decode_config_file(b"not json").is_err());
        assert!(decode_config_file(b"[]").is_err());
        assert!(decode_config_file(&vec![b' '; MAX_CONFIG_BYTES + 1]).is_err());
        let mut app = VectorcraftApp::new(
            vectorcraft_engine::Session::new(),
            crate::Services::default(),
        );
        let original = app.ui.leo_pies.clone();
        app.services.pick_open = Some(Box::new(|_| Some("bad.json".into())));
        app.services.read = Some(Box::new(|_| Ok(b"[]".to_vec())));
        assert!(import_config_file(&mut app).is_err());
        assert_eq!(app.ui.leo_pies, original);
    }
    #[test]
    fn menu_capture_parameters_and_old_slots_roundtrip() {
        let old: PieSlot =
            serde_json::from_str(r#"{"name":"旧功能","command":"object.group"}"#).unwrap();
        assert!(old.params.is_null());
        let captured = PieSlot {
            name: "左对齐".into(),
            command: "object.align".into(),
            params: json!({"horizontal":"left"}),
            ..Default::default()
        };
        let raw = serde_json::to_string(&captured).unwrap();
        assert_eq!(serde_json::from_str::<PieSlot>(&raw).unwrap(), captured);
        assert_eq!(captured.params["horizontal"], "left");
    }
    #[test]
    fn clipping_mask_search_accepts_common_chinese_names() {
        let commands = builtin_commands();
        let (label, id) = commands
            .iter()
            .find(|(_, id)| id == "object.clippingMask.make")
            .unwrap();
        for query in [
            "制作剪切蒙版",
            "创建剪切蒙版",
            "建立剪切蒙版",
            "剪切蒙版",
            "裁剪蒙版",
            "clipping mask",
            "object.clippingMask.make",
        ] {
            assert!(
                command_matches(label, id, query),
                "unmatched query: {query}"
            );
        }
        assert!(!command_matches(label, id, "不存在的功能"));
        assert!(command_matches(
            "对象 › 剪切蒙版 › 创建",
            id,
            "制作剪切蒙版"
        ));
        assert!(!command_matches("显示网格", "view.grid", "制作剪切蒙版"));
    }
    #[test]
    fn chinese_picker_keeps_english_and_grid_searchable() {
        let commands = builtin_commands();
        let (label, _) = commands.iter().find(|(_, id)| id == "view.grid").unwrap();
        assert!(label.contains("显示网格"));
        assert!(label.to_lowercase().contains("grid"));
        let mut seen = std::collections::HashSet::new();
        assert!(commands.iter().all(|(_, id)| seen.insert(id)));
    }
    #[test]
    fn presets_only_reference_available_native_commands() {
        let commands = builtin_commands();
        for (title, ids) in preset_definitions() {
            for id in ids {
                assert!(
                    commands.iter().any(|(_, command)| command == id),
                    "missing preset command {title}: {id}"
                );
            }
        }
    }
    #[test]
    fn shift_d_and_common_shortcuts_are_accepted() {
        assert!(valid_binding("Shift+D", false));
        assert!(valid_binding("Shift + D", false));
        assert!(valid_binding("Ctrl+Shift+D", false));
        assert!(valid_binding("Alt+Shift+ArrowUp", false));
        assert!(valid_binding("Ctrl+Alt+F12", false));
        assert!(keyboard_binding(
            "Shift+D",
            Key::D,
            Modifiers {
                shift: true,
                ..Default::default()
            }
        ));
        assert!(!keyboard_binding("Shift+D", Key::D, Modifiers::NONE));
        assert!(!valid_binding("Shift", false));
        assert!(!valid_binding("Ctrl+ControlLeft", false));
        assert!(!valid_binding("Shift+ShiftLeft", false));
    }
    #[test]
    fn keyboard_binding_is_exact() {
        assert!(keyboard_binding(
            "Shift+Space",
            Key::Space,
            Modifiers {
                shift: true,
                ..Default::default()
            }
        ));
        assert!(!keyboard_binding(
            "Shift+Space",
            Key::Space,
            Modifiers {
                shift: true,
                ctrl: true,
                ..Default::default()
            }
        ));
    }
    #[test]
    fn live_editor_does_not_modify_mouse_chord_rules() {
        // Canvas hit testing (resp.hovered) keeps the editor's text inputs
        // separate from modifier+RMB gestures.
        assert!(mouse_binding(
            "Ctrl+Shift+RMB",
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            }
        ));
        assert!(!mouse_binding(
            "Ctrl+Shift+RMB",
            Modifiers {
                ctrl: true,
                ..Default::default()
            }
        ));
    }
    #[test]
    fn mouse_chords_do_not_swallow_normal_right_click() {
        assert!(mouse_binding(
            "Shift+Alt+RMB",
            Modifiers {
                shift: true,
                alt: true,
                ..Default::default()
            }
        ));
        assert!(mouse_binding(
            "Shift+Ctrl+RMB",
            Modifiers {
                shift: true,
                ctrl: true,
                ..Default::default()
            }
        ));
        assert!(!mouse_binding("Shift+Alt+RMB", Modifiers::default()));
        assert!(!valid_binding("RMB", true));
    }
    #[test]
    fn duplicates_are_rejected() {
        let mut pies = defaults();
        pies[1].mouse = pies[0].mouse.clone();
        assert!(validate(&pies).is_err());
    }
    #[test]
    fn flick_beyond_old_240pt_limit_still_selects() {
        let o = OpenPie {
            menu: 0,
            origin: Pos2::new(0.0, 0.0),
            mouse: true,
            key: None,
            last_sector: None,
            history: vec![],
        };
        assert_eq!(o.sector(Pos2::new(500.0, 0.0), 24.0), Some(0));
        assert_eq!(o.sector(Pos2::new(12.0, 0.0), 24.0), None);
        assert_eq!(o.sector(Pos2::new(0.0, 400.0), 24.0), Some(2));
        assert_eq!(o.sector(Pos2::new(-400.0, 0.0), 24.0), Some(4));
    }
    #[test]
    fn compact_pie_config_is_backward_compatible() {
        let p: PieConfig = serde_json::from_str(
            r#"{"name":"旧菜单","keyboard":"","mouse":"Shift+Alt+RMB","slots":[]}"#,
        )
        .unwrap();
        assert_eq!(p.flick_threshold, 24.0);
        assert_eq!(p.menu_radius, 106.0);
    }
    #[test]
    fn editor_directions_match_runtime_sector_order() {
        let c = Pos2::new(300.0, 300.0);
        let open = OpenPie {
            menu: 0,
            origin: c,
            mouse: false,
            key: None,
            last_sector: None,
            history: vec![],
        };
        for i in 0..8 {
            let at = editor_slot_position(c, 120.0, i);
            assert_eq!(
                open.sector(at, 24.0),
                Some(i),
                "mismatched direction index {}",
                i
            );
        }
        assert!(
            editor_slot_position(c, 120.0, 6).y < c.y,
            "up should be above center"
        );
        assert!(
            editor_slot_position(c, 120.0, 0).x > c.x,
            "right should be right of center"
        );
    }
    #[test]
    fn json_roundtrip_retains_all_eight_slots() {
        let p = defaults();
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Vec<PieConfig>>(&s).unwrap(), p);
    }
}

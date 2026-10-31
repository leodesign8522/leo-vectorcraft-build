//! Leo Design Pie Menu v0.2.0 — native VectorCraft canvas radial commands.
//! Multiple configurable menus, keyboard and modifier+RMB bindings, a searchable
//! native command picker, and JSON import/export. No OS-global mouse hook.
use egui::{Color32, Key, Modifiers, PointerButton, Pos2, Rect, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::VectorcraftApp;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PieSlot {
    pub name: String,
    pub command: String,
}
impl Default for PieSlot {
    fn default() -> Self {
        Self { name: String::new(), command: String::new() }
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
        ("编组", "object.group"), ("拆组", "object.ungroup"),
        ("镜像", "object.reflect"), ("连接路径", "path.join"),
        ("取消选择", "select.none"), ("对齐", "object.align"),
        ("直接选择", "tool:directSelection"), ("选择", "tool:selection"),
    ].iter().enumerate() {
        a.slots[i] = PieSlot { name: (*name).into(), command: (*cmd).into() };
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
}
impl OpenPie {
    pub fn sector(&self, at: Pos2, threshold: f32) -> Option<usize> {
        let d = at - self.origin;
        // Blender-like flick selection: only a small inner deadzone;
        // NEVER impose a maximum distance (fast gestures often overshoot).
        if d.length_sq() < threshold.clamp(8.0,64.0).powi(2) { return None; }
        let theta = d.y.atan2(d.x);
        Some(((theta + std::f32::consts::FRAC_PI_8).rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::FRAC_PI_4).floor() as usize % 8)
    }
}
fn normalize(s: &str) -> String { s.replace(' ', "").to_ascii_uppercase() }
fn mouse_binding(s: &str, m: Modifiers) -> bool {
    let s = normalize(s);
    if s.is_empty() { return false; }
    let alt = s.contains("ALT+");
    let shift = s.contains("SHIFT+");
    let ctrl = s.contains("CTRL+");
    s.ends_with("RMB") && (alt || shift || ctrl)
        && m.alt == alt && m.shift == shift && m.ctrl == ctrl && !m.mac_cmd
}
fn keyboard_binding(s: &str, key: Key, m: Modifiers) -> bool {
    let s = normalize(s);
    if s.is_empty() || s.contains("RMB") { return false; }
    let alt = s.contains("ALT+");
    let shift = s.contains("SHIFT+");
    let ctrl = s.contains("CTRL+");
    let name = s.rsplit('+').next().unwrap_or("");
    let current = format!("{key:?}").to_ascii_uppercase();
    (current == name || (name == "SPACE" && key == Key::Space))
        && m.alt == alt && m.shift == shift && m.ctrl == ctrl && !m.mac_cmd
}
fn keyboard_text(key: Key, m: Modifiers) -> String {
    let mut v = String::new();
    if m.ctrl { v.push_str("Ctrl+"); }
    if m.alt { v.push_str("Alt+"); }
    if m.shift { v.push_str("Shift+"); }
    v.push_str(&format!("{key:?}"));
    v
}
fn mouse_text(m: Modifiers) -> String {
    let mut v = String::new();
    if m.ctrl { v.push_str("Ctrl+"); }
    if m.alt { v.push_str("Alt+"); }
    if m.shift { v.push_str("Shift+"); }
    v.push_str("RMB");
    v
}
fn valid_binding(s: &str, mouse: bool) -> bool {
    if s.trim().is_empty() { return true; }
    let s=normalize(s);
    let parts: Vec<_> = s.split('+').collect();
    if parts.is_empty() { return false; }
    let mut seen = std::collections::HashSet::new();
    for part in &parts[..parts.len()-1] {
        if !matches!(*part, "CTRL"|"ALT"|"SHIFT") || !seen.insert(*part) { return false; }
    }
    if mouse { parts.last() == Some(&"RMB") && parts.len()>1 }
    else {
        // There is no hardcoded list of allowed letters, Fn keys or special
        // keys. Accept any keyboard key exposed by this version of egui.
        // That includes Shift+D, Ctrl+Shift+D and combinations with arrows,
        // punctuation and function keys. Dedicated modifier keys alone are
        // still rejected.
        let name=*parts.last().unwrap_or(&"");
        !name.is_empty() && egui::Key::ALL.iter().any(|key| {
            // Modifier-key press/release events themselves cannot be the
            // terminal shortcut key; they made Ctrl+ControlLeft appear valid.
            let k = format!("{key:?}");
            !matches!(*key,
                Key::ShiftLeft | Key::ShiftRight |
                Key::ControlLeft | Key::ControlRight |
                Key::AltLeft | Key::AltRight |
                Key::SuperLeft | Key::SuperRight)
                && k.eq_ignore_ascii_case(name)
        })
    }
}
fn validate(configs: &[PieConfig]) -> Result<(), String> {
    if configs.is_empty() || configs.len()>24 { return Err("饼菜单数量必须在 1–24 之间".into()); }
    let mut bindings=std::collections::HashSet::new();
    for config in configs {
        if config.slots.len()!=8 { return Err(format!("「{}」必须恰好有 8 个方向",config.name)); }
        if !config.flick_threshold.is_finite() || !(8.0..=64.0).contains(&config.flick_threshold)
            || !config.menu_radius.is_finite() || !(70.0..=180.0).contains(&config.menu_radius) {
            return Err(format!("「{}」的手势距离或菜单尺寸无效",config.name));
        }
        if !valid_binding(&config.keyboard,false) || !valid_binding(&config.mouse,true) {
            return Err(format!("「{}」的快捷键格式无效",config.name));
        }
        for binding in [&config.keyboard, &config.mouse] {
            if !binding.is_empty() && !bindings.insert(normalize(binding)) {
                return Err(format!("重复快捷键：{binding}"));
            }
        }
    }
    Ok(())
}
fn run_slot(app: &mut VectorcraftApp, selected: usize, sector: usize) {
    let Some(slot) = app.ui.leo_pies.get(selected).and_then(|m| m.slots.get(sector)).cloned() else { return; };
    if slot.command.is_empty() { return; }
    if let Some(tool) = slot.command.strip_prefix("tool:") {
        crate::menus::invoke(app, "tool.select", json!({"tool":tool}));
    } else if let Some(panel) = slot.command.strip_prefix("panel:") {
        crate::menus::invoke(app, "window.panel", json!({"panel":panel}));
    } else {
        crate::menus::invoke(app, &slot.command, json!({}));
    }
}

/// Keyboard events are intercepted *before* VectorCraft's regular keymap.
/// Never trigger over a focused editor, dialog, IME or command palette.
pub fn keyboard_frame(app: &mut VectorcraftApp, ctx: &egui::Context) -> bool {
    // An open Pie Menu editor is a NON-MODAL configuration panel: the user
    // must be able to try gestures on the document without closing it.
    // The focus guard below still prevents shortcut activation while the
    // user types into any of its name, command, search or JSON fields.
    if app.ui.dialog.is_some() || app.ui.palette_open || ctx.egui_wants_keyboard_input()
        || app.session.tool_wants_text() || app.session.tool_composing() {
        return false;
    }
    if let Some(open) = app.ui.leo_pie_open.clone() {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            app.ui.leo_pie_open = None;
            return true;
        }
        // Optional Blender-style keyboard chord: hold the key, move the
        // cursor toward a sector, release the same key to execute.
        if let Some(key) = open.key {
            let release_enabled = app.ui.leo_pies.get(open.menu)
                .is_some_and(|m| m.keyboard_release_select);
            if release_enabled && ctx.input(|i| i.key_released(key)) {
                let at = ctx.input(|i| i.pointer.hover_pos());
                let threshold = app.ui.leo_pies[open.menu].flick_threshold;
                let slot = at.and_then(|p| open.sector(p,threshold)).or(open.last_sector);
                app.ui.leo_pie_open = None;
                if let Some(slot) = slot { run_slot(app,open.menu,slot); }
                return true;
            }
        }
        // While the menu is open, don't let tool-specific keyboard bindings
        // alter the active document under the menu.
        return true;
    }
    let found = ctx.input(|i| i.events.iter().find_map(|e| {
        if let egui::Event::Key {key,pressed:true,modifiers,..}=e {
            app.ui.leo_pies.iter().enumerate().find(|(_,p)|{
                keyboard_binding(&p.keyboard,*key,*modifiers) && (
                    p.allow_keyboard_override ||
                    (normalize(&p.keyboard) != "SHIFT+SPACE"
                        && crate::shortcut_editor::conflicts_with(&app.ui.shortcut_overrides,&p.keyboard,"ui.leoPieSettings").is_empty()))
            }).map(|(idx,_)|(idx,*key,*modifiers))
        } else { None }
    }));
    if let Some((menu,key,mods)) = found {
        let origin = ctx.input(|i| i.pointer.hover_pos()).or_else(|| app.canvas_rect.map(|r|r.center())).unwrap_or(Pos2::new(400.0,300.0));
        app.ui.leo_pie_open=Some(OpenPie{menu,origin,mouse:false,key:Some(key),last_sector:None});
        ctx.input_mut(|i| { i.consume_key(mods,key); });
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
    if app.ui.dialog.is_some() || app.ui.screen_mode>=3
        || app.session.tool_wants_text() || app.session.tool_composing() {
        return false;
    }
    let input = ui.input(|i| (i.pointer.clone(), i.modifiers, i.key_pressed(Key::Escape)));
    let (pointer, mods, escape) = input;
    if let Some(mut active)=app.ui.leo_pie_open.clone() {
        if escape {
            app.ui.leo_pie_open=None;
            return true;
        }
        let threshold=app.ui.leo_pies.get(active.menu).map_or(24.0,|m|m.flick_threshold);
        // hover_pos remains available even after button release; interact_pos
        // may vanish on the release frame, which formerly lost fast flicks.
        let at=pointer.hover_pos().or_else(||pointer.interact_pos());
        if let Some(p)=at { active.last_sector=active.sector(p,threshold); }
        if active.mouse && pointer.button_released(PointerButton::Secondary) {
            let slot=active.last_sector;
            app.ui.leo_pie_open=None;
            if let Some(slot)=slot {run_slot(app,active.menu,slot);}
            return true;
        }
        if !active.mouse && pointer.button_pressed(PointerButton::Primary) {
            let slot=active.last_sector;
            app.ui.leo_pie_open=None;
            if let Some(slot)=slot {run_slot(app,active.menu,slot);}
            return true;
        }
        if active.mouse && !pointer.button_down(PointerButton::Secondary) {
            app.ui.leo_pie_open=None; // lost focus: never leave a stuck menu
            return true;
        }
        app.ui.leo_pie_open=Some(active);
        return true;
    }
    if !resp.hovered() || !pointer.button_pressed(PointerButton::Secondary) {
        return false;
    }
    if let Some((menu,_))=app.ui.leo_pies.iter().enumerate().find(|(_,p)|mouse_binding(&p.mouse,mods)) {
        if let Some(pos)=pointer.interact_pos() {
            app.ui.leo_pie_open=Some(OpenPie{menu,origin:pos,mouse:true,key:None,last_sector:None});
            return true;
        }
    }
    false
}

/// Draw above the canvas using egui primitives, without replacing the native
/// application context menu on a normal (unmodified) RMB click.
pub fn paint(ui: &Ui, app: &VectorcraftApp) {
    let Some(open)=&app.ui.leo_pie_open else {return;};
    let Some(cfg)=app.ui.leo_pies.get(open.menu) else {return;};
    let pointer=ui.input(|i|i.pointer.hover_pos());
    let picked=pointer.and_then(|p|open.sector(p,cfg.flick_threshold));
    let radius=cfg.menu_radius.clamp(70.0,180.0);
    let painter=ui.painter();
    // Preserve the actual gesture origin; only the VISUAL layout is clamped
    // away from viewport edges so all eight options stay visible.
    let clip=ui.clip_rect();
    let margin=radius+46.0;
    let center=Pos2::new(
        if clip.width()>2.0*margin {open.origin.x.clamp(clip.min.x+margin,clip.max.x-margin)} else {clip.center().x},
        if clip.height()>2.0*margin {open.origin.y.clamp(clip.min.y+margin,clip.max.y-margin)} else {clip.center().y},
    );
    // Keep the visual weight low. Distinct command labels and a fast
    // directional highlight matter more than a giant opaque 8-slice disc.
    painter.circle_filled(center,31.0,Color32::from_rgba_unmultiplied(24,30,40,226));
    painter.circle_stroke(center,31.0,Stroke::new(1.4,Color32::from_rgb(112,138,170)));
    let active_color=Color32::from_rgb(115,182,250);
    for sector in 0..8 {
        let a=sector as f32*std::f32::consts::FRAC_PI_4;
        let direction=Vec2::angled(a);
        let at=center+direction*radius;
        let is_active=picked==Some(sector);
        let name=cfg.slots.get(sector).map(|x|x.name.as_str()).unwrap_or("");
        let is_empty=name.trim().is_empty();
        let label=if is_empty{"＋ 添加"}else{name};
        let text_size=if radius<88.0{11.0}else{13.0};
        // Compact, elongated pills mimic Blender's readable option layout.
        let text_width=(label.chars().count() as f32*text_size*0.77+22.0).clamp(66.0,148.0);
        let box_rect=Rect::from_center_size(at,Vec2::new(text_width,29.0));
        let fill=if is_active {Color32::from_rgb(65,106,157)}
            else {Color32::from_rgba_unmultiplied(38,46,58,235)};
        painter.rect_filled(box_rect,egui::CornerRadius::same(7),fill);
        painter.rect_stroke(box_rect,egui::CornerRadius::same(7),
            Stroke::new(if is_active{1.8}else{0.7},
                if is_active{active_color}else{Color32::from_rgb(69,82,100)}),
                egui::StrokeKind::Inside);
        if is_active {
            painter.line_segment([center+direction*34.0,at-direction*(text_width*0.5+4.0)],
                Stroke::new(2.0,active_color));
        }
        painter.text(at,egui::Align2::CENTER_CENTER,label,
            egui::FontId::proportional(text_size),
            if is_empty{Color32::from_rgb(150,162,178)}else{Color32::WHITE});
    }
    painter.text(center,egui::Align2::CENTER_CENTER,"Leo",
        egui::FontId::proportional(12.5),Color32::WHITE);
    // No unconditional request_repaint(). Native pointer input already
    // requests another frame; forcing redraw every frame wastes CPU/GPU.
}

fn builtin_commands()->Vec<(String,String)>{
    let mut v:Vec<_>=crate::palette::items().into_iter().map(|(label,id,_)|(label,id)).collect();
    for (id,label) in crate::state::all_panels() {
        v.push((format!("面板 › {label}"),format!("panel:{id}")));
    }
    v.sort_by(|a,b|a.0.cmp(&b.0));
    v.dedup_by(|a,b|a.1==b.1);
    v
}
pub fn editor(app: &mut VectorcraftApp, ctx: &egui::Context) {
    if !app.ui.leo_pie_editor{return;}
    let mut menus=app.ui.leo_pies.clone();
    let mut index=app.ui.leo_pie_selected.min(menus.len().saturating_sub(1));
    let mut sector=app.ui.leo_pie_slot.min(7);
    let mut query=app.ui.leo_pie_query.clone();
    let mut recording=app.ui.leo_pie_recording.clone();
    let mut import=app.ui.leo_pie_import.clone();
    let mut status=app.ui.leo_pie_message.clone();
    let mut open=true;
    let mut changed=false;
    let mut test_slot=None;
    let commands=builtin_commands();
    egui::Window::new("Leo Design · 饼菜单编辑器")
        .id(egui::Id::new("leo-pie-config-window"))
        .open(&mut open).default_pos([60.0,70.0]).default_size([710.0,540.0])
        .resizable(true).show(ctx,|ui|{
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
                menus.remove(index);index=0;changed=true;
            }
        });
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
                    if !slot.command.is_empty()
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
        ui.label("八方向槽位：点击一个方向，再从右侧原生命令目录添加功能");
        ui.columns(2,|cols|{
            cols[0].vertical(|ui|{
                for (i,label) in ["→ 右","↘ 右下","↓ 下","↙ 左下","← 左","↖ 左上","↑ 上","↗ 右上"].iter().enumerate() {
                    let title=format!("{}  {}",label,if menu.slots[i].name.is_empty(){"(空)"}else{&menu.slots[i].name});
                    ui.selectable_value(&mut sector,i,title);
                }
                ui.separator();
                ui.label(format!("当前槽位：{}",sector+1));
                ui.label("显示名称");
                changed|=ui.text_edit_singleline(&mut menu.slots[sector].name).changed();
                ui.label("命令 ID（高级）");
                changed|=ui.text_edit_singleline(&mut menu.slots[sector].command).changed();
                if ui.button("清空此方向").clicked(){menu.slots[sector]=PieSlot::default();changed=true;}
            });
            cols[1].vertical(|ui|{
                ui.label("搜索并添加 VectorCraft 功能");
                ui.text_edit_singleline(&mut query);
                let needle=query.trim().to_lowercase();
                egui::ScrollArea::vertical().max_height(210.0).show(ui,|ui|{
                    for (label,id) in commands.iter().filter(|(name,id)|{
                        needle.is_empty() || name.to_lowercase().contains(&needle) || id.to_lowercase().contains(&needle)
                    }).take(60) {
                        if ui.button(format!("＋ {label}")).on_hover_text(id).clicked(){
                            menu.slots[sector]=PieSlot{name:label.rsplit(" › ").next().unwrap_or(label).to_owned(),command:id.clone()};
                            changed=true;
                        }
                    }
                });
            });
        });
        ui.separator();
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
        if let Err(e)=validate(&menus){ui.colored_label(Color32::from_rgb(255,130,100),format!("配置冲突：{e}"));}
        if !status.is_empty(){ui.label(status.as_str());}
    });
    if changed {
        // Keep in-progress text edits (including temporarily invalid strings),
        // but validate before interpreting any gesture or importing JSON.
        app.ui.leo_pies=menus;
        app.prefs_changed=true;
    }
    app.ui.leo_pie_selected=index;
    app.ui.leo_pie_slot=sector;
    app.ui.leo_pie_query=query;
    app.ui.leo_pie_recording=recording;
    app.ui.leo_pie_import=import;
    app.ui.leo_pie_message=status;
    app.ui.leo_pie_editor=open;
    // A command test is performed only after the live edits above have been
    // committed to UiState; the editor stays open during the action.
    if let Some(slot)=test_slot {
        run_slot(app,index,slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn shift_d_and_common_shortcuts_are_accepted() {
        assert!(valid_binding("Shift+D",false));
        assert!(valid_binding("Shift + D",false));
        assert!(valid_binding("Ctrl+Shift+D",false));
        assert!(valid_binding("Alt+Shift+ArrowUp",false));
        assert!(valid_binding("Ctrl+Alt+F12",false));
        assert!(keyboard_binding("Shift+D", Key::D, Modifiers {shift:true,..Default::default()}));
        assert!(!keyboard_binding("Shift+D", Key::D, Modifiers::NONE));
        assert!(!valid_binding("Shift",false));
        assert!(!valid_binding("Ctrl+ControlLeft",false));
        assert!(!valid_binding("Shift+ShiftLeft",false));
    }
    #[test] fn keyboard_binding_is_exact() {
        assert!(keyboard_binding("Shift+Space",Key::Space,Modifiers{shift:true,..Default::default()}));
        assert!(!keyboard_binding("Shift+Space",Key::Space,Modifiers{shift:true,ctrl:true,..Default::default()}));
    }
    #[test] fn live_editor_does_not_modify_mouse_chord_rules() {
        // Canvas hit testing (resp.hovered) keeps the editor's text inputs
        // separate from modifier+RMB gestures.
        assert!(mouse_binding("Ctrl+Shift+RMB",Modifiers {
            ctrl:true,shift:true,..Default::default()
        }));
        assert!(!mouse_binding("Ctrl+Shift+RMB",Modifiers {
            ctrl:true,..Default::default()
        }));
    }
    #[test] fn mouse_chords_do_not_swallow_normal_right_click() {
        assert!(mouse_binding("Shift+Alt+RMB",Modifiers{shift:true,alt:true,..Default::default()}));
        assert!(mouse_binding("Shift+Ctrl+RMB",Modifiers{shift:true,ctrl:true,..Default::default()}));
        assert!(!mouse_binding("Shift+Alt+RMB",Modifiers::default()));
        assert!(!valid_binding("RMB",true));
    }
    #[test] fn duplicates_are_rejected() {
        let mut pies=defaults();
        pies[1].mouse=pies[0].mouse.clone();
        assert!(validate(&pies).is_err());
    }
    #[test] fn flick_beyond_old_240pt_limit_still_selects() {
        let o=OpenPie{menu:0,origin:Pos2::new(0.0,0.0),mouse:true,key:None,last_sector:None};
        assert_eq!(o.sector(Pos2::new(500.0,0.0),24.0),Some(0));
        assert_eq!(o.sector(Pos2::new(12.0,0.0),24.0),None);
        assert_eq!(o.sector(Pos2::new(0.0,400.0),24.0),Some(2));
        assert_eq!(o.sector(Pos2::new(-400.0,0.0),24.0),Some(4));
    }
    #[test] fn compact_pie_config_is_backward_compatible() {
        let p:PieConfig=serde_json::from_str(r#"{"name":"旧菜单","keyboard":"","mouse":"Shift+Alt+RMB","slots":[]}"#).unwrap();
        assert_eq!(p.flick_threshold,24.0);
        assert_eq!(p.menu_radius,106.0);
    }
    #[test] fn json_roundtrip_retains_all_eight_slots() {
        let p=defaults();let s=serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Vec<PieConfig>>(&s).unwrap(),p);
    }
}

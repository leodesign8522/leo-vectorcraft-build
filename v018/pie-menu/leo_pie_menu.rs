//! Leo Design Pie Menu v0.1.0 — native VectorCraft canvas radial commands.
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
    pub slots: Vec<PieSlot>,
}
impl Default for PieConfig {
    fn default() -> Self {
        Self {
            name: "我的常用设计".into(),
            keyboard: String::new(),
            mouse: "Shift+Alt+RMB".into(),
            allow_keyboard_override: false,
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
}
impl OpenPie {
    pub fn sector(&self, at: Pos2) -> Option<usize> {
        let d = at - self.origin;
        if d.length() < 38.0 || d.length() > 240.0 { return None; }
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
        let name=*parts.last().unwrap_or(&"");
        name == "SPACE" || name.len() == 1 && name.as_bytes()[0].is_ascii_alphanumeric()
            || matches!(name, "ENTER"|"TAB"|"BACKSPACE"|"DELETE"|"F1"|"F2"|"F3"|"F4"|"F5"|"F6"|"F7"|"F8"|"F9"|"F10"|"F11"|"F12")
    }
}
fn validate(configs: &[PieConfig]) -> Result<(), String> {
    if configs.is_empty() || configs.len()>24 { return Err("饼菜单数量必须在 1–24 之间".into()); }
    let mut bindings=std::collections::HashSet::new();
    for config in configs {
        if config.slots.len()!=8 { return Err(format!("「{}」必须恰好有 8 个方向",config.name)); }
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
    if app.ui.dialog.is_some() || app.ui.palette_open || app.ui.leo_pie_editor
        || ctx.egui_wants_keyboard_input() || app.session.tool_wants_text() || app.session.tool_composing() {
        return false;
    }
    if app.ui.leo_pie_open.is_some() {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            app.ui.leo_pie_open = None;
            return true;
        }
        return false;
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
        app.ui.leo_pie_open=Some(OpenPie{menu,origin,mouse:false});
        ctx.input_mut(|i| { i.consume_key(mods,key); });
        return true;
    }
    false
}

/// Returns true when the radial overlay owns the canvas and must suppress
/// the ordinary right-click popup and document tool pointer handling.
pub fn mouse_frame(app: &mut VectorcraftApp, ui: &Ui, resp: &egui::Response) -> bool {
    if app.ui.leo_pie_editor || app.ui.dialog.is_some() || app.ui.screen_mode>=3
        || app.session.tool_wants_text() || app.session.tool_composing() {
        return false;
    }
    let input = ui.input(|i| (i.pointer.clone(), i.modifiers, i.key_pressed(Key::Escape)));
    let (pointer, mods, escape) = input;
    if let Some(active)=app.ui.leo_pie_open.clone() {
        if escape {
            app.ui.leo_pie_open=None;
            return true;
        }
        if active.mouse && pointer.button_released(PointerButton::Secondary) {
            let slot=pointer.interact_pos().and_then(|p|active.sector(p));
            app.ui.leo_pie_open=None;
            if let Some(slot)=slot {run_slot(app,active.menu,slot);}
            return true;
        }
        if !active.mouse && pointer.button_pressed(PointerButton::Primary) {
            let slot=pointer.interact_pos().and_then(|p|active.sector(p));
            app.ui.leo_pie_open=None;
            if let Some(slot)=slot {run_slot(app,active.menu,slot);}
            return true;
        }
        return true;
    }
    if !resp.hovered() || !pointer.button_pressed(PointerButton::Secondary) {
        return false;
    }
    if let Some((menu,_))=app.ui.leo_pies.iter().enumerate().find(|(_,p)|mouse_binding(&p.mouse,mods)) {
        if let Some(pos)=pointer.interact_pos() {
            app.ui.leo_pie_open=Some(OpenPie{menu,origin:pos,mouse:true});
            return true;
        }
    }
    false
}

/// Draw above the canvas using egui primitives, without replacing the native
/// application context menu on a normal (unmodified) RMB click.
pub fn paint(ui: &Ui, app: &VectorcraftApp) {
    let Some(open)=&app.ui.leo_pie_open else {return;};
    let Some(cfg)=app.ui.leo_pies.get(open.menu) else{return;};
    let cursor=ui.input(|i|i.pointer.hover_pos());
    let picked=cursor.and_then(|p|open.sector(p));
    let p=ui.painter();
    let c=open.origin;
    let outer=126.0;
    for sector in 0..8 {
        let base=sector as f32 * std::f32::consts::FRAC_PI_4;
        let a=base-std::f32::consts::FRAC_PI_8;
        let b=base+std::f32::consts::FRAC_PI_8;
        let mut verts=vec![c];
        for j in 0..=8 {
            let angle=a+(b-a)*(j as f32/8.0);
            verts.push(c + Vec2::angled(angle)*outer);
        }
        let selected=picked==Some(sector);
        let fill=if selected {Color32::from_rgb(56,103,158)}
            else if sector%2==0 {Color32::from_rgb(43,51,66)}
            else {Color32::from_rgb(36,44,58)};
        p.add(egui::Shape::convex_polygon(verts,fill,Stroke::new(1.0,Color32::from_gray(95))));
        let at=c+Vec2::angled(base)*83.0;
        let name=cfg.slots.get(sector).map(|x|x.name.as_str()).unwrap_or("");
        let name=if name.is_empty(){"＋"}else{name};
        p.text(at,egui::Align2::CENTER_CENTER,name,egui::FontId::proportional(13.0),Color32::WHITE);
    }
    p.circle_filled(c,41.0,Color32::from_rgb(20,27,38));
    p.circle_stroke(c,41.0,Stroke::new(2.0,Color32::from_rgb(115,151,194)));
    p.text(c,egui::Align2::CENTER_CENTER,&cfg.name,egui::FontId::proportional(11.0),Color32::WHITE);
    p.text(c+Vec2::new(0.0,18.0),egui::Align2::CENTER_CENTER,"Esc 取消",egui::FontId::proportional(10.0),Color32::LIGHT_GRAY);
    ui.ctx().request_repaint();
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
    let commands=builtin_commands();
    egui::Window::new("Leo Design · 饼菜单编辑器")
        .id(egui::Id::new("leo-pie-config-window"))
        .open(&mut open).default_size([740.0,630.0])
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
        ui.label("触发快捷键（可以输入，也可以录制）");
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
                if key==Key::Escape{recording.clear();}
                else { menu.keyboard=keyboard_text(key,mods);recording.clear();changed=true; }
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
            match serde_json::from_str::<Vec<PieConfig>>(&import)
                .map_err(|e|e.to_string()).and_then(|x|{validate(&x)?;Ok(x)}) {
                Ok(v)=>{menus=v;index=0;changed=true;status="导入成功".into();}
                Err(e)=>status=format!("导入失败：{e}"),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn keyboard_binding_is_exact() {
        assert!(keyboard_binding("Shift+Space",Key::Space,Modifiers{shift:true,..Default::default()}));
        assert!(!keyboard_binding("Shift+Space",Key::Space,Modifiers{shift:true,ctrl:true,..Default::default()}));
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
    #[test] fn json_roundtrip_retains_all_eight_slots() {
        let p=defaults();let s=serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Vec<PieConfig>>(&s).unwrap(),p);
    }
}

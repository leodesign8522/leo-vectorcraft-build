//! Persistent shortcuts belong to actions, never to their mutable list indexes.
use crate::{VectorcraftApp, state::Dialog};
use craft_keymap::{Chord, Trigger};
use egui::Key;
#[cfg(test)]
use egui::Modifiers;
use serde_json::{Value, json};

fn chord(text: &str) -> Option<Chord> {
    if text.len() > 96 { return None; }
    let c = crate::keymap::vc_chord(text)?;
    let sc = crate::shortcuts::parse(text)?;
    if crate::shortcuts::is_modifier(sc.logical_key)
        || matches!(sc.logical_key, Key::Escape | Key::Enter | Key::Tab)
        || (sc.modifiers.is_none() && matches!(sc.logical_key,
            Key::Space | Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight
            | Key::Delete | Key::Backspace | Key::Home | Key::End | Key::PageUp | Key::PageDown))
    { return None; }
    Some(c)
}

fn same(a: &Chord, b: &Chord) -> bool {
    let platform = crate::keymap::platform();
    Trigger::single(a.clone()).effective(platform) == Trigger::single(b.clone()).effective(platform)
}

fn pie_conflict(_app: &VectorcraftApp, _chord: &Chord) -> Option<String> { None }

/// Native bindings always win, including bindings changed after an action was configured.
fn available(app: &VectorcraftApp, text: &str) -> Result<Chord, String> {
    let c = chord(text).ok_or("此按键用于编辑或导航，请换一个组合键或功能键。")?;
    let km = crate::keymap::keymap_with(&app.ui.shortcut_overrides);
    if km.is_reserved(&c) { return Err("此快捷键由操作系统保留。".into()); }
    let conflicts = crate::shortcut_editor::conflicts_with(&app.ui.shortcut_overrides, text, "");
    if !conflicts.is_empty() {
        let names: Vec<String> = conflicts.iter().map(|id| crate::shortcut_editor::entry(id)
            .map_or_else(|| id.clone(), |e| e.label.clone())).collect();
        return Err(format!("与现有快捷键冲突：{}。请换一个按键，或先在键盘快捷键中解除原绑定。", names.join("、")));
    }
    if let Some(name) = pie_conflict(app, &c) { return Err(format!("与饼菜单「{name}」的快捷键冲突。")); }
    Ok(c)
}

pub fn validate(app: &VectorcraftApp, target: (usize, usize), text: &str) -> Result<String, String> {
    if app.ui.action_sets.get(target.0).and_then(|s| s.actions.get(target.1)).is_none() {
        return Err("动作已不存在，请重新选择。".into());
    }
    if text.trim().is_empty() { return Ok(String::new()); }
    let c = available(app, text)?;
    for (si, set) in app.ui.action_sets.iter().enumerate() {
        for (ai, a) in set.actions.iter().enumerate() {
            if (si, ai) != target && chord(&a.leo_shortcut).is_some_and(|other| same(&c, &other)) {
                return Err(format!("已被动作「{} / {}」使用。", set.name, a.name));
            }
        }
    }
    crate::shortcut_editor::normalize(text).ok_or_else(|| "无法识别此快捷键。".into())
}

pub fn assign(app: &mut VectorcraftApp, target: (usize, usize), text: &str) -> Result<(), String> {
    let text = validate(app, target, text)?;
    let a = app.ui.action_sets.get_mut(target.0).and_then(|s| s.actions.get_mut(target.1)).ok_or("动作已不存在。")?;
    a.leo_shortcut = text;
    Ok(())
}

pub const SET_SHORTCUT: &str = "ui.action.shortcut";

pub fn command(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let index = |key| p.get(key).and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok());
    let target = (index("set").ok_or("no such action set")?, index("action").ok_or("no such action")?);
    let text = p.get("shortcut").and_then(Value::as_str).ok_or("shortcut must be a string")?;
    assign(app, target, text)?;
    Ok(json!({"shortcut": app.ui.action_sets.get(target.0).and_then(|s| s.actions.get(target.1)).map(|a| &a.leo_shortcut)}))
}

fn target(d: &Dialog) -> Option<(usize, usize)> {
    let index = |key| d.fields.get(key).and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok());
    Some((index("set")?, index("action")?))
}

pub fn recording(app: &VectorcraftApp) -> bool {
    app.ui.dialog.as_ref().is_some_and(|d| d.kind == crate::dialogs::action_set_options::KIND
        && d.fields.get("__leo_recording").and_then(Value::as_bool) == Some(true))
}

/// Runs before dialog Escape/Enter handling. No captured key reaches artwork or a text field.
pub fn capture_frame(app: &mut VectorcraftApp, ctx: &egui::Context) -> bool {
    if !recording(app) { return false; }
    let event = ctx.input(|i| i.events.iter().find_map(|e| match e {
        egui::Event::Key { key, modifiers, pressed: true, repeat: false, .. }
            if !crate::shortcuts::is_modifier(*key) => Some((*key, *modifiers)),
        _ => None,
    }));
    ctx.input_mut(|i| i.events.retain(|e| !matches!(e,
        egui::Event::Key { .. } | egui::Event::Text(_) | egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_))));
    if let Some((key, modifiers)) = event {
        let destination = app.ui.dialog.as_ref().and_then(target);
        let result = if key == Key::Escape { None } else {
            let text = crate::shortcut_editor::format(&egui::KeyboardShortcut::new(modifiers, key));
            Some(destination.ok_or_else(|| "动作已不存在。".to_string()).and_then(|t| validate(app, t, &text)))
        };
        if let Some(d) = app.ui.dialog.as_mut() {
            d.fields.insert("__leo_recording".into(), json!(false));
            match result {
                Some(Ok(text)) => {
                    d.fields.insert("leo_shortcut".into(), json!(text));
                    d.fields.insert("__leo_message".into(), json!("点击确定保存快捷键。"));
                }
                Some(Err(message)) => { d.fields.insert("__leo_message".into(), json!(message)); }
                None => { d.fields.insert("__leo_message".into(), json!("已取消录入，原快捷键保留。")); }
            }
        }
    }
    true
}

pub fn options(app: &VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) {
    if target(d).is_none() { return; }
    ui.add_space(8.0);
    let is_recording = d.fields.get("__leo_recording").and_then(Value::as_bool) == Some(true);
    ui.horizontal(|ui| {
        ui.label("快捷键：");
        let text = d.str("leo_shortcut");
        let label = if is_recording { "请按快捷键…（Esc 取消）".to_string() }
            else if text.is_empty() { "点击录入快捷键".into() } else { crate::menus::pretty_shortcut(&text) };
        if ui.button(label).clicked() {
            d.fields.insert("__leo_recording".into(), json!(true));
            d.fields.insert("__leo_message".into(), json!(""));
        }
        if ui.button("清除").clicked() {
            d.fields.insert("leo_shortcut".into(), json!(""));
            d.fields.insert("__leo_recording".into(), json!(false));
            d.fields.insert("__leo_message".into(), json!("点击确定移除快捷键。"));
        }
    });
    let message = d.str("__leo_message");
    if !message.is_empty() { ui.label(message); }
    if let Some(t) = target(d) && let Err(message) = validate(app, t, &d.str("leo_shortcut")) { ui.label(message); }
    ui.label("快捷键随动作保存；编辑文字、录制动作和打开对话框时不触发。");
}

fn find(app: &VectorcraftApp, candidates: &[Chord]) -> Option<(usize, usize)> {
    let mut found = None;
    for (si, set) in app.ui.action_sets.iter().enumerate() {
        for (ai, action) in set.actions.iter().enumerate() {
            let Some(c) = chord(&action.leo_shortcut) else { continue };
            if !candidates.iter().any(|candidate| same(&c, candidate)) { continue; }
            if available(app, &action.leo_shortcut).is_err() { continue; }
            // Duplicate imports or edited preferences must never choose an arbitrary action.
            if found.is_some() { return None; }
            found = Some((si, ai));
        }
    }
    found
}

/// One command.batch per non-repeating key press, with native undo behaviour.
pub fn keyboard_frame(app: &mut VectorcraftApp, ctx: &egui::Context) -> bool {
    if app.ui.dialog.is_some() || app.ui.palette_open || app.ui.recording.is_some()
        || app.ui.screen_mode == 3 || ctx.egui_wants_keyboard_input() || egui::Popup::is_any_open(ctx)
        || app.session.tool_wants_text() || app.session.tool_composing() || app.session.tool_busy()
    { return false; }
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        if !matches!(event, egui::Event::Key { pressed: true, repeat: false, .. }) { continue; }
        let Some(input) = craft_keymap::KeyInput::from_egui(&event, crate::keymap::platform()) else { continue };
        let candidates = craft_keymap::Layout::default().candidates(&input);
        if let Some((si, ai)) = find(app, &candidates) {
            // Return after one action and discard its paired character event as well.
            ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))));
            if let Err(message) = crate::panels::actions::play(app, si, ai) { app.status(message); }
            return true;
        }
    }
    false
}

/// A file can carry shortcuts, but importing another copy cannot hijack existing bindings.
pub fn sanitize_import(app: &VectorcraftApp, set: &mut crate::panels::actions::ActionSet) {
    let mut accepted: Vec<Chord> = Vec::new();
    for action in &mut set.actions {
        if action.leo_shortcut.is_empty() { continue; }
        let valid = available(app, &action.leo_shortcut).ok().filter(|c| {
            !accepted.iter().any(|other| same(c, other)) && !app.ui.action_sets.iter().any(|s|
                s.actions.iter().any(|a| chord(&a.leo_shortcut).is_some_and(|other| same(c, &other))))
        });
        if let Some(c) = valid { accepted.push(c); } else { action.leo_shortcut.clear(); }
    }
}

#[cfg(test)]
mod leo_action_shortcut_tests {
    use super::*;
    use crate::panels::actions::{Action, ActionSet};
    fn app() -> VectorcraftApp {
        let mut a = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        a.ui.action_sets = vec![ActionSet::new("自定义", vec![
            Action { name: "一".into(), steps: vec![], leo_shortcut: String::new() },
            Action { name: "二".into(), steps: vec![], leo_shortcut: String::new() },
        ])];
        a
    }
    const FREE: &str = "Ctrl+Alt+Shift+F12";
    fn event(key: Key, modifiers: Modifiers, repeat: bool) -> egui::Event {
        egui::Event::Key { key, physical_key: Some(key), pressed: true, repeat, modifiers }
    }
    #[test]
    fn assignments_clear_conflicts_and_old_preferences_roundtrip() {
        let mut a = app();
        assign(&mut a, (0, 0), FREE).unwrap();
        assert!(assign(&mut a, (0, 1), FREE).is_err());
        assert!(assign(&mut a, (0, 0), "Ctrl+S").is_err());
        for bad in ["Escape", "Enter", "Tab", "Space", "Delete", "Shift+ShiftLeft", "garbage"] {
            assert!(assign(&mut a, (0, 0), bad).is_err(), "{bad}");
        }
        let raw = serde_json::to_value(&a.ui).unwrap();
        let restored: crate::state::UiState = serde_json::from_value(raw).unwrap();
        assert_eq!(restored.action_sets[0].actions[0].leo_shortcut, FREE);
        let old: Action = serde_json::from_value(json!({"name":"old", "steps":[]})).unwrap();
        assert!(old.leo_shortcut.is_empty());
        assign(&mut a, (0, 0), "").unwrap();
        assert!(a.ui.action_sets[0].actions[0].leo_shortcut.is_empty());
    }
    #[test]
    fn rename_reorder_and_native_reassignment_are_safe() {
        let mut a = app();
        assign(&mut a, (0, 0), FREE).unwrap();
        a.ui.action_sets[0].actions[0].name = "改名".into();
        a.ui.action_sets[0].actions.swap(0, 1);
        let c = chord(FREE).unwrap();
        assert_eq!(find(&a, &[c.clone()]), Some((0, 1)));
        crate::keymap::set_override(&mut a.ui.shortcut_overrides, "edit.undo", FREE);
        assert_eq!(find(&a, &[c]), None);
    }
    #[test]
    fn duplicate_and_import_cannot_steal_shortcuts() {
        let mut a = app();
        assign(&mut a, (0, 0), FREE).unwrap();
        let text = crate::panels::actions::write_set(&a.ui.action_sets[0]);
        let mut imported = crate::panels::actions::read_set(&text, "").unwrap();
        assert_eq!(imported.actions[0].leo_shortcut, FREE);
        sanitize_import(&a, &mut imported);
        assert!(imported.actions[0].leo_shortcut.is_empty());
        crate::panels::actions::duplicate(&mut a, &json!({"set":0,"action":0})).unwrap();
        assert_eq!(a.ui.action_sets[0].actions[0].leo_shortcut, FREE);
        assert!(a.ui.action_sets[0].actions[1].leo_shortcut.is_empty());
        a.ui.action_sets[0].actions[1].leo_shortcut = FREE.into();
        assert!(find(&a, &[chord(FREE).unwrap()]).is_none());
    }
    #[test]
    fn recording_esc_cancels_without_closing_and_enter_never_confirms() {
        let mut a = app();
        crate::dialogs::action_set_options::open_action(&mut a, 0, 0);
        a.ui.dialog.as_mut().unwrap().fields.insert("__leo_recording".into(), json!(true));
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::Escape, Modifiers::NONE, false)], ..Default::default() });
        assert!(capture_frame(&mut a, &ctx));
        assert!(!recording(&a));
        assert!(a.ui.dialog.is_some());
        assert!(ctx.input(|i| i.events.is_empty()));
        ctx.end_pass().textures_delta.clear();
    }
    #[test]
    fn dialog_capture_precedes_fields_and_confirmation_and_cancel_discards_changes() {
        let mut a = app();
        crate::dialogs::action_set_options::open_action(&mut a, 0, 0);
        a.ui.dialog.as_mut().unwrap().fields.insert("__leo_recording".into(), json!(true));
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::Enter, Modifiers::NONE, false)], ..Default::default() });
        crate::dialogs::show(&mut a, &ctx);
        ctx.end_pass().textures_delta.clear();
        assert!(a.ui.dialog.is_some(), "recorded Enter must not confirm the dialog");
        assert!(a.ui.action_sets[0].actions[0].leo_shortcut.is_empty());
        a.ui.dialog.as_mut().unwrap().fields.insert("__leo_recording".into(), json!(true));
        let mods = Modifiers { ctrl:true, command:true, alt:true, shift:true, ..Modifiers::NONE };
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::F12, mods, false)], ..Default::default() });
        crate::dialogs::show(&mut a, &ctx);
        ctx.end_pass().textures_delta.clear();
        assert!(!a.ui.dialog.as_ref().unwrap().str("leo_shortcut").is_empty());
        crate::dialogs::cancel(&mut a);
        assert!(a.ui.action_sets[0].actions[0].leo_shortcut.is_empty());
        a.run(SET_SHORTCUT, json!({"set":0,"action":0,"shortcut":FREE})).unwrap();
        crate::dialogs::action_set_options::open_action(&mut a, 0, 0);
        a.ui.dialog.as_mut().unwrap().fields.insert("name".into(), json!("改名后"));
        crate::dialogs::confirm(&mut a).unwrap();
        assert_eq!(a.ui.action_sets[0].actions[0].name, "改名后");
        assert_eq!(a.ui.action_sets[0].actions[0].leo_shortcut, FREE);
    }
    #[test]
    fn key_runs_exact_action_once_and_recording_or_dialog_blocks_it() {
        let mut a = app();
        a.run("file.new", json!({"width":100,"height":100})).unwrap();
        a.ui.action_sets[0].actions[0].steps = vec![("shape.rectangle".into(), json!({"x":0,"y":0,"width":20,"height":10}))];
        assign(&mut a, (0, 0), FREE).unwrap();
        let ctx = egui::Context::default();
        let mods = Modifiers { ctrl:true, command:true, alt:true, shift:true, ..Modifiers::NONE };
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::F12, mods, false)], ..Default::default() });
        assert!(keyboard_frame(&mut a, &ctx));
        assert_eq!(a.session.active().unwrap().selection.len(), 1);
        assert!(!keyboard_frame(&mut a, &ctx));
        a.run("edit.undo", json!({})).unwrap();
        assert!(a.session.active().unwrap().selection.is_empty());
        ctx.end_pass().textures_delta.clear();
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::F12, mods, true)], ..Default::default() });
        assert!(!keyboard_frame(&mut a, &ctx));
        ctx.end_pass().textures_delta.clear();
        a.ui.recording = Some((0, "record".into(), 0));
        ctx.begin_pass(egui::RawInput { events: vec![event(Key::F12, mods, false)], ..Default::default() });
        assert!(!keyboard_frame(&mut a, &ctx));
        a.ui.recording = None;
        crate::dialogs::action_set_options::open_action(&mut a, 0, 1);
        assert!(!keyboard_frame(&mut a, &ctx));
        ctx.end_pass().textures_delta.clear();
    }
}

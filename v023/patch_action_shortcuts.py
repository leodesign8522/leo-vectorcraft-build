#!/usr/bin/env python3
"""Add persistent per-action shortcuts after all Core/Pie integration patches."""
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
here = Path(__file__).resolve().parent
base = root / 'crates/ui-egui/src'

def change(rel, old, new, expected=1):
    p = base / rel
    s = p.read_text(encoding='utf-8')
    if s.count(old) != expected:
        raise SystemExit(f'Unexpected action shortcut anchor: {rel}: {s.count(old)}: {old!r}')
    p.write_text(s.replace(old, new), encoding='utf-8')

change('lib.rs', 'pub mod workspaces;\n', 'pub mod workspaces;\npub mod leo_action_shortcuts;\n')
change('panels/actions.rs', 'pub struct Action {\n', '''pub struct Action {
    /// Leo Design: saved with the action, including action-set export; absent in old files.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub leo_shortcut: String,
''')

# Native 0.21.1 routes all construction and duplication through Action methods.
change('panels/actions.rs', 'Self { id: next_action_id(), name: name.into(),',
       'Self { leo_shortcut: String::new(), id: next_action_id(), name: name.into(),')
change('panels/actions.rs', 'Self { id: next_action_id(), key: None, ..self.clone() }',
       'Self { leo_shortcut: String::new(), id: next_action_id(), key: None, ..self.clone() }')

change('panels/actions.rs', '                    crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps"),',
       '''                    if a.leo_shortcut.is_empty() {
                        crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps")
                    } else {
                        format!("{} �� {}", crate::menus::pretty_shortcut(&a.leo_shortcut),
                            crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps"))
                    },''')
change('panels/actions.rs', 'pub use files::{ACTIONS_EXT, LOAD_SET, SAVE_SET, load_set, save_set};',
       'pub use files::{ACTIONS_EXT, LOAD_SET, SAVE_SET, load_set, save_set, read_set, write_set};')
change('panels/actions/files.rs', 'struct FileAction {\n', '''struct FileAction {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    leo_shortcut: String,
''')
change('panels/actions/files.rs', '.map(|a| FileAction {\n', '.map(|a| FileAction {\n            leo_shortcut: a.leo_shortcut.clone(),\n')
change('panels/actions/files.rs', 'Action { key: a.key, color: a.color, excluded_steps, ..Action::new(a.name,',
       'Action { leo_shortcut: a.leo_shortcut, key: a.key, color: a.color, excluded_steps, ..Action::new(a.name,')
change('panels/actions/files.rs', '    set.name = free_set_name(&app.ui.action_sets, &set.name);',
       '    crate::leo_action_shortcuts::sanitize_import(app, &mut set);\n    set.name = free_set_name(&app.ui.action_sets, &set.name);', expected=2)

change('dialogs/action_set_options.rs',
       '"name": a.name, "key": key,',
       '"name": a.name, "leo_shortcut": a.leo_shortcut, "key": key,')
change('dialogs/action_set_options.rs', 'fn body(_app: &mut VectorcraftApp,', 'fn body(app: &mut VectorcraftApp,')
change('dialogs/action_set_options.rs', '    false\n}', '    crate::leo_action_shortcuts::options(app, ui, d);\n    false\n}')
change('dialogs/action_set_options.rs', '        app.run(crate::panels::actions::ACTION_OPTIONS, p)?;',
       '''        let index = |k| d.fields.get(k).and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok());
        let target = (index("set").ok_or("no such action set")?, index("action").ok_or("no such action")?);
        if crate::leo_action_shortcuts::recording(app) { return Err("������ɿ�ݼ�¼�롣".into()); }
        let custom = if d.fields.contains_key("leo_shortcut") {
            Some(crate::leo_action_shortcuts::validate(app, target, &d.str("leo_shortcut"))?)
        } else { None };
        if let Some(text) = custom.as_deref().filter(|text| !text.is_empty())
            && let Some(key) = crate::panels::actions::FunctionKey::from_params(&p)?
            && key.chord().zip(crate::keymap::vc_chord(text)).is_some_and(|(a,b)| {
                a.effective(crate::keymap::platform()) == b.effective(crate::keymap::platform())
            })
        { return Err("�Զ����ݼ��빦�ܼ��ظ����뱣������һ����".into()); }
        app.run(crate::panels::actions::ACTION_OPTIONS, p)?;
        if let Some(text) = custom {
            crate::leo_action_shortcuts::assign(app, target, &text)?;
        }''')

change('menus/run.rs', '        crate::panels::actions::RENAME_ACTION => crate::panels::actions::rename_action(app, p),',
       '''        crate::panels::actions::RENAME_ACTION => crate::panels::actions::rename_action(app, p),
        crate::leo_action_shortcuts::SET_SHORTCUT => crate::leo_action_shortcuts::command(app, p),''')
change('menus/ui_commands.rs', '        crate::panels::actions::RENAME_ACTION,',
       '''        crate::leo_action_shortcuts::SET_SHORTCUT,
        "Set Action Shortcut", "", "{set, action, shortcut} assigns or clears an action shortcut",
    ),
    (
        crate::panels::actions::RENAME_ACTION,''')

change('shortcuts.rs', '    if app.ui.dialog.is_some() || app.ui.palette_open {',
       '    if crate::leo_action_shortcuts::capture_frame(app, ctx) { return; }\n    if app.ui.dialog.is_some() || app.ui.palette_open {')
change('dialogs.rs', 'pub fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {',
       '''pub fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {
    // Capture before name fields and the common Enter-to-confirm handler see the press.
    crate::leo_action_shortcuts::capture_frame(app, ctx);''')
change('shortcuts.rs', '    // A plain Delete or Backspace after a click on a Layers or Appearance row',
       '''    if crate::leo_action_shortcuts::keyboard_frame(app, ctx) { return; }
    // A plain Delete or Backspace after a click on a Layers or Appearance row''')

source = (here / 'leo_action_shortcuts.rs').read_text(encoding='utf-8')
if (base / 'leo_pie_menu.rs').exists():
    source = source.replace('fn pie_conflict(_app: &VectorcraftApp, _chord: &Chord) -> Option<String> { None }', '''fn pie_conflict(app: &VectorcraftApp, c: &Chord) -> Option<String> {
    app.ui.leo_pies.iter().find(|p| {
        // Pie settings use Ctrl/Alt/Shift spelling, case insensitive.
        let text = p.keyboard.split('+').map(|part| match part.trim().to_ascii_uppercase().as_str() {
            "CTRL" => "Ctrl".to_string(), "ALT" => "Alt".to_string(), "SHIFT" => "Shift".to_string(),
            _ => egui::Key::ALL.iter().find(|key| key.name().eq_ignore_ascii_case(part.trim()))
                .map_or_else(|| part.trim().to_string(), |key| key.name().to_string()),
        }).collect::<Vec<_>>().join("+");
        crate::keymap::vc_chord(&text).is_some_and(|other| same(c, &other))
    }).map(|p| p.name.clone())
}''')
(base / 'leo_action_shortcuts.rs').write_text(source, encoding='utf-8')
print('PASS persistent action shortcuts, UI recorder, native conflict protection and file compatibility')

if (base / 'leo_pie_menu.rs').exists():
    with (base / 'leo_action_shortcuts.rs').open('a', encoding='utf-8') as f:
        f.write('''
#[cfg(test)]
mod leo_action_pie_conflict_tests {
    #[test]
    fn leo_action_shortcut_tests_recognize_uppercase_pie_special_key_names() {
        let mut app = crate::VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.ui.leo_pies[0].keyboard = "SHIFT+SPACE".into();
        assert!(super::available(&app, "Shift+Space").unwrap_err().contains("���˵�"));
    }
}
''')

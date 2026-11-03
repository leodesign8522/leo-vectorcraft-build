#!/usr/bin/env python3
"""Add persistent per-action shortcuts after all Core/Pie integration patches."""
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
here = Path(__file__).resolve().parent
base = root / 'crates/ui-egui/src'

def change(rel, old, new):
    p = base / rel
    s = p.read_text(encoding='utf-8')
    if s.count(old) != 1:
        raise SystemExit(f'Unexpected action shortcut anchor: {rel}: {s.count(old)}')
    p.write_text(s.replace(old, new, 1), encoding='utf-8')

change('lib.rs', 'pub mod workspaces;\n', 'pub mod workspaces;\npub mod leo_action_shortcuts;\n')
change('panels/actions.rs', 'pub struct Action {\n', '''pub struct Action {
    /// Leo Design: saved with the action, including action-set export; absent in old files.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub leo_shortcut: String,
''')

# Only the two native Actions files construct this type. Match its first field,
# never the struct declaration, enum patterns, or a string such as "Action {n}".
for rel in ['panels/actions.rs', 'panels/actions/files.rs']:
    p = base / rel
    s = p.read_text(encoding='utf-8')
    s, n = re.subn(r'\bAction \{(?=\s*name(?:\s*:|\s*,))', 'Action { leo_shortcut: String::new(),', s)
    if n == 0: raise SystemExit(f'No action constructors: {rel}')
    p.write_text(s, encoding='utf-8')

change('panels/pathfinder.rs', 'crate::panels::actions::Action { name:',
       'crate::panels::actions::Action { leo_shortcut: String::new(), name:')

change('panels/actions.rs', '        copy.name = format!("{} copy", copy.name);',
       '        copy.leo_shortcut.clear();\n        copy.name = format!("{} copy", copy.name);')
change('panels/actions.rs', '        let copy = ActionSet::new(format!("{} copy", src.name), src.actions.clone());',
       '''        let mut copy = ActionSet::new(format!("{} copy", src.name), src.actions.clone());
        for action in &mut copy.actions { action.leo_shortcut.clear(); }''')
change('panels/actions.rs', '                    crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps"),',
       '''                    if a.leo_shortcut.is_empty() {
                        crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps")
                    } else {
                        format!("{} · {}", crate::menus::pretty_shortcut(&a.leo_shortcut),
                            crate::i18n::tn(a.steps.len() as u64, "{n} step", "{n} steps"))
                    },''')
change('panels/actions.rs', 'pub use files::{ACTIONS_EXT, LOAD_SET, SAVE_SET, load_set, save_set};',
       'pub use files::{ACTIONS_EXT, LOAD_SET, SAVE_SET, load_set, save_set, read_set, write_set};')
change('panels/actions/files.rs', 'struct FileAction {\n', '''struct FileAction {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    leo_shortcut: String,
''')
change('panels/actions/files.rs', '.map(|a| FileAction {\n', '.map(|a| FileAction {\n            leo_shortcut: a.leo_shortcut.clone(),\n')
change('panels/actions/files.rs', 'Action { leo_shortcut: String::new(), name: a.name,',
       'Action { leo_shortcut: a.leo_shortcut, name: a.name,')
change('panels/actions/files.rs', '    set.name = free_set_name(&app.ui.action_sets, &set.name);',
       '    crate::leo_action_shortcuts::sanitize_import(app, &mut set);\n    set.name = free_set_name(&app.ui.action_sets, &set.name);')

change('dialogs/action_set_options.rs', 'json!({"set": set, "action": action, "name": a.name})',
       'json!({"set": set, "action": action, "name": a.name, "leo_shortcut": a.leo_shortcut})')
change('dialogs/action_set_options.rs', 'fn body(_: &mut VectorcraftApp,', 'fn body(app: &mut VectorcraftApp,')
change('dialogs/action_set_options.rs', '    false\n}', '    crate::leo_action_shortcuts::options(app, ui, d);\n    false\n}')
change('dialogs/action_set_options.rs', '    if d.fields.contains_key("action") {\n        app.run',
       '''    if d.fields.contains_key("action") {
        let index = |k| d.fields.get(k).and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok());
        let target = (index("set").ok_or("no such action set")?, index("action").ok_or("no such action")?);
        if crate::leo_action_shortcuts::recording(app) { return Err("请先完成快捷键录入。".into()); }
        app.run(crate::leo_action_shortcuts::SET_SHORTCUT, json!({"set":target.0, "action":target.1, "shortcut":d.str("leo_shortcut")}))?;
        app.run''')

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
            _ => part.trim().to_string(),
        }).collect::<Vec<_>>().join("+");
        crate::keymap::vc_chord(&text).is_some_and(|other| same(c, &other))
    }).map(|p| p.name.clone())
}''')
(base / 'leo_action_shortcuts.rs').write_text(source, encoding='utf-8')
print('PASS persistent action shortcuts, UI recorder, native conflict protection and file compatibility')

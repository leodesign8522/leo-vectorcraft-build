"""Route the Direct Selection tool's idle H/L/S claims before global shortcuts."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
p=root/'crates/ui-egui/src/shortcuts.rs'
s=p.read_text(encoding='utf-8')
anchor='    let rows_take = |k: Key| rows_keys && m.is_none() && matches!(k, Key::Delete | Key::Backspace);'
if s.count(anchor)!=1:raise SystemExit('Unexpected idle shortcut routing anchor')
insert='''    // Path Tools claims these letters while idle too. The upstream letter
    // dispatch in busy_tool_keys runs only during a drag/modal gesture.
    if app.session.tool_id() == "directSelection" && !(m.command || m.ctrl || m.alt) {
        for (key, letter) in [(Key::H, 'h'), (Key::L, 'l'), (Key::S, 's')] {
            if ctx.input(|i| i.key_pressed(key))
                && app.session.tool_claims_key(ToolKey::Letter(letter), view)
                && ctx.input_mut(|i| i.consume_key(m, key)) {
                ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Text(t) if t.eq_ignore_ascii_case(&letter.to_string()))));
                let result = app.session.tool_key(ToolKey::Letter(letter), crate::canvas::mods(m, false), view);
                crate::canvas::apply_requests(app, result);
                return true;
            }
        }
    }
'''
s=s.replace(anchor,insert+anchor,1)
p.write_text(s,encoding='utf-8',newline='\n')
with (root/'crates/ui-egui/src/shortcuts/tests.rs').open('a',encoding='utf-8') as f:
    f.write((Path(__file__).parent/'ui_keys_tests.rs').read_text(encoding='utf-8'))

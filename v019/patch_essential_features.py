#!/usr/bin/env python3
"""Leo Design v0.19 enhancement modules: CJK clipboard, paragraph, Ctrl+Enter.
Only apply after base Core Bridge; all anchors are checked. These port the
historical v0.17 behavior to v0.19 source before rebuilding the main program.
"""
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_essential_features.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])
project = Path(__file__).resolve().parents[1]

for script in [
    "patch_illustrator_text_bridge.py",
    "patch_pdf_cjk_font_names.py",
    "patch_chinese_paragraph.py",
]:
    print(f"Leo Design v0.19 port: {script}", flush=True)
    subprocess.run([sys.executable, str(project / "scripts" / script), str(root)], check=True)

path = root / "crates/ui-egui/src/shortcuts.rs"
src = path.read_text(encoding="utf-8")
old = """    let busy = app.session.tool_busy();
    for (k, tk) in [(Key::Enter, ToolKey::Enter), (Key::Escape, ToolKey::Escape)] {"""
new = """    let busy = app.session.tool_busy();

    // Leo Design (v0.19): Ctrl+Enter commits the live Type editing session.
    // Never fire while an IME composes, a dialog or input has keyboard focus,
    // or a user-defined command owns the Ctrl+Enter shortcut.
    if !typing && !composing && app.session.tool_wants_text()
        && ctx.input(|i| {
            i.key_pressed(Key::Enter)
                && i.modifiers.command
                && !i.modifiers.shift
                && !i.modifiers.alt
                && !runs_a_command(i, Key::Enter, true)
        })
    {
        let actions = app.session.tool_key(ToolKey::Escape, Mods::default(), view);
        crate::canvas::apply_requests(app, actions);
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::COMMAND, Key::Enter);
            i.events.retain(|e| !matches!(e, egui::Event::Text(t) if t == "\\n" || t == "\\r"));
        });
        return;
    }

    for (k, tk) in [(Key::Enter, ToolKey::Enter), (Key::Escape, ToolKey::Escape)] {"""
count = src.count(old)
if count != 1:
    raise SystemExit(f"FAIL v0.19 Ctrl+Enter patch anchor={count}")
path.write_text(src.replace(old, new, 1), encoding="utf-8", newline="\n")
print("SUCCESS: Leo Design v0.19 type editing, CJK paragraph and font compatibility")

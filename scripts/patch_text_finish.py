#!/usr/bin/env python3
"""Leo Design Ctrl+Enter: finish text editing, like Adobe Illustrator.

Scope: VectorCraft v0.17.0. Only active canvas text editing (not numeric fields,
dialogs, IME composition, other tools). Honours explicit user command bindings.
Escape already commits text changes and returns to the previous/Selection tool.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python scripts/patch_text_finish.py VECTORCRAFT_ROOT")
path = Path(sys.argv[1]) / "crates/ui-egui/src/shortcuts.rs"
source = path.read_text(encoding="utf-8")

def replace_once(original: str, replacement: str, label: str):
    global source
    count = source.count(original)
    if count != 1:
        raise SystemExit(f"FAIL: {label}: expected 1 anchor, found {count}")
    source = source.replace(original, replacement, 1)

replace_once(
    """    let busy = app.session.tool_busy();
    for (k, tk) in [(Key::Enter, ToolKey::Enter), (Key::Escape, ToolKey::Escape)] {""",
    """    let busy = app.session.tool_busy();

    // Leo Design: Illustrator-style Ctrl+Enter finishes on-canvas text editing.
    // Do not affect ordinary Enter (newline), IME confirmation, text fields, dialogs,
    // other tools, AltGr, or a custom command explicitly assigned to Ctrl+Enter.
    if !typing
        && !composing
        && app.session.tool_wants_text()
        && ctx.input(|i| {
            i.key_pressed(Key::Enter)
                && i.modifiers.command
                && !i.modifiers.shift
                && !i.modifiers.alt
                && !runs_a_command(i, Key::Enter, true)
        })
    {
        // The Type tool's Escape performs the existing safe commit/finish workflow
        // and switches to the previous tool or the Selection (V) tool.
        let actions = app.session.tool_key(ToolKey::Escape, Mods::default(), view);
        crate::canvas::apply_requests(app, actions);
        ctx.input_mut(|i| {
            // Consume this shortcut so it cannot become a line break or another command.
            i.consume_key(Modifiers::COMMAND, Key::Enter);
            i.events.retain(|e| !matches!(e, egui::Event::Text(t) if t == "\\n" || t == "\\r"));
        });
        return;
    }

    for (k, tk) in [(Key::Enter, ToolKey::Enter), (Key::Escape, ToolKey::Escape)] {""",
    "Intercept Ctrl+Enter only in canvas text editing",
)

replace_once(
    """    /// A command bound to Cmd+Enter runs without the active tool's Enter: the Selection tool opens""",
    """    /// Leo Design: Ctrl+Enter exits text editing while keeping the text unchanged.
    #[test]
    fn leo_ctrl_enter_finishes_text_editing_without_newline() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.session.execute("file.new", &json!({"width": 400, "height": 300})).unwrap();
        let id = app.session.execute("text.create", &json!({
            "x": 20, "y": 250, "text": "航展文字"
        })).unwrap()["id"].as_u64().unwrap();
        app.select_tool("type");
        app.session.apply_actions(vec![vectorcraft_tools::Action::Notify("text.editNew".into())]).unwrap();
        assert!(app.session.tool_wants_text());

        frame(&mut app, press(Key::Enter, Modifiers::COMMAND));

        assert!(!app.session.tool_wants_text(), "Ctrl+Enter must exit text edit mode");
        assert_eq!(text_and_layers(&mut app, id).0, "航展文字", "Ctrl+Enter keeps original text");
        assert_eq!(app.session.tool_id(), "selection", "editing ends in Selection tool");
        assert!(app.ui.dialog.is_none(), "Ctrl+Enter opens no dialog");
    }

    /// A command bound to Cmd+Enter runs without the active tool's Enter: the Selection tool opens""",
    "Add regression test for no-newline and text preservation",
)
path.write_text(source, encoding="utf-8", newline="\n")
print("SUCCESS: Ctrl+Enter exits canvas text editing and keeps text content unchanged.")

#!/usr/bin/env python3
"""Disable the Shift+Space Rotate View chord; keep Space pan and explicit Rotate View."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
def edit(rel,old,new):
    p=root/rel;s=p.read_text(encoding='utf-8')
    if s.count(old)!=1:raise SystemExit(f'FAIL Shift+Space: {rel}: {s.count(old)} anchors: {old[:70]}')
    p.write_text(s.replace(old,new),encoding='utf-8',newline='\n')
edit('crates/ui-egui/src/canvas/input.rs',
     '    // Held, Space is the Hand tool for the moment, Space+Shift is the Rotate View tool and',
     '    // Held, Space (also with Shift) is the Hand tool for the moment, and')
edit('crates/ui-egui/src/canvas/input.rs',
     'let rotate_mode = if space { m.shift && !m.command } else { tool == "rotateView" };',
     'let rotate_mode = !space && tool == "rotateView";')
edit('crates/ui-egui/src/canvas/input.rs',
     'let pan_mode = if space { !m.command && !m.shift } else { tool == "hand" };',
     'let pan_mode = if space { !m.command } else { tool == "hand" };')
edit('crates/ui-egui/src/canvas.rs',
     '(!zoom_drag && if space { m.shift && !m.command } else { app.session.tool_id() == "rotateView" })',
     '(!zoom_drag && !space && app.session.tool_id() == "rotateView")')
rel='crates/ui-egui/src/canvas/tests.rs'
edit(rel,'fn space_shift_drag_rotates_the_view_whatever_the_tool()', 'fn leo_text_mode_ui_shift_space_pans_without_rotating()')
edit(rel,'''    assert!((after.rotation - 90.0).abs() < 1e-3, "{after:?}");
    assert_eq!((after.center, after.zoom), (before.center, before.zoom), "the view turns about its center");''',
     '''    assert_eq!((after.rotation, after.zoom), (before.rotation, before.zoom), "Shift+Space must not rotate");
    assert_ne!(after.center, before.center, "Shift+Space keeps Space panning");''')
edit(rel,'    // Shift does not round the angle: 90° and 20° more make 110°, not 105°.',
     '    // Another Shift+Space drag must still keep the angle unchanged.')
edit(rel,'    assert!((app.view().unwrap().rotation - 110.0).abs() < 1e-3, "{:?}", app.view());',
     '    assert_eq!(app.view().unwrap().rotation, before.rotation);')
edit(rel,'fn the_rotate_cursor_shows_for_space_shift_and_the_rotate_view_tool()',
     'fn leo_text_mode_ui_shift_space_hand_and_explicit_rotate_cursor()')
edit(rel,'    assert_eq!(shown(&mut app, vec![Event::ModifiersChanged(Modifiers::SHIFT), space(true, Modifiers::SHIFT)]), CursorIcon::Alias);',
     '    assert_eq!(shown(&mut app, vec![Event::ModifiersChanged(Modifiers::SHIFT), space(true, Modifiers::SHIFT)]), CursorIcon::Grab);')
edit(rel,'    // Shift let go during the drag keeps the rotate cursor; after it, Space shows the hand.',
     '    // Shift let go during the pan keeps the grabbed hand; after it, Space shows the hand.')
edit(rel,'    assert_eq!(shown(&mut app, vec![Event::ModifiersChanged(Modifiers::NONE), Event::PointerMoved(c + vec2(20.0, 30.0))]), CursorIcon::Alias);',
     '    assert_eq!(shown(&mut app, vec![Event::ModifiersChanged(Modifiers::NONE), Event::PointerMoved(c + vec2(20.0, 30.0))]), CursorIcon::Grabbing);')
edit(rel,'fn a_rotate_view_drag_shows_a_dial_with_the_angle()',
     'fn leo_text_mode_ui_explicit_rotate_dial_but_no_shift_space_dial()')
edit(rel,'    // A Space+Shift drag with another tool shows it too: 30° and 90° more make 120°.',
     '    // Space+Shift now pans without a dial or an angle change.')
edit(rel,'    assert_eq!(dial(&mut app), (Some(c), Some(120), Some("120°".to_string())));',
     '    assert_eq!(dial(&mut app), (None, None, None));')
edit(rel,'    assert!((app.view().unwrap().rotation - 120.0).abs() < 1e-2, "{:?}", app.view());',
     '    assert!((app.view().unwrap().rotation - 30.0).abs() < 1e-2, "{:?}", app.view());')
print('Disabled Shift+Space view rotation; preserved Space pan and explicit Rotate View')

#!/usr/bin/env python3
"""LC-024: update the native Shift-click regression for deferred click toggling."""
from pathlib import Path
import sys
root = Path(sys.argv[1])
path = root / "crates/tools/src/direct/tests.rs"
s = path.read_text(encoding="utf-8")
old = """#[test]
fn shift_clicking_anchor_after_segment_pick_toggles_or_adds() {
    let (d, id) = doc_with_rect();
    let p = paint();
    let mut s = Selection::default();
    s.set([id]);
    s.segments.insert(id, [(0, 0)].into());
    s.anchors.insert(id, [(0, 0), (0, 1)].into());
    let cx = cx(&d, &s, &p);
    let mut t = DirectSelectionTool::new(false);
    let shift = Mods { shift: true, ..Default::default() };
    // Shift-click on anchor (0, 0) (top-left at 100, 100) toggles it off
    let a = t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 100.0, 100.0).with_mods(shift));
    assert_eq!(a, vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 0]], "mode": "toggle"}))]);
    // Shift-click on anchor (0, 2) (bottom-right at 200, 200) toggles it on (adds it)
    let a2 = t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 200.0, 200.0).with_mods(shift));
    assert_eq!(a2, vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 2]], "mode": "toggle"}))]);
}"""
new = """#[test]
fn shift_clicking_anchor_after_segment_pick_toggles_or_adds() {
    let (d, id) = doc_with_rect();
    let p = paint();
    let mut s = Selection::default();
    s.set([id]);
    s.segments.insert(id, [(0, 0)].into());
    s.anchors.insert(id, [(0, 0), (0, 1)].into());
    let cx = cx(&d, &s, &p);
    let mut t = DirectSelectionTool::new(false);
    let shift = Mods { shift: true, ..Default::default() };
    // An already-selected point stays selected on press; a click toggles it on release.
    assert!(t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 100.0, 100.0).with_mods(shift)).is_empty());
    assert_eq!(t.pointer(&cx, &PointerEvent::new(PointerKind::Up, 100.0, 100.0).with_mods(shift)),
        vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 0]], "mode": "toggle"}))]);
    // An unselected point is added immediately so a Shift-drag can move it.
    assert_eq!(t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 200.0, 200.0).with_mods(shift)),
        vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 2]], "mode": "add"}))]);
    assert!(t.pointer(&cx, &PointerEvent::new(PointerKind::Up, 200.0, 200.0).with_mods(shift)).is_empty());
}"""
if s.count(old) != 1:
    raise SystemExit(f"FAIL LC-024 Shift-click test anchor count={s.count(old)}")
path.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")
print("PASS LC-024 Shift-click and Shift-drag selection regressions")


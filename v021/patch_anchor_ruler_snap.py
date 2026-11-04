#!/usr/bin/env python3
"""LC-028: Smart Guides include visible rulers for direct-selected anchor/segment movement."""
from pathlib import Path
import sys
root = Path(sys.argv[1])

def change(rel, old, new, count=1):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    if s.count(old) != count:
        raise SystemExit(f"FAIL LC-028 {rel}: expected {count}, got {s.count(old)}")
    p.write_text(s.replace(old, new), encoding="utf-8", newline="\n")

change("crates/tools/src/guides/snap.rs", "impl PointSnap {\n", """impl PointSnap {
    /// Direct editing includes shown ruler guides with Smart Guides, independently of Snap to Point.
    /// Other drawing tools keep their native snapping preferences.
    pub fn for_anchor_drag(cx: &ToolContext) -> Self {
        Self::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection).with_rulers(cx))
    }

""")
# The group-anchor and partial-segment paths use the same target gathering as anchor dragging.
p = root / "crates/tools/src/direct.rs"
s = p.read_text(encoding="utf-8")
old = "PointSnap::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection))"
count = s.count(old)
if count != 4:
    raise SystemExit(f"FAIL LC-028 expected four direct-edit snap constructors, got {count}")
p.write_text(s.replace(old, "PointSnap::for_anchor_drag(cx)"), encoding="utf-8", newline="\n")

change("crates/tools/src/guides/targets.rs",
       "        let preview = self.extension_preview(p, tol);",
       "        let preview = if from.and_then(|f| f.constrained(p)).is_none() { self.extension_preview(p, tol) } else { None };")

with (root / "crates/tools/src/direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write('''

#[test]
fn leo_shift_ruler_snap_horizontal_vertical_both_signs_and_without_snap_to_point() {
    use vectorcraft_geom::shapes;
    // (vertical ruler, pointer, expected delta). Source anchor is (380,340).
    for (vertical, pointer, dx, dy, pos) in [
        (true, (403.5, 341.0), 25.0, 0.0, 405.0),
        (true, (356.5, 339.0), -25.0, 0.0, 355.0),
        (false, (381.0, 403.5), 0.0, 65.0, 405.0),
        (false, (379.0, 276.5), 0.0, -65.0, 275.0),
    ] {
        let (mut d, _) = doc_with_rect();
        let layer = d.layers[0].id;
        let source = d.alloc_id();
        d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
            shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
            vectorcraft_doc::Appearance::default_art())).unwrap();
        d.guides.push(vectorcraft_doc::Guide::new(vertical, pos));
        let selected = anchors_of(source, &[(0, 1)]);
        let p = paint();
        let c = ToolContext { smart_guides: true, snap_to_point: false, constrain_angle: 30.0, ..cx(&d, &selected, &p) };
        let shift = Mods { shift: true, ..Mods::default() };
        let mut t = DirectSelectionTool::new(false);
        t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0).with_mods(shift));
        let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, pointer.0, pointer.1).with_mods(shift));
        let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
        assert_eq!(cmd, "path.moveAnchors");
        assert!((v["dx"].as_f64().unwrap() - dx).abs() < 1e-6, "{v}");
        assert!((v["dy"].as_f64().unwrap() - dy).abs() < 1e-6, "{v}");
        assert!(!t.guides.is_empty());
    }
}

#[test]
fn leo_shift_ruler_snap_respects_hidden_guides_and_disabled_snapping() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let source = d.alloc_id();
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.guides.push(vectorcraft_doc::Guide::new(false, 405.0));
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    for (visible, smart) in [(false, true), (true, false)] {
        let c = ToolContext { guides: visible, smart_guides: smart, snap_to_point: false, ..cx(&d, &selected, &p) };
        let shift = Mods { shift: true, ..Mods::default() };
        let mut t = DirectSelectionTool::new(false);
        t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0).with_mods(shift));
        let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 380.0, 403.5).with_mods(shift));
        let Some(Action::Preview(_, v)) = moved.last() else { panic!("{moved:?}") };
        assert!((v["dy"].as_f64().unwrap() - 63.5).abs() < 1e-6, "{v}");
    }
}
#[test]
fn leo_shift_ruler_snap_moves_a_selected_segment_as_one_piece() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let source = d.alloc_id();
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.guides.push(vectorcraft_doc::Guide::new(false, 405.0));
    let mut selected = anchors_of(source, &[(0, 0), (0, 1)]);
    selected.segments.insert(source, [(0, 0)].into_iter().collect());
    let p = paint();
    let c = ToolContext { smart_guides: true, snap_to_point: false, ..cx(&d, &selected, &p) };
    let mut t = DirectSelectionTool::new(false);
    t.pointer(&c, &PointerEvent::new(PointerKind::Down, 340.0, 340.0));
    let shift = Mods { shift: true, ..Mods::default() };
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 340.0, 403.5).with_mods(shift));
    let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
    assert_eq!(cmd, "path.moveAnchors");
    assert!(v["dx"].as_f64().unwrap().abs() < 1e-6);
    assert!((v["dy"].as_f64().unwrap() - 65.0).abs() < 1e-6, "{v}");
}
''')
print("PASS LC-028 anchor and segment ruler snapping")

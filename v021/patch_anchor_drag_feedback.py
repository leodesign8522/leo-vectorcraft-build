#!/usr/bin/env python3
"""Leo Design Snapping Engine v1: replace Direct Selection's drag snapping with real geometry."""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_anchor_drag_feedback.py VECTORCRAFT_ROOT")

root = Path(sys.argv[1]) / "crates/tools/src"

def edit(rel, old, new, expected=1):
    path = root / rel
    source = path.read_text(encoding="utf-8")
    count = source.count(old)
    if count != expected:
        raise SystemExit(f"FAIL Leo Snapping Engine: {rel}, expected {expected} anchors, found {count}")
    path.write_text(source.replace(old, new), encoding="utf-8", newline="\n")

edit("guides/targets.rs",
     "pub struct Targets {",
     "pub struct Targets {\n    pub(super) leo_snap: std::cell::Cell<Option<SnapLock>>,")
edit("guides/targets.rs",
     "use super::snap::",
     '#[path = "leo_snapping_engine.rs"]\nmod leo_snapping_engine;\nuse leo_snapping_engine::SnapLock;\n\nuse super::snap::')
(root / "guides/leo_snapping_engine.rs").write_text(
    Path(__file__).with_name("leo_snapping_engine.rs").read_text(encoding="utf-8"),
    encoding="utf-8", newline="\n"
)

edit("guides/snap.rs", "impl PointSnap {", """impl PointSnap {
    /// Dedicated node-editing target set: do not let bounds, alignments, grids or pixel
    /// snapping override the new true-geometry target detection.
    pub(crate) fn leo_for_anchor_drag(cx: &ToolContext) -> Self {
        let targets = (cx.smart_guides || cx.snap_to_point).then(|| {
            (Targets::leo_for_anchor_drag(cx), 8.0)
        });
        Self { targets }
    }

    pub(crate) fn direct_anchor_drag(&self, cx: &ToolContext, p: Point, from: Option<&Leave>, shift: bool) -> (Point, Vec<Overlay>) {
        // Phase one only replaces unconstrained node dragging; retain native Shift
        // direction constraint until it is independently migrated and tested.
        if shift {
            return self.snap_from(cx, p, from);
        }
        if let Some((targets, _)) = &self.targets {
            if let Some(hit) = targets.leo_snap_anchor(p, cx.tol(8.0), cx.tol(8.0 * 1.6)) {
                return hit;
            }
        }
        // Retain explicit pixel/grid snap settings as a fallback when no geometry
        // is in range; they cannot steal a node/path geometry capture.
        if cx.snap_to_pixel || cx.snap_to_grid {
            return self.snap_from(cx, p, from);
        }
        (p, vec![])
    }
""")
# Dragging the selected anchor, or dragging a partial selection from a fill,
# must both use the same target set and geometric snap calculation.
edit("direct.rs",
     "PointSnap::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection))",
     "PointSnap::leo_for_anchor_drag(cx)", expected=3)
edit("direct.rs",
     "snap.snap_from(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()))",
     "snap.direct_anchor_drag(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()), ev.mods.shift)")
edit("direct.rs",
     "snap.snap_from(cx, held + (p - start), shift.as_ref())",
     "snap.direct_anchor_drag(cx, held + (p - start), shift.as_ref(), ev.mods.shift)")
# When a whole path is selected, pressing one of its anchors enters
# MoveObject (not MoveAnchors). This path previously fell through to
# PointSnap::snap_from and completely bypassed our new geometry engine.
edit("direct.rs",
     "snap.snap_from(cx, grab + (p - start), shift.as_ref())",
     "snap.direct_anchor_drag(cx, grab + (p - start), shift.as_ref(), ev.mods.shift)")

# Pointer-to-command integration: a real target must be reflected in Preview(dx,dy).
with (root / "direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_snapping_engine_direct_drag_preview_uses_exact_anchor_and_segment() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 200.0), Point::new(330.0, 200.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target,
        shapes::line(Point::new(430.0, 300.0), Point::new(430.0, 400.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &cx, (330.0, 200.0), (434.0, 351.0), Mods::default());
    assert_eq!(v, json!({"dx": 100.0, "dy": 151.0}), "{labels:?}");
    assert_eq!(labels, vec!["path".to_string()]);

    let (v, labels) = drag(&mut tool, &cx, (330.0, 200.0), (428.0, 302.0), Mods::default());
    assert_eq!(v, json!({"dx": 100.0, "dy": 100.0}), "{labels:?}");
    assert_eq!(labels, vec!["anchor".to_string()]);
}
""")
# Screenshot regression: grab an anchor of a fully selected path and snap
# the whole polygon to the visible right edge of an otherwise blank artboard.
with (root / "direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_snapping_engine_whole_selected_path_anchor_hits_artboard_edge() {
    use vectorcraft_geom::shapes;
    let mut d = vectorcraft_doc::Document::new(500.0, 500.0);
    let layer = d.layers[0].id;
    let id = d.alloc_id();
    d.insert(Some(layer), 0, vectorcraft_doc::Node::path(id,
        shapes::line(Point::new(600.0, 200.0), Point::new(630.0, 200.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let mut selected = Selection::default();
    selected.set([id]);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &cx, (600.0, 200.0), (504.0, 235.0), Mods::default());
    assert_eq!(v["matrix"], json!([1.0, 0.0, 0.0, 1.0, -100.0, 35.0]), "{v:?} {labels:?}");
    assert_eq!(labels, vec!["path".to_string()]);
}

#[test]
fn leo_snapping_engine_partial_anchor_hits_artboard_edge() {
    use vectorcraft_geom::shapes;
    let mut d = vectorcraft_doc::Document::new(500.0, 500.0);
    let layer = d.layers[0].id;
    let id = d.alloc_id();
    d.insert(Some(layer), 0, vectorcraft_doc::Node::path(id,
        shapes::line(Point::new(600.0, 200.0), Point::new(630.0, 200.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(id, &[(0, 0)]);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &cx, (600.0, 200.0), (504.0, 235.0), Mods::default());
    assert_eq!(v, json!({"dx": -100.0, "dy": 35.0}), "{labels:?}");
    assert_eq!(labels, vec!["path".to_string()]);
}
""")
print("PASS Leo Design Snapping Engine v1.1: screenshot artboard edge and whole-object anchor drag regression")

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
     "pub struct Targets {\n    pub(super) leo_scale: std::cell::RefCell<Option<LeoScaleLock>>,\n    pub(super) leo_snap: std::cell::Cell<Option<SnapLock>>,\n    pub(super) leo_intersection: std::cell::Cell<Option<LeoIntersectionLock>>,\n    pub(super) leo_drag_origin: Option<Point>,\n    pub(super) leo_extension: std::cell::Cell<Option<LeoExtension>>,\n    pub(super) leo_extension_index: Option<LeoExtensionIndex>,\n    pub(super) leo_scene: Option<LeoScene>,")
edit("guides/targets.rs",
     "use super::snap::",
     '#[path = "leo_snapping_engine.rs"]\nmod leo_snapping_engine;\nuse leo_snapping_engine::{LeoScene, SnapLock, LeoExtension, LeoExtensionIndex, LeoScaleLock, LeoIntersectionLock};\npub(crate) use leo_snapping_engine::ObjectMoveSnap;\n\nuse super::snap::')
(root / "guides/leo_snapping_engine.rs").write_text(
    Path(__file__).with_name("leo_snapping_engine.rs").read_text(encoding="utf-8")+"\n"+Path(__file__).with_name("rotation_snapping.rs").read_text(encoding="utf-8"),
    encoding="utf-8", newline="\n"
)

edit("guides.rs",
     "pub use targets::Targets;",
     "pub use targets::Targets;\npub(crate) use targets::ObjectMoveSnap;")
edit("guides/targets.rs","pub(crate) use leo_snapping_engine::ObjectMoveSnap;","pub(crate) use leo_snapping_engine::{ObjectMoveSnap,LeoRotationSnap};")
edit("guides.rs","pub(crate) use targets::ObjectMoveSnap;","pub(crate) use targets::{ObjectMoveSnap,LeoRotationSnap};")
edit("xform/transform.rs","use crate::guides::{PointSnap, Targets, snap_pick};","use crate::guides::{PointSnap, Targets, snap_pick, LeoRotationSnap};")
edit("xform/transform.rs","    snap: Option<PointSnap>,","    snap: Option<PointSnap>,\n    rotation: Option<LeoRotationSnap>,")
edit("xform/transform.rs","hover: vec![], snap: None }","hover: vec![], snap: None, rotation: None }")
edit("xform/transform.rs","        self.snap = None;","        self.snap = None;\n        self.rotation = None;")
edit("xform/transform.rs","                    self.snap = Some(PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)));","                    self.snap = Some(PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)));\n                    self.rotation=if self.kind==TransformKind::Rotate {LeoRotationSnap::new(cx,d.origin)} else {None};")
edit("xform/transform.rs","                let (p, guides) = self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p));","                let (p, guides) = if self.rotation.is_some() {(p,vec![])} else {self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p))};")
edit("xform/transform.rs","                let (m, text) = self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds);","""                let (mut m, mut text) = self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds);
                if let Some(rotation)=&self.rotation {
                    let raw=(p-d.origin).atan2()-(d.pick-d.origin).atan2();
                    let c=m.as_coeffs();text=format!("{:.1}°",vectorcraft_geom::normalize_deg(-c[1].atan2(c[0]).to_degrees()));
                    if let Some((matrix,angle,guides))=rotation.snap(cx,raw,p,ev.mods.shift || ev.backquote) {
                        m=matrix;text=format!("{:.1}°",vectorcraft_geom::normalize_deg(-angle.to_degrees()));self.hover=guides;
                    }
                }""")

edit("guides/snap.rs", "impl PointSnap {", """impl PointSnap {
    pub(crate) fn leo_for_scale(cx: &ToolContext) -> Self {
        Self {targets:(cx.smart_guides || cx.snap_to_point).then(|| (Targets::leo_scale_targets(cx),8.0))}
    }

    /// Dedicated node-editing target set: do not let bounds, alignments, grids or pixel
    /// snapping override the new true-geometry target detection.
    pub(crate) fn leo_for_anchor_drag(cx: &ToolContext) -> Self {
        let targets = (cx.smart_guides || cx.snap_to_point).then(|| {
            (Targets::leo_for_anchor_drag(cx), 8.0)
        });
        Self { targets }
    }

    pub(crate) fn leo_captured(&self) -> bool {
        self.targets.as_ref().is_some_and(|(targets, _)| targets.leo_snap.get().is_some() || targets.leo_extension.get().is_some() || targets.leo_intersection.get().is_some())
    }

    pub(crate) fn direct_anchor_drag(&self, cx: &ToolContext, p: Point, from: Option<&Leave>, shift: bool) -> (Point, Vec<Overlay>) {
        let constrained = if shift { from.and_then(|f| f.constrained(p)) } else { None };
        let p = constrained.unwrap_or(p);
        if let Some((targets, _)) = &self.targets {
            if !shift && let Some(f) = from.filter(|f| f.contact)
                && let Some(hit) = targets.leo_snap_contact(p, f.at, cx.tol(8.0)) {
                targets.leo_extension.set(None);
                return hit;
            }
            // Existing perpendicular/tangent contact takes priority for open
            // line endpoints; otherwise a real guide/path crossing wins over
            // the generic nearest-path projection.
            if cx.smart_guides {
                if let Some(hit) = targets.leo_snap_guide_intersection(p, from, shift, cx.tol(8.0), cx.tol(12.8)) {
                    return hit;
                }
            } else {
                targets.leo_intersection.set(None);
            }
            let hit = if let (Some(q), Some(f)) = (constrained, from) {
                targets.leo_snap_anchor_along(q, f.at, cx.tol(8.0), cx.tol(12.8))
            } else {
                targets.leo_snap_anchor(p, cx.tol(8.0), cx.tol(12.8))
            };
            if let Some(hit) = hit {
                targets.leo_extension.set(None);
                return hit;
            }
            let direction = constrained.zip(from).map(|(q,f)| q-f.at);
            let extension = if let Some(view)=cx.screen.as_ref() {
                targets.leo_snap_extension_in_view(p,cx.tol(8.0),cx.tol(12.8),direction,Some(view))
            } else {
                targets.leo_snap_extension(p,cx.tol(8.0),cx.tol(12.8),direction)
            };
            if let Some(hit) = extension {
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
edit("select.rs", "PointSnap::new(cx, || Targets::for_scale(cx))", "PointSnap::leo_for_scale(cx)")
edit("guides/snap.rs", "        let step = if cx.snap_to_pixel {", """        if let Some((targets,_))=&self.targets {
            if let Some(hit)=targets.leo_resize(cx,bx,handle,a,proportional,from_center) {return hit;}
        }
        let step = if cx.snap_to_pixel {""")
# Dragging the selected anchor, or dragging a partial selection from a fill,
# must both use the same target set and geometric snap calculation.
edit("select.rs",
     "use crate::guides::{PointSnap, Targets, distance_guides, hold_on_path};",
     "use crate::guides::{ObjectMoveSnap, PointSnap, Targets, distance_guides, hold_on_path};")
edit("select.rs",
     "    rulers: Option<Targets>,\n}",
     "    rulers: Option<Targets>,\n    /// Leo Design's shared geometry engine for whole-object translation.\n    leo: Option<ObjectMoveSnap>,\n}")
edit("select.rs",
     "            rulers: (guides && cx.snap_to_point).then(|| Targets::rulers(cx)),",
     "            rulers: (guides && cx.snap_to_point).then(|| Targets::rulers(cx)),\n            leo: (cx.smart_guides || cx.snap_to_point)\n                .then(|| ObjectMoveSnap::new(cx, selection_bounds(cx))),")
edit("select.rs",
     "        Self { bounds, targets: None, points: None, copy_targets: None, copy_points: None, rulers: None }",
     "        Self { bounds, targets: None, points: None, copy_targets: None, copy_points: None, rulers: None, leo: None }")
edit("select.rs",
     "    pub(crate) fn snap(&self, cx: &ToolContext, start: Point, mut d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {",
     "    pub(crate) fn snap(&self, cx: &ToolContext, start: Point, mut d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {\n        if let Some(leo) = &self.leo {\n            let hit = leo.snap(cx, start, d, copy);\n            if leo.captured() || !(cx.snap_to_grid || cx.snap_to_pixel) || (hit.0 - d).hypot() > 1e-9 { return hit; }\n        }")
edit("select.rs",
     "    pub(crate) fn snap_along(&self, cx: &ToolContext, start: Point, mut d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {",
     "    pub(crate) fn snap_along(&self, cx: &ToolContext, start: Point, mut d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {\n        if let Some(leo) = &self.leo {\n            let hit = leo.snap_along(cx, start, d, copy);\n            if leo.captured() || !(cx.snap_to_grid || cx.snap_to_pixel) || (hit.0 - d).hypot() > 1e-9 { return hit; }\n        }")

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

# Alt-copy must gather geometry even with grid/pixel options active.
edit("guides/snap.rs", "    pub fn new(cx: &ToolContext, targets:", """    pub(crate) fn leo_for_anchor_copy(cx: &ToolContext) -> Self {
        let targets = (cx.smart_guides || cx.snap_to_point).then(|| {
            (Targets::leo_for_object_move(cx, &[]), 8.0)
        });
        Self { targets }
    }

    pub fn new(cx: &ToolContext, targets:""")
edit("direct.rs", "PointSnap::new(cx, || Targets::for_anchor_drag(cx.doc, &Selection::default()))",
     "PointSnap::leo_for_anchor_copy(cx)")
edit("direct.rs", '''            if let Some(grid) = &self.partial_grid {
                (d, self.guides) = if ev.mods.shift { grid.snap_along(cx, held, d, false) } else { grid.snap(cx, held, d, false) };
            } else {
                let shift = ev.mods.shift.then(|| Leave::segment(cx, held, true));
                let (q, guides) = snap.direct_anchor_drag(cx, held + (p - start), shift.as_ref(), ev.mods.shift);
                (d, self.guides) = (q - held, guides);
            }''', '''            let shift = ev.mods.shift.then(|| Leave::segment(cx, held, true));
            let (q, guides) = snap.direct_anchor_drag(cx, held + (p - start), shift.as_ref(), ev.mods.shift);
            (d, self.guides) = (q - held, guides);
            if !snap.leo_captured() && let Some(grid) = &self.partial_grid {
                let raw = move_delta(start, p, ev.mods.shift, cx.constrain_angle);
                (d, self.guides) = if ev.mods.shift { grid.snap_along(cx, held, raw, false) } else { grid.snap(cx, held, raw, false) };
            }''')

# Shift-click toggles a selected anchor only on release. Toggling it on Down
# empties the live Session selection before path.moveAnchors can move anything.
edit("direct.rs",
     "    click: Option<(NodeId, usize, usize)>,\n    /// A click on a whole path picks the segment on release;",
     "    click: Option<(NodeId, usize, usize)>,\n    /// A selected anchor's Shift-click toggles only when released without dragging.\n    shift_deselect: Option<(NodeId, usize, usize)>,\n    /// A click on a whole path picks the segment on release;")
edit("direct.rs", "            click: None,\n            click_segment: None,",
     "            click: None,\n            shift_deselect: None,\n            click_segment: None,")
edit("direct.rs",
     "        (self.click, self.click_segment, self.click_group, self.frame_click) = (None, None, None, None);",
     "        (self.click, self.click_segment, self.click_group, self.frame_click) = (None, None, None, None);\n        self.shift_deselect = None;")
edit("direct.rs",
     "    fn press_anchor(&mut self, cx: &ToolContext, ev: &PointerEvent, p: Point, (id, si, ai, grab): (NodeId, usize, usize, Point)) -> Vec<Action> {\n",
     "    fn press_anchor(&mut self, cx: &ToolContext, ev: &PointerEvent, p: Point, (id, si, ai, grab): (NodeId, usize, usize, Point)) -> Vec<Action> {\n        self.shift_deselect = None;\n")
edit("direct.rs",
     '        if ev.mods.shift {\n            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": "toggle"}))];\n        }\n',
     '''        if ev.mods.shift {
            if already || cx.selection.anchors.get(&id).is_some_and(|anchors| anchors.contains(&(si, ai))) {
                self.shift_deselect = Some((id, si, ai));
                return vec![];
            }
            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": "toggle"}))];
        }
''')
edit("direct.rs",
     "        let group = self.click_group.take();\n        // A click on one of several selected frame anchors selects it alone.",
     '''        let group = self.click_group.take();
        let shift_deselect = self.shift_deselect.take();
        if !began && let Some((id, si, ai)) = shift_deselect {
            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": "toggle"}))];
        }
        // A click on one of several selected frame anchors selects it alone.''')
edit("direct/tests.rs",
     '''    let a = t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 100.0, 100.0).with_mods(shift));
    assert_eq!(a, vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 0]], "mode": "toggle"}))]);''',
     '''    assert!(t.pointer(&cx, &PointerEvent::new(PointerKind::Down, 100.0, 100.0).with_mods(shift)).is_empty());
    let a = t.pointer(&cx, &PointerEvent::new(PointerKind::Up, 100.0, 100.0).with_mods(shift));
    assert_eq!(a, vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[0, 0]], "mode": "toggle"}))]);''', expected=2)

# These native assertions describe the deliberately replaced node contract:
# Finite geometry and original-edge extension constraints have distinct labels.
edit("direct/tests.rs",
     '    assert_eq!((v, labels), (json!({"dx": 0.0, "dy": 30.0}), vec!["align".to_string()]));',
     '    assert_eq!((v, labels), (json!({"dx": 0.0, "dy": 30.0}), vec!["延长线吸附".to_string()]), "original edge direction is an extension constraint");')
edit("direct/tests.rs",
     '    assert_eq!(drag(&mut t, &c, (300.0, 300.0), (203.0, 199.0), none).0, json!({"dx": -97.0, "dy": -101.0}));',
     '    assert_eq!(drag(&mut t, &c, (300.0, 300.0), (203.0, 199.0), none).0, json!({"dx": -100.0, "dy": -100.0}), "Snap to Point uses the shared 8px capture");')
edit("direct/tests.rs",
     '    assert_eq!(drag(&mut t, &c, (260.0, 170.0), (202.0, 163.0), none).1, Vec::<String>::new());',
     '    assert_eq!(drag(&mut t, &c, (260.0, 170.0), (202.0, 163.0), none).1, ["path"], "Snap to Point also captures real finite paths");')

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

with (root / "select/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_object_move_snaps_selection_edge_to_artboard_edge() {
    let (d, id) = doc_with_rect();
    let mut selected = Selection::default();
    selected.add(id);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = SelectionTool::default();
    tool.pointer(&cx, &ev(PointerKind::Down, 150.0, 150.0));
    let actions = tool.pointer(&cx, &ev(PointerKind::Drag, 449.0, 150.0));
    let Some(Action::Preview(name, value)) = actions.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else {
        panic!("no object preview: {actions:?}");
    };
    assert_eq!(name, "object.transform");
    assert_eq!(value["matrix"][4], json!(300.0), "the right edge must land on artboard x=500: {value}");
}

#[test]
fn leo_object_move_snaps_multiple_selected_objects_as_one_rigid_selection() {
    use vectorcraft_doc::{Appearance, Node};
    let (mut d, first) = doc_with_rect();
    let second = d.alloc_id();
    d.insert(Some(d.layers[0].id), 1, Node::path(
        second,
        vectorcraft_geom::shapes::rectangle(Rect::new(220.0, 100.0, 260.0, 200.0)),
        Appearance::default_art(),
    )).unwrap();
    let mut selected = Selection::default();
    selected.set([first, second]);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = SelectionTool::default();
    tool.pointer(&cx, &ev(PointerKind::Down, 150.0, 150.0));
    let actions = tool.pointer(&cx, &ev(PointerKind::Drag, 389.0, 150.0));
    let Some(Action::Preview(_, value)) = actions.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else {
        panic!("no multi-object preview: {actions:?}");
    };
    assert_eq!(value["matrix"][4], json!(240.0), "union right edge must land on x=500: {value}");
}

#[test]
fn leo_object_move_snaps_a_selected_group_without_deforming_children() {
    use std::sync::Arc;
    use vectorcraft_doc::{Appearance, Node};
    let mut d = vectorcraft_doc::Document::new(500.0, 500.0);
    let layer = d.layers[0].id;
    let a = Node::path(d.alloc_id(), vectorcraft_geom::shapes::rectangle(Rect::new(100.0, 100.0, 150.0, 200.0)), Appearance::default_art());
    let b = Node::path(d.alloc_id(), vectorcraft_geom::shapes::rectangle(Rect::new(160.0, 100.0, 200.0, 200.0)), Appearance::default_art());
    let group = Node::group(d.alloc_id(), vec![Arc::new(a), Arc::new(b)]);
    let group_id = group.id;
    d.insert(Some(layer), 0, group).unwrap();
    let mut selected = Selection::default();
    selected.add(group_id);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = SelectionTool::default();
    tool.pointer(&cx, &ev(PointerKind::Down, 125.0, 125.0));
    let actions = tool.pointer(&cx, &ev(PointerKind::Drag, 424.0, 125.0));
    let Some(Action::Preview(_, value)) = actions.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else {
        panic!("no group preview: {actions:?}");
    };
    assert_eq!(value["matrix"][4], json!(300.0), "group bounds must land on artboard edge: {value}");
}

#[test]
fn leo_object_move_alt_copy_can_snap_to_the_original_without_moving_the_source() {
    let (d, id) = doc_with_rect();
    let mut selected = Selection::default();
    selected.add(id);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut down = ev(PointerKind::Down, 150.0, 150.0);
    down.mods.alt = true;
    let mut drag = ev(PointerKind::Drag, 160.0, 150.0);
    drag.mods.alt = true;
    let mut tool = SelectionTool::default();
    tool.pointer(&cx, &down);
    let actions = tool.pointer(&cx, &drag);
    let Some(Action::Preview(_, value)) = actions.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else {
        panic!("no copy preview: {actions:?}");
    };
    assert_eq!(value["copy"], json!(true));
    assert_eq!(value["matrix"][4], json!(10.0), "the copy remains movable while original geometry is a target: {value}");
}
""")

with (root / "direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_object_move_direct_selection_path_body_snaps_to_artboard_edge() {
    let (d, id) = doc_with_rect();
    let mut selected = Selection::default();
    selected.add(id);
    let p = paint();
    let cx = cx(&d, &selected, &p);
    let mut tool = DirectSelectionTool::new(false);
    tool.pointer(&cx, &PointerEvent::new(PointerKind::Down, 150.0, 150.0));
    let actions = tool.pointer(&cx, &PointerEvent::new(PointerKind::Drag, 449.0, 150.0));
    let Some(Action::Preview(name, value)) = actions.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else {
        panic!("no Direct Selection object preview: {actions:?}");
    };
    assert_eq!(name, "object.transform");
    assert_eq!(value["matrix"][4], json!(300.0), "whole path right edge must land on artboard x=500: {value}");
}
""")




# Give the extension cue a dark backing so yellow remains legible on a white page.
ui_path=Path(sys.argv[1]) / "crates/ui-egui/src/canvas/overlays.rs"
ui_source=ui_path.read_text(encoding="utf-8")
ui_anchor='                let sp = xf.to_screen(*pt) + vec2(8.0, -14.0);'
if ui_source.count(ui_anchor)!=1: raise SystemExit("FAIL extension label background anchor")
ui_source=ui_source.replace(ui_anchor,ui_anchor+'\n                if text == "延长线吸附" || text.starts_with("旋转吸附") {\n                    let galley=p.layout_no_wrap(text.to_owned(),egui::FontId::proportional(11.0),c32(*color));\n                    let backdrop=egui::Rect::from_min_size(sp-vec2(3.0,2.0),galley.size()+vec2(6.0,4.0));\n                    p.rect_filled(backdrop,CornerRadius::same(3),Color32::from_rgba_unmultiplied(0x20,0x24,0x2b,235));\n                }\n')
ui_path.write_text(ui_source,encoding="utf-8",newline="\n")

print("PASS Leo Snapping Engine 1.7: finite geometry, indexed extensions and yellow feedback")

# Resize now shares Leo's fixed 8 screen-pixel acquisition contract. The native
# readout-preference checks still run, with the superseded 0/4px pull updated.
edit("select/tests.rs",
     "for (tol, want) in [(4.0, 94.0), (8.0, 100.0), (0.0, 94.0)] {",
     "for (tol, want) in [(4.0, 100.0), (8.0, 100.0), (0.0, 100.0)] { // Leo: fixed 8 screen pixels")

with (root / "xform/transform.rs").open("a",encoding="utf-8") as f:
    f.write(Path(__file__).with_name("rotation_tool_regressions.rs").read_text(encoding="utf-8"))

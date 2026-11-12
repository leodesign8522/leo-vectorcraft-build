#!/usr/bin/env python3
"""Install the optional Leo Design drag snapping engine on VectorCraft 0.24.1."""
from pathlib import Path
import re
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_anchor_drag_feedback.py VECTORCRAFT_ROOT")

upstream = Path(sys.argv[1])
root = upstream / "crates/tools/src"
legacy = Path(__file__).resolve().parent.parent / "v023"

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
    (legacy / "leo_snapping_engine.rs").read_text(encoding="utf-8")+"\n"+(Path(__file__).with_name("rotation_snapping.rs")).read_text(encoding="utf-8"),
    encoding="utf-8", newline="\n"
)

edit("guides/leo_snapping_engine.rs",
     "        self.leo_scene.as_ref()?;\n        let acquire=cx.tol(ACQUIRE_PX);",
     "        if cx.snap_to_grid { return None; }\n        self.leo_scene.as_ref()?;\n        let acquire=cx.tol(ACQUIRE_PX);")

edit("guides.rs",
     "pub use targets::Targets;",
     "pub use targets::Targets;\npub(crate) use targets::ObjectMoveSnap;")
edit("guides/targets.rs","pub(crate) use leo_snapping_engine::ObjectMoveSnap;","pub(crate) use leo_snapping_engine::{ObjectMoveSnap,LeoRotationSnap};")

# Keep native scaling math and grid snapping. Restrict only the Bridge's
# fallback to nearby, visible object alignment and mark its actual target.
edit("guides/targets.rs",
     """        type Hit = Option<(f64, Point)>;
        // Where a snap lands: in line with a target's x and/or y, or on an anchor or centre.""",
     """        self.snap_scale_impl(bx, handle, a, proportional, from_center, tol, None)
    }

    fn snap_scale_impl(
        &self, bx: &OrientedBox, handle: Handle, a: Affine, proportional: bool,
        from_center: bool, tol: f64, local: Option<(f64, Option<&crate::ScreenFrame>)>,
    ) -> (Affine, Vec<Overlay>) {
        type Hit = Option<(f64, Point, Kind)>;
        // Where a snap lands: in line with a target's x and/or y, or on an anchor or centre.""")
edit("guides/targets.rs",
     """                .copied()
                .chain(rulers)
                .map(|(t, from, _)| (t, from))
                .filter(|(t, _)| (t - v).abs() <= tol)
                .min_by(|p, q| (p.0 - v).abs().total_cmp(&(q.0 - v).abs()))""",
     """                .copied()
                .filter(|(_, from, _)| local.is_none_or(|(reach, view)| {
                    let along = if x { (p.y - from.y).abs() } else { (p.x - from.x).abs() };
                    along <= reach && view.is_none_or(|view| {
                        view.to_screen(*from).is_some_and(|(x, y)| {
                            (0.0..=view.size.0).contains(&x) && (0.0..=view.size.1).contains(&y)
                        })
                    })
                }))
                .chain(rulers.map(|(value, from, kind)| {
                    let target = if local.is_none() { from } else if x {
                        Point::new(value, p.y)
                    } else {
                        Point::new(p.x, value)
                    };
                    (value, target, kind)
                }))
                .filter(|(t, _, _)| (t - v).abs() <= tol)
                .min_by(|a, b| {
                    (a.0 - v).abs().total_cmp(&(b.0 - v).abs()).then_with(|| {
                        if local.is_some() { a.1.distance(p).total_cmp(&b.1.distance(p)) }
                        else { std::cmp::Ordering::Equal }
                    })
                })""")
edit("guides/targets.rs",
     'let on = self.nearest_point(p, tol).map(|(q, k)| (Some((q.x, q)), Some((q.y, q)), Some((q, k))));',
     'let on = self.nearest_point(p, tol).map(|(q, k)| (Some((q.x, q, k)), Some((q.y, q, k)), Some((q, k))));')
edit("guides/targets.rs",
     """        if let Some((x, from)) = hit_x {
            ov.extend(self.line(Point::new(x, from.y.min(nr.y0)), Point::new(x, from.y.max(nr.y1))));
        }
        if let Some((y, from)) = hit_y {
            ov.extend(self.line(Point::new(from.x.min(nr.x0), y), Point::new(from.x.max(nr.x1), y)));
        }
        if hit_x.is_some() || hit_y.is_some() {
            ov.extend(self.label(to_doc * (out * handle.pos(r)), "align"));
        }""",
     """        if let Some((x, from, kind)) = hit_x {
            ov.extend(self.line(Point::new(x, from.y.min(nr.y0)), Point::new(x, from.y.max(nr.y1))));
            if local.is_some() {
                ov.extend(leo_snapping_engine::alignment_marker(self, from, kind));
            }
        }
        if let Some((y, from, kind)) = hit_y {
            ov.extend(self.line(Point::new(from.x.min(nr.x0), y), Point::new(from.x.max(nr.x1), y)));
            if local.is_some() && !hit_x.is_some_and(|(_, point, _)| point == from) {
                ov.extend(leo_snapping_engine::alignment_marker(self, from, kind));
            }
        }
        if local.is_none() && (hit_x.is_some() || hit_y.is_some()) {
            ov.extend(self.label(to_doc * (out * handle.pos(r)), "align"));
        }""")
edit("guides.rs","pub(crate) use targets::ObjectMoveSnap;","pub(crate) use targets::{ObjectMoveSnap,LeoRotationSnap};")
edit("xform/transform.rs","use crate::guides::{PointSnap, Targets, snap_pick};","use crate::guides::{PointSnap, Targets, snap_pick, LeoRotationSnap};")
edit("xform/transform.rs","    snap: Option<PointSnap>,","    snap: Option<PointSnap>,\n    rotation: Option<LeoRotationSnap>,")
edit("xform/transform.rs","hover: vec![], snap: None }","hover: vec![], snap: None, rotation: None }")
edit("xform/transform.rs","        self.snap = None;","        self.snap = None;\n        self.rotation = None;")
edit("xform/transform.rs","                    self.snap = Some(PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)));","                    self.rotation = if cx.leo_snapping && !cx.snap_to_grid && self.kind == TransformKind::Rotate { LeoRotationSnap::new(cx, d.origin) } else { None };\n                    self.snap = self.rotation.is_none().then(|| PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)));")
edit("xform/transform.rs","                let (p, guides) = self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p));","                let (p, guides) = if self.rotation.is_some() {(p,vec![])} else {self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p))};")
edit("xform/transform.rs","                let (m, text) = self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds);","""                let (mut m, mut text) = self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds);
                if let Some(rotation)=&self.rotation {
                    let raw=(p-d.origin).atan2()-(d.pick-d.origin).atan2();
                    let c=m.as_coeffs();text=format!("{:.1}°",vectorcraft_geom::normalize_deg(-c[1].atan2(c[0]).to_degrees()));
                    if let Some((matrix,angle,guides))=rotation.snap(cx,raw,p,ev.mods.shift || ev.backquote) {
                        m=matrix;text=format!("{:.1}°",vectorcraft_geom::normalize_deg(-angle.to_degrees()));self.hover=guides;
                    }
                }""")

edit("guides/snap.rs", "    targets: Option<(Targets, f64)>,\n}", "    targets: Option<(Targets, f64)>,\n    leo: bool,\n}")
edit("guides/snap.rs", "        Self { targets }", "        Self { targets, leo: false }")
edit("guides/snap.rs", "PointSnap { targets: Some((targets, cx.snapping_tolerance)) }",
     "PointSnap { targets: Some((targets, cx.snapping_tolerance)), leo: false }")
edit("guides/snap.rs", "impl PointSnap {", """impl PointSnap {
    pub(crate) fn leo_for_scale(cx: &ToolContext) -> Self {
        if !cx.leo_snapping || cx.snap_to_grid { return Self::new(cx, || Targets::for_scale(cx)); }
        Self { targets: (cx.smart_guides || cx.snap_to_point).then(|| (Targets::leo_scale_targets(cx), 8.0)), leo: true }
    }

    /// Dedicated node-editing target set: do not let bounds, alignments, grids or pixel
    /// snapping override the new true-geometry target detection.
    pub(crate) fn leo_for_anchor_drag(cx: &ToolContext) -> Self {
        if !cx.leo_snapping || cx.snap_to_grid { return Self::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection)); }
        let targets = (cx.smart_guides || cx.snap_to_point).then(|| {
            (Targets::leo_for_anchor_drag(cx), 8.0)
        });
        Self { targets, leo: true }
    }

    pub(crate) fn uses_leo(&self) -> bool { self.leo }

    pub(crate) fn leo_captured(&self) -> bool {
        self.targets.as_ref().is_some_and(|(targets, _)| targets.leo_snap.get().is_some() || targets.leo_extension.get().is_some() || targets.leo_intersection.get().is_some())
    }

    pub(crate) fn direct_anchor_drag(&self, cx: &ToolContext, p: Point, from: Option<&Leave>, shift: bool) -> (Point, Vec<Overlay>) {
        if !self.leo { return self.snap_from(cx, p, from); }
        let constrained = if shift { from.and_then(|f| f.constrained(p)) } else { None };
        let p = constrained.unwrap_or(p);
        if let Some((targets, _)) = &self.targets {
            if cx.smart_guides {
                if let Some(hit)=targets.leo_snap_path_intersection(p,from,shift,cx.tol(8.0),cx.tol(12.8)) {return hit;}
            }
            if !shift && let Some(f) = from.filter(|f| f.contact)
                && let Some(hit) = targets.leo_snap_contact(p, f.at, cx.tol(8.0)) {
                targets.leo_extension.set(None);
                targets.leo_intersection.set(None);
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
edit("guides/snap.rs", "        let step = if cx.snap_to_pixel {", """        if self.leo && let Some((targets, _)) = &self.targets
            && let Some(hit) = targets.leo_resize(cx, bx, handle, a, proportional, from_center)
        {
            return hit;
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
     "        let guides = cx.smart_guides && !cx.snap_to_grid;\n        let points = |exclude: &[NodeId]| if cx.snap_to_grid { None } else { Targets::snap_to_point(cx, exclude) };",
     """        let leo = cx.leo_snapping && !cx.snap_to_grid && (cx.smart_guides || cx.snap_to_point);
        let guides = !leo && cx.smart_guides && !cx.snap_to_grid;
        let points = |exclude: &[NodeId]| if leo || cx.snap_to_grid { None } else { Targets::snap_to_point(cx, exclude) };""")
edit("select.rs",
     "            rulers: (guides && cx.snap_to_point).then(|| Targets::rulers(cx)),",
     "            rulers: (guides && cx.snap_to_point).then(|| Targets::rulers(cx)),\n            leo: leo.then(|| ObjectMoveSnap::new(cx, selection_bounds(cx))),")
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
        if !cx.leo_snapping || cx.snap_to_grid { return Self::new(cx, || Targets::for_anchor_drag(cx.doc, &vectorcraft_doc::Selection::default())); }
        let targets = (cx.smart_guides || cx.snap_to_point).then(|| {
            (Targets::leo_for_object_move(cx, &[]), 8.0)
        });
        Self { targets, leo: true }
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
            }''', '''            if let Some(grid) = &self.partial_grid && !snap.uses_leo() {
                (d, self.guides) = if ev.mods.shift { grid.snap_along(cx, held, d, false) } else { grid.snap(cx, held, d, false) };
            } else {
                let shift = ev.mods.shift.then(|| Leave::segment(cx, held, true));
                let (q, guides) = snap.direct_anchor_drag(cx, held + (p - start), shift.as_ref(), ev.mods.shift);
                (d, self.guides) = (q - held, guides);
                if !snap.leo_captured() && let Some(grid) = &self.partial_grid {
                    let raw = move_delta(start, p, ev.mods.shift, cx.constrain_angle);
                    (d, self.guides) = if ev.mods.shift { grid.snap_along(cx, held, raw, false) } else { grid.snap(cx, held, raw, false) };
                }
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
            if cx.leo_snapping && (already || cx.selection.anchors.get(&id).is_some_and(|anchors| anchors.contains(&(si, ai)))) {
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
# Give the extension cue a dark backing so yellow remains legible on a white page.
ui_path=Path(sys.argv[1]) / "crates/ui-egui/src/canvas/overlays.rs"
ui_source=ui_path.read_text(encoding="utf-8")
ui_anchor='                let sp = xf.to_screen(*pt) + vec2(8.0, -14.0);'
if ui_source.count(ui_anchor)!=1: raise SystemExit("FAIL extension label background anchor")
ui_source=ui_source.replace(ui_anchor,ui_anchor+'\n                if text == "延长线吸附" || text.starts_with("旋转吸附") {\n                    let galley=p.layout_no_wrap(text.to_owned(),egui::FontId::proportional(11.0),c32(*color));\n                    let backdrop=egui::Rect::from_min_size(sp-vec2(3.0,2.0),galley.size()+vec2(6.0,4.0));\n                    p.rect_filled(backdrop,CornerRadius::same(3),Color32::from_rgba_unmultiplied(0x20,0x24,0x2b,235));\n                }\n')
ui_path.write_text(ui_source,encoding="utf-8",newline="\n")

# Keep the upstream assertions in native mode. Exercise Leo through its existing
# geometry, tool and session regression fixtures with the preference enabled.
for source, destination in (
    ("geometry_regressions.rs", "guides/leo_snapping_engine.rs"),
    ("rotation_regressions.rs", "guides/leo_snapping_engine.rs"),
    ("direct_regressions.rs", "direct/tests.rs"),
    ("select_regressions.rs", "select/tests.rs"),
    ("rotation_tool_regressions.rs", "xform/transform.rs"),
):
    source_text = (legacy / source).read_text(encoding="utf-8")
    if source in {"direct_regressions.rs", "select_regressions.rs", "rotation_tool_regressions.rs"}:
        source_text = re.sub(r"\bcx\(", "crate::testutil::leo_cx(", source_text)
        source_text = source_text.replace("use crate::testutil::{cx,paint};", "use crate::testutil::paint;")
    with (root / destination).open("a", encoding="utf-8") as file:
        file.write("\n" + source_text)

with (upstream / "crates/engine/src/tests/clipboard.rs").open("a", encoding="utf-8") as file:
    file.write("\n" + (legacy / "session_regressions.rs").read_text(encoding="utf-8"))

# Check that turning the preference off goes through native tolerance, while
# Leo captures the same target and marks its actual location at six pixels.
with (root / "guides/snap.rs").open("a", encoding="utf-8") as file:
    file.write("""
#[cfg(test)]
mod leo_snapping_mode_tests {
    use super::*;
    use crate::testutil::{cx, paint};
    use vectorcraft_doc::{Appearance, Document, Node, Selection};
    use vectorcraft_geom::shapes;

    #[test]
    fn leo_snapping_engine_mode_selects_one_anchor_drag_route() {
        let mut doc = Document::new(500.0, 500.0);
        let layer = doc.layers[0].id;
        let id = doc.alloc_id();
        let target = Point::new(150.0, 180.0);
        doc.insert(Some(layer), 0, Node::path(
            id, shapes::line(target, Point::new(220.0, 180.0)), Appearance::default_art(),
        )).unwrap();
        let selection = Selection::default();
        let paint = paint();
        let native = ToolContext {
            snap_to_point: false,
            snapping_tolerance: 2.0,
            ..cx(&doc, &selection, &paint)
        };
        let raw = Point::new(153.0, 175.0);
        let (point, guides) = PointSnap::leo_for_anchor_drag(&native).direct_anchor_drag(&native, raw, None, false);
        assert_eq!(point, raw);
        assert!(!guides.iter().any(|guide| matches!(guide, Overlay::CurvePoint { p, .. } if *p == target)));
        let leo = ToolContext { leo_snapping: true, ..native };
        let (point, guides) = PointSnap::leo_for_anchor_drag(&leo).direct_anchor_drag(&leo, raw, None, false);
        assert_eq!(point, target);
        assert!(guides.iter().any(|guide| matches!(guide, Overlay::CurvePoint { p, .. } if *p == target)));
    }
}
""")

print("Applied switchable Leo Design snapping engine")

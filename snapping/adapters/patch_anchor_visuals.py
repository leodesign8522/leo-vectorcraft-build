#!/usr/bin/env python3
"""LC-026: page-axis Shift snapping and high-contrast anchor-extension guides."""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_anchor_visuals.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def change(rel, old, new, title):
    path = root / rel
    source = path.read_text(encoding="utf-8")
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"FAIL LC-026 {title}: {rel}, anchor count={count}")
    path.write_text(source.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("PASS LC-026", title)

snap = "crates/tools/src/guides/snap.rs"
change(snap,
       "    /// A curved source cannot make a line-to-line perpendicular or line-to-curve tangent contact.\n",
       """    /// A direct-selected anchor follows page axes while Shift is held: horizontal, vertical\n    /// and 45-degree diagonals. This is independent of General > Constrain Angle, which may be\n    /// rotated for drawing tools; Smart Guides construction-angle preferences still apply.\n    pub fn segment_page_axes(cx: &ToolContext, at: Point, shift: bool) -> Self {\n        Self {\n            at,\n            shift: shift.then_some(Keep::Angle { step: 45.0, base: 0.0 }),\n            angles: if cx.smart_guides { cx.construction_angles } else { &[] },\n            contact: true,\n        }\n    }\n\n    /// A curved source cannot make a line-to-line perpendicular or line-to-curve tangent contact.\n""",
       "add fixed page-axis Shift constraint for direct anchor movement")

direct = "crates/tools/src/direct.rs"
change(direct,
       "        let mut d = move_delta(start, p, ev.mods.shift, cx.constrain_angle);\n        let unsnapped = d;\n        // The grabbed anchor snaps and the others follow it. Shift keeps the move at its\n",
       "        let mut d = move_delta(start, p, ev.mods.shift, 0.0);\n        let unsnapped = d;\n        // The grabbed anchor snaps and the others follow it. Shift keeps the move at its\n",
       "make direct-anchor movement use absolute page axes")
change(direct,
       "            let shift = ev.mods.shift.then(|| Leave::segment(cx, grab, true));\n            let (q, guides) = snap.snap_from(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()));",
       "            let shift = ev.mods.shift.then(|| Leave::segment_page_axes(cx, grab, true));\n            let (q, guides) = snap.snap_from(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()));",
       "route direct-anchor Shift snap through page-axis constraint")
change(direct,
       "        let mut d = move_delta(start, p, ev.mods.shift, cx.constrain_angle);\n        let unsnapped = d;\n        self.guides.clear();\n        if !ev.mods.alt\n            && let Some(snap) = &self.partial_snap",
       "        let mut d = if self.partial_snap.is_some() && ev.mods.shift { move_delta(start, p, true, 0.0) } else { move_delta(start, p, ev.mods.shift, cx.constrain_angle) };\n        let unsnapped = d;\n        self.guides.clear();\n        if !ev.mods.alt\n            && let Some(snap) = &self.partial_snap",
       "keep partial-anchor snapped state consistent with page-axis movement")
change(direct,
       "                let shift = ev.mods.shift.then(|| Leave::segment(cx, held, true));\n                let (q, guides) = snap.snap_from(cx, held + (p - start), shift.as_ref());",
       "                let shift = ev.mods.shift.then(|| Leave::segment_page_axes(cx, held, true));\n                let (q, guides) = snap.snap_from(cx, held + (p - start), shift.as_ref());",
       "apply page-axis Shift constraint to partial anchor selections")

targets = "crates/tools/src/guides/targets.rs"
change(targets,
       "const BUDGET: usize = 20_000;\n",
       "const BUDGET: usize = 20_000;\n\nconst EXTENSION_PREVIEW_COLOR: [u8; 3] = [0x35, 0xd7, 0xff];\nconst EXTENSION_SNAP_COLOR: [u8; 3] = [0xff, 0xb3, 0x3d];\n",
       "add distinct extension-preview and extension-snap colours")
change(targets,
       "    pub(super) extensions: Vec<(Point, Vec2)>,",
       "    pub(super) extensions: Vec<(Point, Point, Vec2)>,",
       "remember both fixed and dragged ends of source segments")
change(targets,
       """    fn push_extensions(&mut self, sp: &SubPath, moving: impl Fn(usize) -> bool) {\n        let n = sp.anchors.len();\n        for seg in (0..sp.segment_count()).filter(|&seg| moving(seg) != moving((seg + 1) % n) && sp.segment_is_line(seg)) {\n            let c = sp.segment(seg);\n            let (fixed, dragged) = if moving(seg) { (c.p3, c.p0) } else { (c.p0, c.p3) };\n            let d = dragged - fixed;\n            let len = d.hypot();\n            if len.is_finite() && len > 1e-9 {\n                self.extensions.push((fixed, d / len));\n            }\n        }\n    }""",
       """    fn push_extensions(&mut self, sp: &SubPath, moving: impl Fn(usize) -> bool) {\n        let n = sp.anchors.len();\n        for seg in (0..sp.segment_count()).filter(|&seg| moving(seg) != moving((seg + 1) % n) && sp.segment_is_line(seg)) {\n            let c = sp.segment(seg);\n            let (fixed, dragged) = if moving(seg) { (c.p3, c.p0) } else { (c.p0, c.p3) };\n            let d = dragged - fixed;\n            let len = d.hypot();\n            if len.is_finite() && len > 1e-9 {\n                self.extensions.push((fixed, dragged, d / len));\n            }\n        }\n    }""",
       "store the original moving endpoint as the bidirectional guide origin")

old_extension = """    /// The nearest forward ray of a straight segment ending at the dragged anchor.\n    fn extension(&self, p: Point, tol: f64) -> Option<(Point, Vec2, Point)> {\n        self.extensions\n            .iter()\n            .filter(|(at, _)| at.distance(p) > tol)\n            .map(|&(at, d)| (at, d, at + d * (p - at).dot(d)))\n            .map(|(at, d, q)| (at, d, q, q.distance(p)))\n            .filter(|(at, d, q, dist)| *dist <= tol && (*q - *at).dot(*d) > 0.0)\n            .min_by(|a, b| a.3.total_cmp(&b.3))\n            .map(|(at, d, q, _)| (at, d, q))\n    }"""
new_extension = """    /// The original straight segment direction, usable on both sides of the dragged anchor.\n    /// Preserve collinearity while lengthening, shortening, or crossing the fixed endpoint.\n    fn extension_candidate(&self, p: Point, min_dist: f64, max_dist: f64) -> Option<(f64, Point, Point, Vec2, Point)> {\n        self.extensions\n            .iter()\n            .filter_map(|&(at, end, dir)| {\n                let along = (p - at).dot(dir);\n                if !along.is_finite() {\n                    return None;\n                }\n                let q = at + dir * along;\n                let dist = q.distance(p);\n                (dist > min_dist && dist <= max_dist).then_some((dist, at, end, dir, q))\n            })\n            .min_by(|a, b| a.0.total_cmp(&b.0))\n    }\n\n    /// A real snap candidate, within the ordinary Smart Guides tolerance.\n    fn extension(&self, p: Point, tol: f64) -> Option<(Point, Point, Vec2, Point)> {\n        self.extension_candidate(p, -1e-9, tol).map(|(_, at, end, dir, q)| (at, end, dir, q))\n    }\n\n    /// A preview candidate: close enough to show the ray, but not yet close enough to snap.\n    fn extension_preview(&self, p: Point, tol: f64) -> Option<(Point, Point, Vec2)> {\n        self.extension_candidate(p, tol, tol * 3.0).map(|(_, _, end, dir, q)| {\n            let preview_dir = if (q - end).dot(dir) < 0.0 { -dir } else { dir };\n            (end, q, preview_dir)\n        })\n    }"""
change(targets, old_extension, new_extension, "detect the source direction in both drag directions")

change(targets,
       "    pub fn snap_from(&self, p: Point, from: Option<&Leave>, tol: f64) -> (Point, Vec<Overlay>) {\n",
       """    pub fn snap_from(&self, p: Point, from: Option<&Leave>, tol: f64) -> (Point, Vec<Overlay>) {\n        let preview = self.extension_preview(p, tol);\n        let (q, mut overlays) = self.snap_from_inner(p, from, tol);\n        if let Some((end, projected, dir)) = preview {\n            let length = (projected - end).dot(dir).max(0.0);\n            let extra = (tol * 1.5 - length).max(tol * 0.5);\n            overlays.push(Overlay::Line {\n                a: end,\n                b: projected + dir * extra,\n                color: EXTENSION_PREVIEW_COLOR,\n                dashed: true,\n            });\n        }\n        (q, overlays)\n    }\n\n    fn snap_from_inner(&self, p: Point, from: Option<&Leave>, tol: f64) -> (Point, Vec<Overlay>) {\n""",
       "show a dashed preview without disabling normal snapping")

old_branch = """        // Prefer the source segment's forward extension to unrelated nearby targets.\n        if let Some((at, dir, q)) = self.extension(p, tol) {\n            return match self.slide(at, dir, q, tol, true) {\n                Some((hit, target, kind)) => {\n                    let mut guides: Vec<Overlay> = self.line(at, hit).into_iter().collect();\n                    if kind == \"intersect\" && self.nearest_point(hit, 1e-9).is_none() {\n                        guides.extend(self.label(hit, \"intersect\"));\n                    } else {\n                        guides.extend(self.lined_up(hit, target));\n                    }\n                    (hit, guides)\n                }\n                None => (q, self.line(at, q).into_iter().collect()),\n            };\n        }"""
new_branch = """        // A source-line extension has its own colour, dashed ray and hit marker so it is\n        // visually distinct from ordinary magenta alignment guides.\n        if let Some((at, end, dir, q)) = self.extension(p, tol) {\n            let (hit, target, kind) = self.slide(at, dir, q, tol, true).unwrap_or((q, q, \"extension\"));\n            let mut guides = vec![\n                Overlay::Line { a: end, b: hit, color: EXTENSION_SNAP_COLOR, dashed: true },\n                Overlay::CurvePoint { p: hit, color: EXTENSION_SNAP_COLOR, filled: true, size: 8.0 },\n                Overlay::Label { p: hit, text: \"延长线吸附\".into(), color: EXTENSION_SNAP_COLOR },\n            ];\n            if kind == \"intersect\" && self.nearest_point(hit, 1e-9).is_none() {\n                guides.extend(self.label(hit, \"intersect\"));\n            } else if kind == \"align\" {\n                guides.extend(self.lined_up(hit, target));\n            }\n            return (hit, guides);\n        }"""
change(targets, old_branch, new_branch, "render a dedicated amber extension-snap line, marker and label")

change(targets,
       "targets.extensions.push((Point::new(0.0, 0.0), Vec2::new(0.0, 1.0)));",
       "targets.extensions.push((Point::new(0.0, 0.0), Point::new(0.0, 5.0), Vec2::new(0.0, 1.0)));",
       "migrate vertical extension fixture")
change(targets,
       "targets.extensions.push((Point::new(0.0, 0.0), Vec2::new(1.0, 0.0)));",
       "targets.extensions.push((Point::new(0.0, 0.0), Point::new(5.0, 0.0), Vec2::new(1.0, 0.0)));",
       "migrate horizontal extension fixture")
change(targets,
       "        assert!(targets.extension(Point::new(0.2, -10.0), 0.5).is_none());",
       "        assert!(targets.extension(Point::new(0.2, -10.0), 0.5).is_some());\n        assert!(targets.extension(Point::new(0.2, 4.9), 0.5).is_some());",
       "accept shortening and reverse extension")
change(targets,
       "        targets.extensions.push((at, dir));",
       "        targets.extensions.push((at, Point::new(3.0, 4.0), dir));",
       "migrate arbitrary-angle extension fixture")

with (root / "crates/tools/src/direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_shift_drag_vertical_snaps_to_horizontal_edge_with_page_axes() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target,
        shapes::line(Point::new(350.0, 400.0), Point::new(410.0, 400.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let mut c = cx(&d, &selected, &p);
    // A rotated global Constrain Angle must not remove the page's vertical Shift direction.
    c.constrain_angle = 30.0;
    let shift = Mods { shift: true, ..Mods::default() };
    let mut t = DirectSelectionTool::new(false);
    t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0).with_mods(shift));
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 380.0, 403.5).with_mods(shift));
    let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
    assert_eq!(cmd, "path.moveAnchors");
    assert!(v["dx"].as_f64().unwrap().abs() < 1e-8, "{v}");
    assert!((v["dy"].as_f64().unwrap() - 60.0).abs() < 1e-5, "{v}");
}

#[test]
fn leo_shift_drag_45_degree_direction_snaps_to_target_edge() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target,
        shapes::line(Point::new(405.0, 340.0), Point::new(405.0, 390.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let c = cx(&d, &selected, &p);
    let shift = Mods { shift: true, ..Mods::default() };
    let mut t = DirectSelectionTool::new(false);
    t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0).with_mods(shift));
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 403.5, 363.5).with_mods(shift));
    let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
    assert_eq!(cmd, "path.moveAnchors");
    assert!((v["dx"].as_f64().unwrap() - 25.0).abs() < 1e-5, "{v}");
    assert!((v["dy"].as_f64().unwrap() - 25.0).abs() < 1e-5, "{v}");
}
""")

with (root / "crates/tools/src/guides/targets.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[cfg(test)]
mod leo_anchor_extension_visual_tests {
    use super::*;

    #[test]
    fn extension_preview_is_cyan_dashed_and_starts_at_dragged_endpoint() {
        let mut targets = Targets::default();
        let at = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0);
        let dir = Vec2::new(1.0, 0.0);
        targets.extensions.push((at, end, dir));
        let p = Point::new(10.0, 2.0);
        let (hit, overlays) = targets.snap_from(p, None, 1.0);
        assert_eq!(hit, p);
        assert!(overlays.iter().any(|overlay| matches!(overlay,
            Overlay::Line { a, b, color, dashed: true }
                if *a == end && *color == EXTENSION_PREVIEW_COLOR && b.x > 10.0 && b.y == 0.0
        )), "{overlays:?}");
    }

    #[test]
    fn extension_snap_uses_amber_line_marker_and_label() {
        let mut targets = Targets::default();
        let at = Point::new(0.0, 0.0);
        let end = Point::new(5.0, 0.0);
        let dir = Vec2::new(1.0, 0.0);
        targets.extensions.push((at, end, dir));
        let (hit, overlays) = targets.snap_from(Point::new(10.0, 0.5), None, 1.0);
        assert!(hit.distance(Point::new(10.0, 0.0)) < 1e-9, "{hit:?}");
        assert!(overlays.iter().any(|overlay| matches!(overlay,
            Overlay::Line { a, b, color, dashed: true }
                if *a == end && *b == hit && *color == EXTENSION_SNAP_COLOR
        )), "{overlays:?}");
        assert!(overlays.iter().any(|overlay| matches!(overlay,
            Overlay::CurvePoint { p, color, filled: true, .. }
                if *p == hit && *color == EXTENSION_SNAP_COLOR
        )), "{overlays:?}");
        assert!(overlays.iter().any(|overlay| matches!(overlay,
            Overlay::Label { text, color, .. }
                if text == "延长线吸附" && *color == EXTENSION_SNAP_COLOR
        )), "{overlays:?}");
    }
}
""")

print("SUCCESS LC-026 page-axis Shift snapping and extension guide feedback")


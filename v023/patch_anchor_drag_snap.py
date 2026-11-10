#!/usr/bin/env python3
"""LC-024: Shift anchor drag and arbitrary-angle extension intersections."""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_anchor_drag_snap.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def change(rel, old, new, title):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    n = s.count(old)
    if n != 1:
        raise SystemExit(f"FAIL LC-024 {title}: {rel}, anchor count={n}")
    p.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("OK LC-024", title)

direct = "crates/tools/src/direct.rs"
change(direct,
       "    click: Option<(NodeId, usize, usize)>,\n    /// A click on a whole path picks the segment on release;",
       "    click: Option<(NodeId, usize, usize)>,\n"
       "    /// A Shift-click on a selected anchor toggles only on release.\n"
       "    shift_deselect: Option<(NodeId, usize, usize)>,\n"
       "    /// A click on a whole path picks the segment on release;",
       "add pending Shift-click state")
change(direct, "            click: None,\n            click_segment: None,",
       "            click: None,\n            shift_deselect: None,\n            click_segment: None,", "initialize Shift-click state")
change(direct,
       "        (self.click, self.click_segment, self.click_group, self.frame_click) = (None, None, None, None);",
       "        (self.click, self.click_segment, self.click_group, self.frame_click) = (None, None, None, None);\n        self.shift_deselect = None;",
       "clear Shift-click state on cancel")
change(direct,
       "    fn press_anchor(&mut self, cx: &ToolContext, ev: &PointerEvent, p: Point, (id, si, ai, grab): (NodeId, usize, usize, Point)) -> Vec<Action> {\n",
       "    fn press_anchor(&mut self, cx: &ToolContext, ev: &PointerEvent, p: Point, (id, si, ai, grab): (NodeId, usize, usize, Point)) -> Vec<Action> {\n        self.shift_deselect = None;\n",
       "reset Shift-click state on press")
change(direct,
       '        if ev.mods.shift {\n            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": "toggle"}))];\n        }\n',
       '        if ev.mods.shift {\n'
       '            // Keep an already-selected anchor selected while the pointer is dragged.\n'
       '            // A Shift-click toggles it off only when released without moving.\n'
       '            if cx.selection.anchors.get(&id).is_some_and(|anchors| anchors.contains(&(si, ai))) {\n'
       '                self.shift_deselect = Some((id, si, ai));\n'
       '                return vec![];\n'
       '            }\n'
       '            let mode = if already { "toggle" } else { "add" };\n'
       '            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": mode}))];\n'
       '        }\n',
       "preserve selected anchor during Shift-drag")
change(direct,
       "        let group = self.click_group.take();\n        // A click on one of several selected frame anchors selects it alone.",
       "        let group = self.click_group.take();\n"
       "        let shift_deselect = self.shift_deselect.take();\n"
       "        if !began && let Some((id, si, ai)) = shift_deselect {\n"
       '            return vec![Action::Exec("select.anchors".into(), json!({"id": id.0, "anchors": [[si, ai]], "mode": "toggle"}))];\n'
       "        }\n"
       "        // A click on one of several selected frame anchors selects it alone.",
       "toggle selected anchor only on click release")

targets = "crates/tools/src/guides/targets.rs"
change(targets,
       "            .filter(|(at, .., dist)| *dist <= tol && (p.x - at.x).abs().min((p.y - at.y).abs()) > *dist)\n",
       "            .filter(|(at, d, q, dist)| *dist <= tol && (*q - *at).dot(*d) > 0.0)\n",
       "recognize horizontal, vertical and oblique extensions")
change(targets,
       "        if let Some(f) = from\n"
       "            && let Some(q) = f.constrained(p)\n"
       "        {\n"
       "            return match self.slide(f.at, q - f.at, q, tol, true) {\n"
       "                Some((q, target)) => (q, self.lined_up(q, target)),\n"
       "                None => (q, vec![]),\n"
       "            };\n"
       "        }\n"
       "        if let Some((q, k)) = self.nearest_point(p, tol) {",
       "        if let Some(f) = from\n"
       "            && let Some(q) = f.constrained(p)\n"
       "        {\n"
       "            return match self.slide(f.at, q - f.at, q, tol, true) {\n"
       "                Some((hit, target, kind)) => {\n"
       "                    let mut guides: Vec<Overlay> = self.line(f.at, hit).into_iter().collect();\n"
       "                    if kind == \"intersect\" && self.nearest_point(hit, 1e-9).is_none() {\n"
       "                        guides.extend(self.label(hit, \"intersect\"));\n"
       "                    } else {\n"
       "                        guides.extend(self.lined_up(hit, target));\n"
       "                    }\n"
       "                    (hit, guides)\n"
       "                }\n"
       "                None => (q, self.line(f.at, q).into_iter().collect()),\n"
       "            };\n"
       "        }\n"
       "        // Prefer the source segment's forward extension to unrelated nearby targets.\n"
       "        if let Some((at, dir, q)) = self.extension(p, tol) {\n"
       "            return match self.slide(at, dir, q, tol, true) {\n"
       "                Some((hit, target, kind)) => {\n"
       "                    let mut guides: Vec<Overlay> = self.line(at, hit).into_iter().collect();\n"
       "                    if kind == \"intersect\" && self.nearest_point(hit, 1e-9).is_none() {\n"
       "                        guides.extend(self.label(hit, \"intersect\"));\n"
       "                    } else {\n"
       "                        guides.extend(self.lined_up(hit, target));\n"
       "                    }\n"
       "                    (hit, guides)\n"
       "                }\n"
       "                None => (q, self.line(at, q).into_iter().collect()),\n"
       "            };\n"
       "        }\n"
       "        if let Some((q, k)) = self.nearest_point(p, tol) {",
       "prioritize locked directions and real edge intersections")
change(targets,
       "            .or_else(|| self.extension(p, tol))\n            .or_else(|| self.on_angled(p, tol))",
       "            .or_else(|| self.on_angled(p, tol))",
       "remove late extension fallback")
change(targets,
       "    fn slide(&self, at: Point, dir: Vec2, q: Point, tol: f64, ray: bool) -> Option<(Point, Point)> {",
       "    fn slide(&self, at: Point, dir: Vec2, q: Point, tol: f64, ray: bool) -> Option<(Point, Point, &'static str)> {",
       "extend slide results with snap kind")
change(targets, "        let mut best: Option<(f64, Point, Point)> = None;",
       "        let mut best: Option<(f64, Point, Point, &'static str)> = None;", "track snap kind")
change(targets, "                    best = Some((moved, hit, *from));",
       "                    best = Some((moved, hit, *from, \"align\"));", "tag axis snaps")
change(targets, "                best = Some((moved, hit, on));",
       "                best = Some((moved, hit, on, \"align\"));", "tag angled guide snaps")
change(targets,
       "        best.map(|(_, hit, from)| (hit, from))",
       "        // Also intersect the constrained line with actual target paths.\n"
       "        let unit = dir * (1.0 / dir.hypot());\n"
        "        let extent = (q - at).hypot() + reach;\n"
        "        // kurbo::Line is finite: span both sides far enough to include q and its snap radius.\n"
        "        let line = Line::new(at - unit * extent, at + unit * extent);\n"
       "        for (bounds, segment) in self.segments.iter().filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q)) {\n"
       "            for intersection in segment.intersect_line(line) {\n"
       "                let hit = line.eval(intersection.line_t);\n"
       "                let along = (hit - at).dot(dir) / dir.hypot2();\n"
       "                let moved = hit.distance(q);\n"
       "                if moved <= reach && (!ray || along > 0.0) && best.is_none_or(|b| moved < b.0) {\n"
       "                    best = Some((moved, hit, hit, \"intersect\"));\n"
       "                }\n"
       "            }\n"
       "        }\n"
       "        best.map(|(_, hit, from, kind)| (hit, from, kind))",
       "intersect locked direction with target path edges")

with (root / "crates/tools/src/direct/tests.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[test]
fn leo_shift_drag_keeps_a_selected_anchor_and_constrains_its_move() {
    let (d, id) = doc_with_rect();
    let selected = anchors_of(id, &[(0, 0), (0, 1)]);
    let p = paint();
    let c = cx(&d, &selected, &p);
    let shift = Mods { shift: true, ..Mods::default() };
    let mut t = DirectSelectionTool::new(false);
    assert!(t.pointer(&c, &PointerEvent::new(PointerKind::Down, 100.0, 100.0).with_mods(shift)).is_empty());
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 110.0, 110.0).with_mods(shift));
    assert!(matches!(moved.as_slice(), [Action::Begin(_), Action::Preview(cmd, v)] if cmd == "path.moveAnchors" && v["dx"] == 10.0 && v["dy"] == 10.0), "{moved:?}");
    assert_eq!(t.pointer(&c, &PointerEvent::new(PointerKind::Up, 110.0, 110.0).with_mods(shift)), vec![Action::Commit]);
}

#[test]
fn leo_shift_drag_snaps_to_an_oblique_target_edge_intersection() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source, shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)), vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target, shapes::line(Point::new(390.0, 300.0), Point::new(410.0, 360.0)), vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let c = cx(&d, &selected, &p);
    let shift = Mods { shift: true, ..Mods::default() };
    let mut t = DirectSelectionTool::new(false);
    t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0).with_mods(shift));
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 403.5, 340.0).with_mods(shift));
    let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
    assert_eq!(cmd, "path.moveAnchors");
    assert!((v["dx"].as_f64().unwrap() - 23.3333333333).abs() < 0.01, "{v}");
    assert!(v["dy"].as_f64().unwrap().abs() < 1e-9, "{v}");
}

#[test]
fn leo_arbitrary_angle_extension_snaps_to_an_oblique_target_edge_intersection() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source, shapes::line(Point::new(300.0, 400.0), Point::new(380.0, 340.0)), vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target, shapes::line(Point::new(400.0, 300.0), Point::new(400.0, 350.0)), vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let c = cx(&d, &selected, &p);
    let mut t = DirectSelectionTool::new(false);
    t.pointer(&c, &PointerEvent::new(PointerKind::Down, 380.0, 340.0));
    let moved = t.pointer(&c, &PointerEvent::new(PointerKind::Drag, 399.0, 326.0));
    let Some(Action::Preview(cmd, v)) = moved.last() else { panic!("{moved:?}") };
    assert_eq!(cmd, "path.moveAnchors");
    assert!((v["dx"].as_f64().unwrap() - 20.0).abs() < 1e-6, "{v}");
    assert!((v["dy"].as_f64().unwrap() + 15.0).abs() < 1e-6, "{v}");
}
""")

with (root / "crates/tools/src/guides/targets.rs").open("a", encoding="utf-8") as f:
    f.write("""
#[cfg(test)]
mod leo_anchor_extension_intersection_tests {
    use super::*;

    #[test]
    fn extension_detection_accepts_horizontal_and_vertical_rays() {
        let mut targets = Targets::default();
        targets.extensions.push((Point::new(0.0, 0.0), Vec2::new(0.0, 1.0)));
        assert!(targets.extension(Point::new(0.2, 10.0), 0.5).is_some());
        assert!(targets.extension(Point::new(0.2, -10.0), 0.5).is_none());
        targets.extensions.clear();
        targets.extensions.push((Point::new(0.0, 0.0), Vec2::new(1.0, 0.0)));
        assert!(targets.extension(Point::new(10.0, 0.2), 0.5).is_some());
    }

    #[test]
    fn arbitrary_angle_extension_locks_to_actual_target_edge_intersection() {
        let mut targets = Targets::default();
        let at = Point::new(0.0, 0.0);
        let dir = Vec2::new(0.6, 0.8);
        let a = Point::new(5.0, 0.0);
        let b = Point::new(5.0, 20.0);
        targets.extensions.push((at, dir));
        targets.segments.push((Rect::from_points(a, b), PathSeg::Line(Line::new(a, b))));
        let (hit, overlays) = targets.snap_from(Point::new(4.8, 6.4), None, 1.0);
        assert!(hit.distance(Point::new(5.0, 6.6666666667)) < 1e-6, "{hit:?}");
        assert!(!overlays.is_empty(), "{overlays:?}");
    }
}
""")

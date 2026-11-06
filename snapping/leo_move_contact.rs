//! Shape-based Smart Guides for moving editable text and outlined artwork.
use super::*;

impl Targets {
    pub(super) fn text_outline(doc: &Document, node: &Node) -> Option<BezPath> {
        let NodeKind::Text(text) = &node.kind else { return None };
        // Threaded frames use the story layout, not each frame's stored text alone.
        if doc.text_threads.iter().any(|thread| thread.contains(&node.id)) {
            return None;
        }
        let mut outline = layout_text(FontDb::global(), text).to_bezpath();
        outline.apply_affine(text.xf);
        (!outline.elements().is_empty() && outline.bounding_box().is_finite()).then_some(outline)
    }

    pub(crate) fn text_ink_bounds(doc: &Document, node: &Node) -> Option<Rect> {
        Self::text_outline(doc, node).map(|outline| outline.bounding_box())
    }

    pub(crate) fn hold_text_outline(doc: &Document, node: &Node, p: Point, tol: f64) -> Option<Point> {
        let outline = Self::text_outline(doc, node)?;
        let anchor = outline.segments().map(|seg| seg.start())
            .filter(|q| q.distance(p) <= tol * 0.5)
            .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
        anchor.or_else(|| {
            outline.segments()
                .filter(|seg| seg.bounding_box().inflate(tol, tol).contains(p))
                .map(|seg| (seg, seg.nearest(p, 1e-6)))
                .filter(|(_, hit)| hit.distance_sq <= tol * tol)
                .min_by(|(_, a), (_, b)| a.distance_sq.total_cmp(&b.distance_sq))
                .map(|(seg, hit)| seg.eval(hit.t))
        })
    }

    pub(in crate::guides) fn compact_move_hints(mut self, cx: &ToolContext) -> Self {
        self.compact_hint = Some(cx.tol(36.0));
        self
    }

    pub(super) fn contact_hint(&self, at: Point, kind: &str) -> Vec<Overlay> {
        let marker = (self.compact_hint.is_some() && self.style.lines).then(|| self.contact_mark(at, kind));
        marker.into_iter().chain(self.label(at, kind)).collect()
    }

    fn contact_mark(&self, p: Point, kind: &str) -> Overlay {
        Overlay::CurvePoint { p, color: self.style.color_for_label(kind), filled: true, size: 6.0 }
    }

    pub(crate) fn move_contact(&self, p: Point, tol: f64, along: Option<Vec2>) -> Option<(Point, Vec<Overlay>)> {
        let point = self.nearest_point(p, tol);
        if let Some((q, kind)) = point
            && along.is_none_or(|dir| (q - p).cross(dir).abs() <= 1e-8)
        {
            let mut guides: Vec<_> = self.label(q, kind.label()).into_iter().collect();
            if kind == Kind::Anchor && self.style.lines {
                guides.push(self.contact_mark(q, kind.label()));
            }
            return Some((q, guides));
        }

        let hit = if let Some(dir) = along {
            let trace = Line::new(p - dir * tol, p + dir * tol);
            self.segments.iter()
                .filter(|(bounds, _)| bounds.inflate(tol, tol).contains(p))
                .flat_map(|(_, seg)| seg.intersect_line(trace).into_iter().map(|hit| trace.eval(hit.line_t)))
                .filter(|q| q.distance(p) <= tol)
                .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
        } else {
            self.on_segment(p, tol)
        };
        if let Some(q) = hit {
            let kind = Kind::Edge.label();
            let mark = self.style.lines.then(|| self.contact_mark(q, kind));
            return Some((q, mark.into_iter().chain(self.label(q, kind)).collect()));
        }

        // Retain the existing projected-anchor behavior for constrained movement.
        point.zip(along).map(|((q, kind), dir)| {
            (p + dir * (q - p).dot(dir), self.label(q, kind.label()).into_iter().collect())
        })
    }
}

#[cfg(test)]
mod leo_move_contact_tests {
    use super::*;
    use crate::select::MoveSnap;
    use crate::testutil::{cx, doc_with_rect, paint};
    use vectorcraft_doc::{Appearance, CharStyle, TextObject};
    use vectorcraft_geom::shapes;

    fn text(doc: &mut Document, p: Point, value: &str, size: f64) -> NodeId {
        let id = doc.alloc_id();
        let layer = doc.layers[0].id;
        let t = TextObject::point(p, value, CharStyle { size, ..CharStyle::default() });
        doc.insert(Some(layer), 0, Node::new(id, NodeKind::Text(Box::new(t)))).unwrap();
        id
    }

    #[test]
    fn editable_text_move_aligns_actual_bottoms() {
        let (mut doc, _) = doc_with_rect();
        let left = text(&mut doc, Point::new(40.0, 280.0), "13", 60.0);
        let right = text(&mut doc, Point::new(250.0, 310.0), "HALL", 20.0);
        let a = Targets::text_ink_bounds(&doc, doc.node(left).unwrap()).unwrap();
        let b = Targets::text_ink_bounds(&doc, doc.node(right).unwrap()).unwrap();
        let sel = Selection { objects: vec![right], ..Selection::default() };
        let paint = paint();
        let cx = cx(&doc, &sel, &paint);
        let (delta, _) = MoveSnap::new(&cx).snap(
            &cx, b.center(), Vec2::new(0.0, a.y1 - b.y1 + 0.5), false,
        );
        assert!((b.y1 + delta.y - a.y1).abs() < 1e-8, "{delta:?}");
    }

    #[test]
    fn text_outline_and_whole_path_move_can_land_on_a_real_edge() {
        let (mut doc, _) = doc_with_rect();
        let text_id = text(&mut doc, Point::new(320.0, 350.0), "HALL", 24.0);
        let node = doc.node(text_id).unwrap();
        let start = Targets::text_outline(&doc, node).unwrap().segments().next().unwrap().start();
        let held = Targets::hold_text_outline(&doc, node, start, 1.0).unwrap();
        assert!(held.distance(start) < 1e-8);
        let target = Point::new(150.0, 200.0);
        let paint = paint();
        for id in [text_id, {
            let id = doc.alloc_id();
            let layer = doc.layers[0].id;
            doc.insert(Some(layer), 0, Node::path(id, shapes::line(start, start + Vec2::new(50.0, 0.0)), Appearance::default_art())).unwrap();
            id
        }] {
            let sel = Selection { objects: vec![id], ..Selection::default() };
            let cx = cx(&doc, &sel, &paint);
            let snap = MoveSnap::new(&cx);
            let (delta, guides) = snap.snap(&cx, held, target - held + Vec2::new(0.0, 1.0), false);
            assert!((held + delta).distance(target) < 1e-8);
            assert!(guides.iter().any(|guide| matches!(guide, Overlay::CurvePoint { p, .. } if p.distance(target) < 1e-8)));
            let anchor = Point::new(100.0, 200.0);
            let (delta, _) = snap.snap(&cx, held, anchor - held + Vec2::new(0.5, 1.0), false);
            assert!((held + delta).distance(anchor) < 1e-8);
        }
    }

    #[test]
    fn direct_anchor_can_land_on_the_edge_of_live_type() {
        let (mut doc, _) = doc_with_rect();
        let id = text(&mut doc, Point::new(250.0, 310.0), "H", 100.0);
        let outline = Targets::text_outline(&doc, doc.node(id).unwrap()).unwrap();
        let edge = outline.segments().find_map(|seg| match seg {
            PathSeg::Line(line) if (line.p1 - line.p0).hypot() > 30.0 => Some(line),
            _ => None,
        }).unwrap();
        let targets = Targets::for_anchor_drag(&doc, &Selection::default());
        let on_edge = edge.eval(0.5);
        let side = edge.p1 - edge.p0;
        let off_edge = on_edge + Vec2::new(-side.y, side.x) / side.hypot();
        let (q, _) = targets.snap_point(off_edge, 2.0);
        assert!(q.distance(on_edge) < 1e-8, "{q:?}");
    }

    #[test]
    fn direct_alignment_hint_is_local() {
        let targets = Targets { compact_hint: Some(36.0), ..Targets::default() };
        let end = Point::new(200.0, 100.0);
        assert!(matches!(
            targets.line(Point::new(0.0, 100.0), end),
            Some(Overlay::Line { a, b, .. }) if a == Point::new(164.0, 100.0) && b == end
        ));
    }
}

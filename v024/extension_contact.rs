#[derive(Clone, Copy)]
enum ExtensionTarget {
    Anchor,
    Edge(PathSeg, f64),
}

#[derive(Clone, Copy)]
pub(in crate::guides) struct LeoExtensionContact {
    line: LeoExtension,
    point: Point,
    target: ExtensionTarget,
}

impl Targets {
    /// Keep one supporting line during this drag. Alt extends its hold while
    /// still measuring target capture and release along the constrained line.
    pub(in crate::guides) fn leo_snap_extension_contact(
        &self, p: Point, acquire: f64, release: f64, alt: bool,
    ) -> Option<(Point, Vec<Overlay>)> {
        let held = self.leo_extension.get().filter(|line| {
            p.is_finite() && (alt || line.project(p, release).is_some())
        });
        if held.is_none() {
            self.leo_extension_contact.set(None);
        }
        let query = held.filter(|_| alt).map_or(p, |line| line.at + line.dir * (p - line.at).dot(line.dir));
        if let Some(hit) = self.leo_extension_contact.get()
            && Some(hit.line) == held && hit.point.distance(query) <= release
        {
            return Some((hit.point, self.extension_feedback(hit.line, hit.point, acquire, Some(hit.target))));
        }
        self.leo_extension_contact.set(None);
        if !query.is_finite() || !acquire.is_finite() || acquire <= 0.0 {
            return None;
        }

        // With no held line, only the dragged node's original straight edges
        // can acquire a compound hit. A previously displayed external
        // extension also works once it has been selected by the normal drag.
        if held.is_none() && (self.extensions.is_empty()
            || self.leo_drag_origin.is_some_and(|origin| origin.distance(p) <= acquire))
        {
            return None;
        }
        let local = self.leo_near(Rect::from_points(query, query).inflate(acquire, acquire));
        let geometry = local.as_ref().unwrap_or(self);
        let mut best = None;
        let lines = held.into_iter().chain(self.extensions.iter().filter_map(|&(at, dir)| {
            let line = LeoExtension { at, end: at, dir };
            (held.is_none() && line.project(p, acquire).is_some()).then_some(line)
        }));
        for line in lines {
            for &(point, kind) in &geometry.points {
                if kind == Kind::Anchor && point.distance(query) <= acquire && on_direction(point - line.at, line.dir) {
                    choose_extension_contact(&mut best, LeoExtensionContact { line, point, target: ExtensionTarget::Anchor }, query);
                }
            }
            for (point, segment, t) in guide_path_hits(geometry, query, line.at, line.dir, acquire) {
                choose_extension_contact(&mut best, LeoExtensionContact { line, point, target: ExtensionTarget::Edge(segment, t) }, query);
            }
        }
        if let Some(hit) = best {
            self.leo_extension.set(Some(hit.line));
            self.leo_extension_contact.set(Some(hit));
            self.leo_snap.set(None);
            self.leo_intersection.set(None);
            return Some((hit.point, self.extension_feedback(hit.line, hit.point, acquire, Some(hit.target))));
        }
        // A held direction stays in force while continuing past a released
        // target. Other geometry cannot silently pull the node off that line.
        let line = held?;
        let point = line.at + line.dir * (p - line.at).dot(line.dir);
        Some((point, self.extension_feedback(line, point, acquire, None)))
    }

    fn extension_feedback(
        &self, line: LeoExtension, q: Point, acquire: f64, target: Option<ExtensionTarget>,
    ) -> Vec<Overlay> {
        let span = (acquire * 64.0).min(1e9);
        let along = (q - line.at).dot(line.dir);
        let at = q - line.dir * along.clamp(-span, span);
        let (label, color) = match target {
            None => ("延长线吸附", EXTENSION_HIT),
            Some(ExtensionTarget::Anchor) => ("延长线 · 命中锚点", ANCHOR_HIT),
            Some(ExtensionTarget::Edge(..)) => ("延长线 · 命中边缘", PATH_HIT),
        };
        let mut guides = vec![];
        if self.style.lines {
            let mut outline = BezPath::new();
            outline.move_to(at);
            outline.line_to(q);
            guides.push(Overlay::Path { path: outline, color: [0x20, 0x24, 0x2b], width: 3.5, dashed: true });
            guides.push(Overlay::Line { a: at, b: q, color: EXTENSION_HIT, dashed: true });
            if let Some(ExtensionTarget::Edge(segment, t)) = target {
                let scale = segment.start().distance(segment.eval(0.5)) + segment.eval(0.5).distance(segment.end());
                let half = (acquire * 3.5 / scale.max(acquire)).min(0.5);
                let part = segment.subsegment((t - half).max(0.0)..(t + half).min(1.0));
                let mut path = BezPath::new();
                path.move_to(part.start());
                path.push(part.as_path_el());
                guides.push(Overlay::Path { path: path.clone(), color: [0x20, 0x24, 0x2b], width: 6.0, dashed: false });
                guides.push(Overlay::Path { path, color: PATH_HIT, width: 3.5, dashed: false });
            }
            let size = if target.is_some() { 14.0 } else { 10.0 };
            guides.push(Overlay::CurvePoint { p: q, color: [0x20, 0x24, 0x2b], filled: true, size: size + 2.0 });
            guides.push(Overlay::CurvePoint { p: q, color, filled: false, size });
        }
        if self.style.labels { guides.push(Overlay::Label { p: q, text: label.into(), color }); }
        guides
    }
}

fn choose_extension_contact(
    best: &mut Option<LeoExtensionContact>, candidate: LeoExtensionContact, query: Point,
) {
    let distance = candidate.point.distance(query);
    if best.is_none_or(|old| {
        let old_distance = old.point.distance(query);
        distance < old_distance - 1e-7 || ((distance - old_distance).abs() <= 1e-7
            && matches!(candidate.target, ExtensionTarget::Anchor) && !matches!(old.target, ExtensionTarget::Anchor))
    }) {
        *best = Some(candidate);
    }
}

#[cfg(test)]
mod leo_extension_contact_tests {
    use super::*;

    #[test]
    fn leo_snapping_engine_extension_contacts_edge_then_anchor_and_releases_alt() {
        let mut targets = Targets::default();
        targets.extensions.push((Point::new(10.0, 10.0), Vec2::new(0.0, 1.0)));
        let edge = PathSeg::Line(Line::new(Point::new(-40.0, 90.0), Point::new(60.0, 110.0)));
        targets.segments.push((Rect::new(-40.0, 90.0, 60.0, 110.0), edge));
        targets.points.push((Point::new(10.0, 140.0), Kind::Anchor));
        targets.points.push((Point::new(12.0, 175.0), Kind::Anchor));

        let (p, edge_guides) = targets.leo_snap_extension_contact(Point::new(12.0, 98.0), 8.0, 12.8, false).unwrap();
        assert!(p.distance(Point::new(10.0, 100.0)) < 1e-8);
        assert!(edge_guides.iter().any(|o| matches!(o, Overlay::Line { color, dashed: true, .. } if *color == EXTENSION_HIT)));
        assert!(edge_guides.iter().any(|o| matches!(o, Overlay::Path { color, dashed: false, .. } if *color == PATH_HIT)));

        let (p, anchor_guides) = targets.leo_snap_extension_contact(Point::new(40.0, 138.0), 8.0, 12.8, true).unwrap();
        assert_eq!(p, Point::new(10.0, 140.0));
        assert!(anchor_guides.iter().any(|o| matches!(o, Overlay::CurvePoint { p, color, .. } if *p == Point::new(10.0, 140.0) && *color == ANCHOR_HIT)));
        let (p, guides) = targets.leo_snap_extension_contact(Point::new(40.0, 175.0), 8.0, 12.8, true).unwrap();
        assert_eq!(p, Point::new(10.0, 175.0));
        assert!(guides.iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "延长线吸附")));
        assert!(targets.leo_snap_extension_contact(Point::new(40.0, 175.0), 8.0, 12.8, false).is_none());
    }
}

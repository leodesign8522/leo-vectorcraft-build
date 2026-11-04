//! Leo Design Snapping v0.1.0: point resolver, owned by the snapping edition.
//! Document collection and object resize/move adapters currently use upstream code.
use super::*;

impl Targets {
    pub(super) fn leo_snap_from(&self, p: Point, from: Option<&Leave>, tol: f64) -> (Point, Vec<Overlay>) {
        if !p.is_finite() || !tol.is_finite() || tol <= 0.0 {
            self.leo_point_lock.set(None);
            return (p, vec![]);
        }
        // Constraints always win over attraction; never move a Shift-drag off its line.
        if let Some(f) = from && let Some(q) = f.constrained(p) {
            self.leo_point_lock.set(None);
            let hit = self.slide(f.at, q - f.at, q, tol, true)
                .filter(|(hit, _, _)| hit.distance(q) <= tol);
            return match hit {
                Some((hit, target, kind)) => {
                    let mut ov: Vec<Overlay> = self.line(f.at, hit).into_iter().collect();
                    if kind == "intersect" { ov.extend(self.label(hit, "intersect")); }
                    else { ov.extend(self.lined_up(hit, target)); }
                    (hit, ov)
                }
                None => (q, self.line(f.at, q).into_iter().collect()),
            };
        }
        // An explicit source extension is resolved before unrelated path projections.
        if let Some((at, end, dir, projected)) = self.extension(p, tol) {
            self.leo_point_lock.set(None);
            let (hit, target, kind) = self.slide(at, dir, projected, tol, true)
                .filter(|(hit, _, _)| hit.distance(projected) <= tol)
                .unwrap_or((projected, projected, "extension"));
            let mut ov = vec![];
            if self.style.lines {
                ov.push(Overlay::Line { a: end, b: hit, color: EXTENSION_SNAP_COLOR, dashed: true });
            }
            ov.push(Overlay::CurvePoint { p: hit, color: EXTENSION_SNAP_COLOR, filled: true, size: 8.0 });
            if self.style.labels {
                ov.push(Overlay::Label { p: hit, text: "延长线吸附".into(), color: EXTENSION_SNAP_COLOR });
            }
            if kind == "intersect" { ov.extend(self.label(hit, "intersect")); }
            else if kind == "align" { ov.extend(self.lined_up(hit, target)); }
            return (hit, ov);
        }
        // A point has a larger release radius than acquisition radius. The lock belongs
        // to this target snapshot/gesture, never a global process or document singleton.
        if let Some((q, kind)) = self.leo_point_lock.get() {
            if q.distance(p) <= tol * 1.5 {
                return (q, self.label(q, kind.label()).into_iter().collect());
            }
            self.leo_point_lock.set(None);
        }
        if let Some((q, kind)) = self.nearest_point(p, tol) {
            self.leo_point_lock.set(Some((q, kind)));
            return (q, self.label(q, kind.label()).into_iter().collect());
        }
        if let Some(q) = self.crossing(p, tol) {
            return (q, self.label(q, "intersect").into_iter().collect());
        }
        if let Some(f) = from.filter(|f| f.contact)
            && let Some((q, perpendicular, dir)) = self.touching(f.at, p, tol) {
            return (q, self.touch_guides(f.at, q, perpendicular, dir, tol));
        }
        if let Some(q) = self.on_segment(p, tol) {
            return (q, self.label(q, Kind::Edge.label()).into_iter().collect());
        }
        if let Some((at, dir, q)) = from
            .and_then(|f| f.construction(p, tol).map(|(dir, q)| (f.at, dir, q)))
            .or_else(|| self.on_angled(p, tol)) {
            let hit = self.slide(at, dir, q, tol, false)
                .filter(|(hit, _, _)| hit.distance(q) <= tol);
            return match hit {
                Some((hit, target, kind)) => {
                    let mut ov: Vec<Overlay> = self.line(at, hit).into_iter().collect();
                    if kind == "intersect" { ov.extend(self.label(hit, "intersect")); }
                    else { ov.extend(self.lined_up(hit, target)); }
                    (hit, ov)
                }
                None => (q, self.line(at, q).into_iter().collect()),
            };
        }
        // Axis alignment is the fallback, so a nearby bounding edge cannot steal a node.
        let nearest = |lines: &[(f64, Point, Kind)], vertical: bool, v: f64| {
            let rulers = self.rulers.iter().filter(|r| r.vertical == vertical).filter_map(|r| r.line_at(p, tol));
            lines_near(lines, v, tol).iter().copied().chain(rulers)
                .filter(|(t, ..)| (t - v).abs() <= tol)
                .min_by(|a, b| (a.0 - v).abs().total_cmp(&(b.0 - v).abs()))
        };
        let mut out = p;
        let mut ov = vec![];
        let mut aligned = false;
        if let Some((x, start, _)) = nearest(&self.xs, true, p.x) {
            out.x = x; aligned = true;
            ov.extend(self.line(start, Point::new(x, p.y)));
        }
        if let Some((y, start, _)) = nearest(&self.ys, false, p.y) {
            out.y = y; aligned = true;
            ov.extend(self.line(start, Point::new(p.x, y)));
        }
        if aligned { ov.extend(self.label(out, "align")); }
        (out, ov)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leo_snapping_point_lock_resists_competing_targets_and_releases() {
        let mut t = Targets::default();
        t.points = vec![(Point::new(0.0, 0.0), Kind::Anchor), (Point::new(8.0, 0.0), Kind::Anchor)];
        let t = t.index();
        assert_eq!(t.snap_point(Point::new(1.0, 0.0), 6.0).0, Point::ZERO);
        assert_eq!(t.snap_point(Point::new(7.5, 0.0), 6.0).0, Point::ZERO);
        assert_eq!(t.snap_point(Point::new(10.0, 0.0), 6.0).0, Point::new(8.0, 0.0));
        assert_eq!(t.snap_point(Point::new(30.0, 20.0), 6.0).0, Point::new(30.0, 20.0));
    }
    #[test]
    fn leo_snapping_lock_is_per_gesture_and_zoom_scales_radius() {
        for zoom in [0.25, 1.0, 4.0] {
            let mut t = Targets::default();
            t.points.push((Point::ZERO, Kind::Anchor));
            let tol = 8.0 / zoom;
            assert_eq!(t.snap_point(Point::new(7.0 / zoom, 0.0), tol).0, Point::ZERO);
            assert_eq!(t.snap_point(Point::new(11.0 / zoom, 0.0), tol).0, Point::ZERO);
            assert_ne!(t.snap_point(Point::new(13.0 / zoom, 0.0), tol).0, Point::ZERO);
            let fresh = Targets::default();
            assert_eq!(fresh.snap_point(Point::new(1.0, 1.0), tol).0, Point::new(1.0, 1.0));
        }
    }
    #[test]
    fn leo_snapping_hidden_feedback_keeps_geometry() {
        let mut t = Targets::default();
        t.style.lines = false; t.style.labels = false;
        t.points.push((Point::ZERO, Kind::Anchor));
        let (p, ov) = t.snap_point(Point::new(1.0, 0.0), 8.0);
        assert_eq!(p, Point::ZERO);
        assert!(ov.is_empty());
    }
    #[test]
    fn leo_snapping_invalid_input_clears_lock() {
        let mut t = Targets::default();
        t.points.push((Point::ZERO, Kind::Anchor));
        t.snap_point(Point::new(1.0, 0.0), 8.0);
        t.snap_point(Point::new(1.0, 0.0), f64::NAN);
        assert!(t.leo_point_lock.get().is_none());
    }
    #[test]
    fn leo_snapping_extension_is_not_pulled_off_line_by_a_nearby_anchor() {
        let mut t = Targets::default();
        t.points.push((Point::new(10.0, 1.0), Kind::Anchor));
        t.extensions.push((Point::ZERO, Point::new(5.0, 0.0), Vec2::new(1.0, 0.0)));
        let (q, _) = t.snap_point(Point::new(10.0, 0.2), 2.0);
        assert_eq!(q, Point::new(10.0, 0.0));
        assert!(t.leo_point_lock.get().is_none());
    }
    #[test]
    fn leo_snapping_shift_clears_an_unconstrained_point_lock() {
        let mut t = Targets::default();
        t.points.push((Point::new(10.0, 0.0), Kind::Anchor));
        assert_eq!(t.snap_point(Point::new(10.0, 1.0), 8.0).0, Point::new(10.0, 0.0));
        let from = Leave::diagonal(Point::ZERO, true);
        let (q, _) = t.snap_from(Point::new(10.0, 1.0), Some(&from), 8.0);
        assert_eq!(q, Point::new(10.0, 10.0));
        assert!(t.leo_point_lock.get().is_none());
    }
}

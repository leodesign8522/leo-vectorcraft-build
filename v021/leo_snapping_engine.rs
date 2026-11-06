//! Leo Design Snapping Engine v1: true geometry snapping for Direct Selection anchors.
//! Stage 1 intentionally excludes alignment lines, bounds, centres and construction guides.
//! Candidate anchors win over paths. Captured targets have hysteresis in SCREEN pixels.
use super::*;

pub(super) const ACQUIRE_PX: f64 = 8.0;
pub(super) const RELEASE_MULTIPLIER: f64 = 1.6;
const ANCHOR_HIT: [u8; 3] = [0x35, 0xc2, 0xfa];
const PATH_HIT: [u8; 3] = [0xff, 0xb3, 0x3d];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::guides) enum SnapLock {
    Anchor(Point),
    Segment(usize),
}

impl Targets {
    pub(in crate::guides) fn clear_leo_snap(&self) {
        self.leo_snap.set(None);
    }

    fn leo_nearest_anchor(&self, p: Point, reach: f64) -> Option<Point> {
        self.points
            .iter()
            .filter(|(_, kind)| *kind == Kind::Anchor)
            .filter(|(q, _)| q.distance(p) <= reach)
            .min_by(|(a, _), (b, _)| a.distance(p).total_cmp(&b.distance(p)))
            .map(|(q, _)| *q)
    }

    fn leo_nearest_segment(&self, p: Point, reach: f64) -> Option<(usize, Point)> {
        self.segments
            .iter()
            .enumerate()
            .filter(|(_, (bounds, _))| bounds.inflate(reach, reach).contains(p))
            .filter_map(|(index, (_, segment))| {
                // Kurbo's nearest-point solver projects onto the actual finite
                // line or Bezier, never its bounding rectangle or guide line.
                let nearest = segment.nearest(p, 1e-8);
                (nearest.distance_sq.is_finite() && nearest.distance_sq <= reach * reach)
                    .then_some((index, segment.eval(nearest.t), nearest.distance_sq))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(index, q, _)| (index, q))
    }

    fn leo_locked_point(&self, lock: SnapLock, p: Point, release: f64) -> Option<Point> {
        match lock {
            SnapLock::Anchor(q) => (q.distance(p) <= release).then_some(q),
            SnapLock::Segment(index) => self.segments.get(index).and_then(|(_, segment)| {
                let nearest = segment.nearest(p, 1e-8);
                (nearest.distance_sq.is_finite() && nearest.distance_sq <= release * release)
                    .then(|| segment.eval(nearest.t))
            }),
        }
    }

    /// Returns None when no REAL geometry captured. No alignment or phantom hit
    /// can be reported to Direct Selection. Distances arrive in document units,
    /// converted from 8px / 12.8px in the caller at the current zoom.
    pub(in crate::guides) fn leo_snap_anchor(&self, p: Point, acquire: f64, release: f64) -> Option<(Point, Vec<Overlay>)> {
        if !p.is_finite() || !acquire.is_finite() || !release.is_finite() || acquire <= 0.0 || release < acquire {
            self.clear_leo_snap();
            return None;
        }

        let anchor = self.leo_nearest_anchor(p, acquire);
        let held = self.leo_snap.get();
        let choice = match held {
            // Keep an existing anchor even if a nearby path looks closer.
            Some(lock @ SnapLock::Anchor(_)) => self.leo_locked_point(lock, p, release).map(|q| (lock, q))
                .or_else(|| anchor.map(|q| (SnapLock::Anchor(q), q))),
            // An actual anchor gets priority over a captured path, particularly at endpoints.
            Some(lock @ SnapLock::Segment(_)) => anchor.map(|q| (SnapLock::Anchor(q), q))
                .or_else(|| self.leo_locked_point(lock, p, release).map(|q| (lock, q))),
            None => anchor.map(|q| (SnapLock::Anchor(q), q)),
        }
        .or_else(|| self.leo_nearest_segment(p, acquire).map(|(i, q)| (SnapLock::Segment(i), q)));

        self.leo_snap.set(choice.map(|(lock, _)| lock));
        let (lock, q) = choice?;
        let (color, label) = match lock {
            SnapLock::Anchor(_) => (ANCHOR_HIT, "anchor"),
            SnapLock::Segment(_) => (PATH_HIT, "path"),
        };
        let mut guides = Vec::new();
        if self.style.lines {
            guides.push(Overlay::CurvePoint { p: q, color, filled: false, size: 8.0 });
        }
        if self.style.labels {
            guides.push(Overlay::Label { p: q, text: label.into(), color });
        }
        Some((q, guides))
    }
}

#[cfg(test)]
mod leo_snapping_engine_tests {
    use super::*;

    fn targets() -> Targets {
        let mut t = Targets::default();
        t.points.push((Point::new(100.0, 100.0), Kind::Anchor));
        // Synthetic centres/bounds must NEVER take priority over geometry.
        t.points.push((Point::new(200.0, 200.0), Kind::Center));
        t.xs.push((250.0, Point::new(250.0, 10.0), Kind::Edge));
        t.ys.push((250.0, Point::new(10.0, 250.0), Kind::Edge));
        t
    }

    fn snap(t: &Targets, p: Point, zoom: f64) -> Option<(Point, Vec<Overlay>)> {
        t.leo_snap_anchor(p, ACQUIRE_PX / zoom, ACQUIRE_PX * RELEASE_MULTIPLIER / zoom)
    }

    #[test]
    fn leo_snapping_engine_anchor_is_exact_and_has_no_phantom_alignment() {
        let t = targets();
        let (q, overlays) = snap(&t, Point::new(106.0, 102.0), 1.0).unwrap();
        assert_eq!(q, Point::new(100.0, 100.0));
        assert!(overlays.iter().any(|o| matches!(o, Overlay::CurvePoint { p, .. } if *p == q)));
        t.clear_leo_snap();
        assert!(snap(&t, Point::new(201.0, 201.0), 1.0).is_none(), "centre is not an anchor");
        assert!(snap(&t, Point::new(250.0, 10.0), 1.0).is_none(), "guide lines cannot steal a hit");
    }

    #[test]
    fn leo_snapping_engine_anchor_priority_over_intersecting_path() {
        let mut t = targets();
        t.segments.push((Rect::new(80.0, 100.0, 120.0, 100.0),
            PathSeg::Line(Line::new((80.0, 100.0), (120.0, 100.0)))));
        let (q, overlays) = snap(&t, Point::new(103.0, 101.0), 1.0).unwrap();
        assert_eq!(q, Point::new(100.0, 100.0));
        assert!(overlays.iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "anchor")));
    }

    #[test]
    fn leo_snapping_engine_finite_line_projects_to_actual_segment() {
        let mut t = Targets::default();
        t.segments.push((Rect::new(10.0, 20.0, 50.0, 20.0),
            PathSeg::Line(Line::new((10.0, 20.0), (50.0, 20.0)))));
        assert_eq!(snap(&t, Point::new(33.0, 26.0), 1.0).unwrap().0, Point::new(33.0, 20.0));
        t.clear_leo_snap();
        assert!(snap(&t, Point::new(60.0, 20.0), 1.0).is_none(), "not an infinite line extension");
    }

    #[test]
    fn leo_snapping_engine_cubic_bezier_uses_nearest_curve_geometry() {
        let mut t = Targets::default();
        let c = vectorcraft_geom::kurbo::CubicBez::new((0.0, 0.0), (0.0, 40.0), (40.0, 40.0), (40.0, 0.0));
        t.segments.push((Rect::new(0.0, 0.0, 40.0, 40.0), PathSeg::Cubic(c)));
        let (q, overlays) = snap(&t, Point::new(20.0, 35.0), 1.0).unwrap();
        assert!(q.distance(Point::new(20.0, 30.0)) < 1e-5, "projected to cubic not bounds: {q:?}");
        assert!(overlays.iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "path")));
    }

    #[test]
    fn leo_snapping_engine_lock_releases_at_twelve_point_eight_screen_pixels() {
        let t = targets();
        assert_eq!(snap(&t, Point::new(107.9, 100.0), 1.0).unwrap().0, Point::new(100.0, 100.0));
        assert_eq!(snap(&t, Point::new(112.7, 100.0), 1.0).unwrap().0, Point::new(100.0, 100.0));
        assert!(snap(&t, Point::new(112.9, 100.0), 1.0).is_none());
        assert_eq!(t.leo_snap.get(), None);
    }

    #[test]
    fn leo_snapping_engine_captured_segment_tracks_curve_until_release() {
        let mut t = Targets::default();
        t.segments.push((Rect::new(0.0, 0.0, 100.0, 0.0), PathSeg::Line(Line::new((0.0, 0.0), (100.0, 0.0)))));
        assert_eq!(snap(&t, Point::new(20.0, 7.0), 1.0).unwrap().0, Point::new(20.0, 0.0));
        assert_eq!(snap(&t, Point::new(24.0, 12.0), 1.0).unwrap().0, Point::new(24.0, 0.0));
        assert!(snap(&t, Point::new(24.0, 13.0), 1.0).is_none());
    }

    #[test]
    fn leo_snapping_engine_eight_pixel_capture_is_zoom_invariant() {
        for zoom in [0.25, 1.0, 4.0] {
            let t = targets();
            let near = Point::new(100.0 + 7.9 / zoom, 100.0);
            let far = Point::new(100.0 + 8.1 / zoom, 100.0);
            assert_eq!(snap(&t, near, zoom).unwrap().0, Point::new(100.0, 100.0), "zoom: {zoom}");
            t.clear_leo_snap();
            assert!(snap(&t, far, zoom).is_none(), "zoom: {zoom}");
        }
    }

    #[test]
    fn leo_snapping_engine_snaps_with_hidden_visual_guides() {
        let mut t = targets();
        t.style.lines = false;
        t.style.labels = false;
        let (q, overlays) = snap(&t, Point::new(104.0, 100.0), 1.0).unwrap();
        assert_eq!(q, Point::new(100.0, 100.0));
        assert!(overlays.is_empty());
    }
}

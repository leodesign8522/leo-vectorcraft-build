//! Priority and feedback for moving anchors with the Direct Selection tool.
use super::*;

const VERTICAL: [u8; 3] = [0x3e, 0xa7, 0x71];
const HORIZONTAL: [u8; 3] = [0x59, 0x4a, 0x97];
const HIT: [u8; 3] = [0xff, 0xbf, 0x99];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::guides) enum DragGuide {
    Vertical(Point),
    Horizontal(Point),
    Extension { fixed: Point, dir: Vec2 },
}

impl DragGuide {
    fn project(self, p: Point) -> Point {
        match self {
            Self::Vertical(at) => Point::new(at.x, p.y),
            Self::Horizontal(at) => Point::new(p.x, at.y),
            Self::Extension { fixed, dir } => fixed + dir * (p - fixed).dot(dir),
        }
    }

    fn distance(self, p: Point) -> f64 {
        self.project(p).distance(p)
    }

    fn overlay(self, q: Point, hit: bool, pad: f64) -> Overlay {
        let (at, color, dir) = match self {
            Self::Vertical(at) => (at, VERTICAL, Vec2::new(0.0, 1.0)),
            Self::Horizontal(at) => (at, HORIZONTAL, Vec2::new(1.0, 0.0)),
            Self::Extension { fixed, dir } => (fixed, HIT, dir),
        };
        let dir = if (q - at).dot(dir) < 0.0 { -dir } else { dir };
        Overlay::Line { a: at - dir * pad, b: q + dir * pad, color: if hit { HIT } else { color }, dashed: false }
    }
}

impl Targets {
    pub(in crate::guides) fn clear_drag_guide(&self) {
        self.drag_guide.set(None);
    }

    fn drag_guide_near(&self, p: Point, reach: f64, bias: f64) -> Option<DragGuide> {
        let axis = |vertical| {
            let (lines, value) = if vertical { (&self.xs, p.x) } else { (&self.ys, p.y) };
            let rulers = self.rulers.iter().filter(|r| r.vertical == vertical).filter_map(|r| r.line_at(p, reach));
            lines_near(lines, value, reach).iter().copied().chain(rulers)
                .filter(|(v, ..)| (v - value).abs() <= reach)
                .map(|(_, at, _)| if vertical { DragGuide::Vertical(at) } else { DragGuide::Horizontal(at) })
                .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
        };
        let axis = [axis(true), axis(false)].into_iter().flatten()
            .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
        // The existing straight-edge source geometry supports both extending and shortening.
        let extension = self.extensions.iter().copied()
            .filter(|(fixed, _)| fixed.distance(p) > reach)
            .map(|(fixed, dir)| DragGuide::Extension { fixed, dir })
            .filter(|guide| guide.distance(p) <= reach)
            .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
        match (axis, extension) {
            (Some(axis), Some(ext)) if ext.distance(p) + bias < axis.distance(p) => Some(ext),
            (axis, extension) => axis.or(extension),
        }
    }

    pub(in crate::guides) fn snap_direct_drag(&self, p: Point, from: Option<&Leave>, native_tol: f64, acquire: f64) -> (Point, Vec<Overlay>) {
        if !p.is_finite() || !acquire.is_finite() || acquire <= 0.0 {
            self.clear_drag_guide();
            return (p, vec![]);
        }
        // Keep a captured guide through small pointer movements. Use a fresh candidate after release.
        let hit = self.drag_guide.get()
            .filter(|guide| guide.distance(p) <= acquire * 1.4)
            .or_else(|| self.drag_guide_near(p, acquire, acquire * 0.18));
        self.drag_guide.set(hit);
        if let Some(guide) = hit {
            let q = guide.project(p);
            let mut out = vec![];
            if self.style.lines {
                out.push(guide.overlay(q, true, acquire * 2.0));
                out.push(Overlay::CurvePoint { p: q, color: HIT, filled: false, size: 8.0 });
            }
            return (q, out);
        }

        let (q, mut out) = self.snap_from(p, from, native_tol);
        if q.distance(p) > 1e-8 || !out.is_empty() {
            self.color_drag_hit(q, &mut out);
        } else if self.style.lines && let Some(guide) = self.drag_guide_near(p, acquire * 1.9, acquire * 0.18) {
            out.push(guide.overlay(guide.project(p), false, acquire * 2.0));
        }
        (q, out)
    }

    pub(in crate::guides) fn color_drag_hit(&self, q: Point, out: &mut Vec<Overlay>) {
        for overlay in out.iter_mut() {
            match overlay {
                Overlay::Line { color, .. } | Overlay::Label { color, .. } | Overlay::Path { color, .. } => *color = HIT,
                _ => (),
            }
        }
        if self.style.lines {
            out.push(Overlay::CurvePoint { p: q, color: HIT, filled: false, size: 8.0 });
        }
    }
}

#[cfg(test)]
mod leo_anchor_drag_feedback_tests {
    use super::*;

    #[test]
    fn previews_are_axis_colored_and_capture_releases_beyond_the_lock_radius() {
        for vertical in [true, false] {
            let mut t = Targets::default();
            let at = Point::ZERO;
            let (preview, near, between, away, expected, color) = if vertical {
                t.xs.push((0.0, at, Kind::Edge));
                (Point::new(9.0, 40.0), Point::new(4.0, 40.0), Point::new(6.0, 41.0), Point::new(8.0, 42.0), Point::new(0.0, 41.0), VERTICAL)
            } else {
                t.ys.push((0.0, at, Kind::Edge));
                (Point::new(40.0, 9.0), Point::new(40.0, 4.0), Point::new(41.0, 6.0), Point::new(42.0, 8.0), Point::new(41.0, 0.0), HORIZONTAL)
            };
            let (q, hints) = t.snap_direct_drag(preview, None, 5.0, 5.0);
            assert_eq!(q, preview);
            assert!(matches!(&hints[..], [Overlay::Line { color: c, .. }] if *c == color));
            let (_, hints) = t.snap_direct_drag(near, None, 5.0, 5.0);
            assert!(matches!(&hints[..], [Overlay::Line { color: a, .. }, Overlay::CurvePoint { color: b, .. }] if *a == HIT && *b == HIT));
            assert_eq!(t.snap_direct_drag(between, None, 5.0, 5.0).0, expected);
            assert_eq!(t.snap_direct_drag(away, None, 5.0, 5.0).0, away);
        }
    }

    #[test]
    fn a_slanted_extension_captures_in_both_directions_without_changing_the_slope() {
        let mut t = Targets::default();
        let dir = Vec2::new(0.6, 0.8);
        t.extensions.push((Point::ZERO, dir));
        for along in [-50.0, 50.0] {
            t.clear_drag_guide();
            let q = Point::ZERO + dir * along;
            let (snapped, hints) = t.snap_direct_drag(q + Vec2::new(0.8, -0.6), None, 5.0, 5.0);
            assert!(snapped.distance(q) < 1e-8);
            assert!(matches!(&hints[..], [Overlay::Line { color: a, .. }, Overlay::CurvePoint { color: b, .. }] if *a == HIT && *b == HIT));
        }
    }
}

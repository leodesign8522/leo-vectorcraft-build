//! Rotation follows a fixed-radius trajectory: only real, reachable geometry is a target.
use super::*;

const ROTATION_COLOR: [u8; 3] = [0xff, 0xb3, 0x3d];

// Isolate roots between derivative roots, including tangencies, on the finite segment [0, 1].
fn roots(coeff: &[f64]) -> Vec<f64> {
    let scale = coeff.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    if !scale.is_finite() || scale == 0.0 {
        return vec![];
    }
    let mut c: Vec<f64> = coeff.iter().map(|v| v / scale).collect();
    while c.len() > 1 && c.last().unwrap().abs() < 1e-14 {
        c.pop();
    }
    if c.len() < 2 {
        return vec![];
    }
    if c.len() == 2 {
        let t = -c[0] / c[1];
        return if (-1e-12..=1.0 + 1e-12).contains(&t) {
            vec![t.clamp(0.0, 1.0)]
        } else {
            vec![]
        };
    }
    let derivative: Vec<f64> = c
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, v)| *v * i as f64)
        .collect();
    let mut cuts = vec![0.0];
    cuts.extend(roots(&derivative));
    cuts.push(1.0);
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    let eval = |t: f64| c.iter().rev().fold(0.0, |out, v| out * t + v);
    let mut out: Vec<f64> = cuts
        .iter()
        .copied()
        .filter(|t| eval(*t).abs() <= 1e-12)
        .collect();
    for range in cuts.windows(2) {
        let (mut lo, mut hi) = (range[0], range[1]);
        let mut f_lo = eval(lo);
        if f_lo * eval(hi) >= 0.0 {
            continue;
        }
        for _ in 0..64 {
            let mid = (lo + hi) * 0.5;
            let f_mid = eval(mid);
            if f_mid == 0.0 {
                lo = mid;
                hi = mid;
                break;
            }
            if f_lo * f_mid <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                f_lo = f_mid;
            }
        }
        out.push((lo + hi) * 0.5);
    }
    out.sort_by(f64::total_cmp);
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    out
}

#[cfg(test)]
mod leo_snapping_rotation_geometry_tests {
    use super::*;

    #[test]
    fn finite_edges_intersect_the_rotation_circle_on_both_sides() {
        let edge = PathSeg::Line(Line::new((80.0, -100.0), (80.0, 100.0)));
        let hits = circle_hits(&edge, Point::ZERO, 100.0);
        assert_eq!(hits.len(), 2);
        for p in [Point::new(80.0, -60.0), Point::new(80.0, 60.0)] {
            assert!(hits.iter().any(|h| h.distance(p) < 1e-8), "{hits:?}");
        }
        let short = PathSeg::Line(Line::new((80.0, 0.0), (80.0, 50.0)));
        assert!(
            circle_hits(&short, Point::ZERO, 100.0).is_empty(),
            "no invisible edge extensions"
        );
    }

    #[test]
    fn circle_solver_finds_tangencies_endpoints_and_curves() {
        let tangent = PathSeg::Line(Line::new((-40.0, 100.0), (40.0, 100.0)));
        let hits = circle_hits(&tangent, Point::ZERO, 100.0);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].distance(Point::new(0.0, 100.0)) < 1e-8);
        let endpoint = PathSeg::Line(Line::new((0.0, 100.0), (0.0, 120.0)));
        assert!(
            circle_hits(&endpoint, Point::ZERO, 100.0)
                .iter()
                .any(|p| p.distance(Point::new(0.0, 100.0)) < 1e-8)
        );
        let curve = PathSeg::Cubic(vectorcraft_geom::kurbo::CubicBez::new(
            (-1.0, 2.0),
            (-1.0 / 3.0, 2.0 / 3.0),
            (1.0 / 3.0, 2.0 / 3.0),
            (1.0, 2.0),
        ));
        assert!(
            circle_hits(&curve, Point::ZERO, 1.0)
                .iter()
                .any(|p| p.distance(Point::new(0.0, 1.0)) < 1e-7)
        );
        let curve = PathSeg::Quad(vectorcraft_geom::kurbo::QuadBez::new(
            (40.0, 20.0),
            (90.0, 120.0),
            (150.0, 50.0),
        ));
        let mid = curve.eval(0.5);
        assert!(
            circle_hits(&curve, Point::ZERO, mid.distance(Point::ZERO))
                .iter()
                .any(|p| p.distance(mid) < 1e-7)
        );
    }

    #[test]
    fn reachable_edges_only_and_hidden_feedback_preserves_geometry() {
        let mut t = Targets::default();
        let edge = PathSeg::Line(Line::new((80.0, 20.0), (80.0, 100.0)));
        t.segments.push((Rect::new(80.0, 20.0, 80.0, 100.0), edge));
        let raw = Affine::rotate(-0.02) * Point::new(80.0, 60.0);
        let (q, ov) = t
            .leo_rotation_hit(Point::ZERO, 100.0, raw, 4.0, true)
            .unwrap();
        assert!(q.distance(Point::new(80.0, 60.0)) < 1e-8);
        assert!(
            ov.iter().all(|o| match o {
                Overlay::Line { color, .. }
                | Overlay::CurvePoint { color, .. }
                | Overlay::Label { color, .. } => *color == ROTATION_COLOR,
                _ => false,
            }),
            "no ordinary horizontal/vertical alignment overlays: {ov:?}"
        );
        t.style.lines = false;
        t.style.labels = false;
        assert!(
            t.leo_rotation_hit(Point::ZERO, 100.0, raw, 4.0, true)
                .unwrap()
                .1
                .is_empty()
        );
        t.leo_clear_rotation();
        assert!(
            t.leo_rotation_hit(
                Point::ZERO,
                50.0,
                Point::new(raw.x * 0.5, raw.y * 0.5),
                4.0,
                true
            )
            .is_none()
        );
    }

    #[test]
    fn lock_resists_competing_points_and_releases_without_axis_guides() {
        let mut t = Targets::default();
        let a = Point::new(100.0, 0.0);
        let b = Affine::rotate(1.0_f64.to_radians()) * a;
        t.points = vec![(a, Kind::Anchor), (b, Kind::Anchor)];
        t.xs.push((b.x, b, Kind::Edge));
        assert_eq!(
            t.leo_rotation_hit(Point::ZERO, 100.0, a, 2.0, true)
                .unwrap()
                .0,
            a
        );
        assert_eq!(
            t.leo_rotation_hit(Point::ZERO, 100.0, b, 2.0, true)
                .unwrap()
                .0,
            a
        );
        let far = Affine::rotate(5.0_f64.to_radians()) * a;
        assert!(
            t.leo_rotation_hit(Point::ZERO, 100.0, far, 2.0, true)
                .is_none()
        );
        assert!(t.leo_rotation_lock.get().is_none());
    }
}

fn circle_hits(segment: &PathSeg, origin: Point, radius: f64) -> Vec<Point> {
    let power: Vec<Vec2> = match segment {
        PathSeg::Line(l) => vec![l.p0 - origin, l.p1 - l.p0],
        PathSeg::Quad(q) => vec![
            q.p0 - origin,
            (q.p1 - q.p0) * 2.0,
            (q.p0 - q.p1) + (q.p2 - q.p1),
        ],
        PathSeg::Cubic(c) => vec![
            c.p0 - origin,
            (c.p1 - c.p0) * 3.0,
            ((c.p0 - c.p1) + (c.p2 - c.p1)) * 3.0,
            (c.p3 - c.p0) + (c.p1 - c.p2) * 3.0,
        ],
    };
    let mut coeff = vec![0.0; power.len() * 2 - 1];
    for (i, a) in power.iter().enumerate() {
        for (j, b) in power.iter().enumerate() {
            coeff[i + j] += a.dot(*b);
        }
    }
    coeff[0] -= radius * radius;
    roots(&coeff)
        .into_iter()
        .map(|t| segment.eval(t))
        .filter(|q| q.is_finite() && (q.distance(origin) - radius).abs() <= radius.max(1.0) * 1e-8)
        .collect()
}

impl Targets {
    pub(crate) fn leo_for_rotation(cx: &ToolContext) -> Self {
        // Unlike drawing, the moving selection is entirely excluded, including its own edges.
        Self::collect_inner(cx.doc, &cx.selection.objects, None, None, Gather::Segments)
    }

    pub(crate) fn leo_clear_rotation(&self) {
        self.leo_rotation_lock.set(None);
    }

    pub(crate) fn leo_rotation_hit(
        &self,
        origin: Point,
        radius: f64,
        raw: Point,
        tol: f64,
        edges: bool,
    ) -> Option<(Point, Vec<Overlay>)> {
        if !raw.is_finite()
            || !origin.is_finite()
            || !radius.is_finite()
            || radius <= 1e-9
            || !tol.is_finite()
            || tol <= 0.0
        {
            self.leo_clear_rotation();
            return None;
        }
        // Small-radius gestures must not jump through a large angle just to catch a target.
        let reach = tol.min(2.0 * radius * 2.5_f64.to_radians().sin());
        let locked = self.leo_rotation_lock.get().filter(|(o, r, q, anchor)| {
            o.distance(origin) < 1e-9
                && (*r - radius).abs() < radius.max(1.0) * 1e-9
                && (*anchor || edges)
                && q.distance(raw) <= reach * 1.5
        });
        let best = if let Some((_, _, q, anchor)) = locked {
            Some((q.distance(raw), q, anchor))
        } else {
            self.leo_clear_rotation();
            let mut best: Option<(f64, Point, bool)> = None;
            for &(q, kind) in &self.points {
                if kind != Kind::Anchor
                    || (q.distance(origin) - radius).abs() > radius.max(1.0) * 1e-9
                {
                    continue;
                }
                let d = q.distance(raw);
                if d <= reach && best.is_none_or(|b| d < b.0) {
                    best = Some((d, q, true));
                }
            }
            if edges {
                for (_, segment) in self
                    .segments
                    .iter()
                    .filter(|(bounds, _)| bounds.inflate(reach, reach).contains(raw))
                {
                    for q in circle_hits(segment, origin, radius) {
                        let d = q.distance(raw);
                        if d <= reach && best.is_none_or(|b| d < b.0 - 1e-9) {
                            best = Some((d, q, false));
                        }
                    }
                }
            }
            best
        };
        let (_, hit, anchor) = best?;
        self.leo_rotation_lock
            .set(Some((origin, radius, hit, anchor)));
        let mut overlays = vec![];
        if self.style.lines {
            overlays.push(Overlay::Line {
                a: origin,
                b: hit,
                color: ROTATION_COLOR,
                dashed: true,
            });
        }
        if self.style.lines || self.style.labels {
            overlays.push(Overlay::CurvePoint {
                p: hit,
                color: ROTATION_COLOR,
                filled: true,
                size: 8.0,
            });
        }
        if self.style.labels {
            overlays.push(Overlay::Label {
                p: hit,
                text: if anchor {
                    "旋转节点吸附"
                } else {
                    "旋转贴边"
                }
                .into(),
                color: ROTATION_COLOR,
            });
        }
        Some((hit, overlays))
    }
}

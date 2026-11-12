use std::f64::consts::{FRAC_PI_2, TAU};
use vectorcraft_geom::{BezPath, ParamCurve, PathSeg, Point, Vec2};

#[derive(Clone, Copy, Debug)]
pub(super) struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64) -> Option<Self> {
        (center.is_finite() && radius.is_finite() && radius > 1e-8
            && (center.x.abs().max(center.y.abs()) + radius).is_finite()).then_some(Self { center, radius })
    }

    pub fn diameter(a: Point, b: Point) -> Option<Self> {
        Self::new(a + (b - a) * 0.5, a.distance(b) * 0.5)
    }

    pub fn through(a: Point, b: Point, c: Point) -> Option<Self> {
        let u = b - a;
        let v = c - a;
        let scale = u.hypot().max(v.hypot());
        if scale <= 1e-8 || !scale.is_finite() { return None; }
        let (u, v) = (u / scale, v / scale);
        let det = 2.0 * u.cross(v);
        if det.abs() < 1e-10 { return None; }
        let offset = Vec2::new(
            u.hypot2() * v.y - v.hypot2() * u.y,
            u.x * v.hypot2() - v.x * u.hypot2(),
        ) * (scale / det);
        Self::new(a + offset, offset.hypot())
    }

    pub fn tangent(start: Point, end: Point, direction: Vec2) -> Option<Self> {
        let n = normal(direction)?;
        let chord = end - start;
        let projection = chord.dot(n);
        if projection.abs() <= 1e-10 * chord.hypot() { return None; }
        let signed_radius = chord.hypot2() / (2.0 * projection);
        Self::new(start + n * signed_radius, signed_radius.abs())
    }

    pub fn angle(self, p: Point) -> f64 {
        let d = p - self.center;
        d.y.atan2(d.x)
    }

    pub fn point(self, angle: f64) -> Point {
        self.center + Vec2::new(angle.cos(), angle.sin()) * self.radius
    }

    pub fn path(self, start: f64, sweep: f64) -> BezPath {
        let pieces = (sweep.abs() / FRAC_PI_2).ceil().clamp(1.0, 4.0) as usize;
        let step = sweep / pieces as f64;
        let k = 4.0 / 3.0 * (step / 4.0).tan() * self.radius;
        let tangent = |a: f64| Vec2::new(-a.sin(), a.cos());
        let mut path = BezPath::new();
        path.move_to(self.point(start));
        for i in 0..pieces {
            let a = start + step * i as f64;
            let b = a + step;
            let (p, q) = (self.point(a), self.point(b));
            path.curve_to(p + tangent(a) * k, q - tangent(b) * k, q);
        }
        path
    }

    pub fn arc(self, a: Point, b: Point, control: Point, tangent: bool) -> BezPath {
        let start = self.angle(a);
        let ccw = (self.angle(b) - start).rem_euclid(TAU);
        let positive = if tangent {
            let n = b - self.center;
            Vec2::new(-n.y, n.x).dot(control - b) <= 0.0
        } else {
            (self.angle(control) - start).rem_euclid(TAU) <= ccw + 1e-10
        };
        self.path(start, if positive { ccw } else { ccw - TAU })
    }
}

fn normal(d: Vec2) -> Option<Vec2> {
    let length = d.hypot();
    (length.is_finite() && length > 1e-9).then(|| Vec2::new(-d.y, d.x) / length)
}

fn derivatives(segment: PathSeg, t: f64) -> (Vec2, Vec2) {
    let u = 1.0 - t;
    match segment {
        PathSeg::Line(l) => (l.p1 - l.p0, Vec2::ZERO),
        PathSeg::Quad(q) => {
            let (a, b) = (q.p1 - q.p0, q.p2 - q.p1);
            ((a * u + b * t) * 2.0, (b - a) * 2.0)
        }
        PathSeg::Cubic(c) => {
            let (a, b, c) = (c.p1 - c.p0, c.p2 - c.p1, c.p3 - c.p2);
            ((a * (u * u) + b * (2.0 * u * t) + c * (t * t)) * 3.0, ((b - a) * u + (c - b) * t) * 6.0)
        }
    }
}

pub(super) fn curvature(segment: PathSeg, t: f64) -> Option<Circle> {
    let (d, dd) = derivatives(segment, t);
    let n = normal(d)?;
    let k = d.cross(dd) / d.hypot().powi(3);
    if !k.is_finite() || k.abs() < 1e-12 { return None; }
    Circle::new(segment.eval(t) + n / k, (1.0 / k).abs())
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Tangent {
    pub circle: Circle,
    pub first: f64,
    pub second: f64,
}

fn push_solution(out: &mut Vec<Tangent>, candidate: Tangent) {
    let epsilon = 1e-6 + candidate.circle.radius * 1e-8;
    if !out.iter().any(|s| s.circle.center.distance(candidate.circle.center) <= epsilon
        && (s.circle.radius - candidate.circle.radius).abs() <= epsilon)
    {
        out.push(candidate);
    }
}

pub(super) fn anchored(first: PathSeg, t: f64, second: PathSeg, hint: f64) -> Vec<Tangent> {
    let mut out = Vec::new();
    let a = first.eval(t);
    let Some(n) = normal(derivatives(first, t).0) else { return out; };
    // Multiply through by the chord's normal projection to avoid poles while
    // isolating roots. Reject coincident points and flat derivatives afterward.
    let residual = |u| {
        let chord = second.eval(u) - a;
        let d = derivatives(second, u).0;
        (n * chord.hypot2() - chord * (2.0 * n.dot(chord))).dot(d)
    };
    let mut accept = |u| {
        let b = second.eval(u);
        let Some(circle) = Circle::tangent(a, b, derivatives(first, t).0) else { return; };
        let Some(nb) = normal(derivatives(second, u).0) else { return; };
        let radial = circle.center - b;
        let epsilon = 1e-6 + circle.radius * 1e-8;
        if radial.cross(nb).abs() <= epsilon && (radial.hypot() - circle.radius).abs() <= epsilon {
            push_solution(&mut out, Tangent { circle, first: t, second: u });
        }
    };
    accept(hint);
    for i in 0..64 {
        let (mut lo, mut hi) = (i as f64 / 64.0, (i + 1) as f64 / 64.0);
        let mut left = residual(lo);
        accept(lo);
        let right = residual(hi);
        if left.is_finite() && right.is_finite() && left.signum() != right.signum() {
            for _ in 0..42 {
                let mid = (lo + hi) * 0.5;
                let value = residual(mid);
                if value.signum() == left.signum() { lo = mid; left = value; } else { hi = mid; }
            }
            accept((lo + hi) * 0.5);
        }
    }
    accept(1.0);
    out
}

// Evaluate a parallel curve and its derivative at the original finite
// segment parameter. Its intersections give candidate circle centres.
fn offset(segment: PathSeg, t: f64, r: f64) -> Option<(Point, Vec2)> {
    let (d, dd) = derivatives(segment, t);
    let n = normal(d)?;
    let k = d.cross(dd) / d.hypot().powi(3);
    Some((segment.eval(t) + n * r, d * (1.0 - r * k)))
}

pub(super) fn fixed(first: PathSeg, a: f64, second: PathSeg, b: f64, radius: f64) -> Vec<Tangent> {
    let mut out = Vec::new();
    let epsilon = 1e-6 + radius * 1e-8;
    for ra in [-radius, radius] {
        for rb in [-radius, radius] {
            // Seed finite parameter space as well as the two pointer hits.
            for i in 0..=8 {
                for j in 0..=8 {
                    let (mut t, mut u) = (if i == 8 { a } else { i as f64 / 7.0 }, if j == 8 { b } else { j as f64 / 7.0 });
                    for _ in 0..24 {
                        let (Some((p, v)), Some((q, w))) = (offset(first, t, ra), offset(second, u, rb)) else { break; };
                        let error = p - q;
                        if error.hypot() <= epsilon {
                            if let Some(circle) = Circle::new(p, radius) {
                                push_solution(&mut out, Tangent { circle, first: t, second: u });
                            }
                            break;
                        }
                        let det = v.cross(w);
                        if det.abs() <= 1e-12 * v.hypot() * w.hypot() { break; }
                        let dt = w.cross(error) / det;
                        let du = v.cross(error) / det;
                        let scale = 1.0_f64.max(dt.abs().max(du.abs()) * 4.0);
                        let next = ((t + dt / scale).clamp(0.0, 1.0), (u + du / scale).clamp(0.0, 1.0));
                        if next == (t, u) { break; }
                        (t, u) = next;
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_geom::Line;
    use vectorcraft_geom::kurbo::CubicBez;

    #[test]
    fn leo_geometry_three_points_and_major_arc() {
        let (a, b, c) = (Point::new(10.0, 0.0), Point::new(0.0, 10.0), Point::new(-10.0, 0.0));
        let circle = Circle::through(a, b, c).unwrap();
        assert!(circle.center.distance(Point::ORIGIN) < 1e-8);
        assert!((circle.radius - 10.0).abs() < 1e-8);
        assert!(Circle::through(a, c, Point::ORIGIN).is_none());
        let arc = circle.arc(a, b, c, false);
        assert_eq!(arc.segments().count(), 3);
        assert!(arc.segments().last().unwrap().end().distance(b) < 1e-8);
        let tangent = Circle::tangent(b, a, Vec2::new(-1.0, 0.0)).unwrap();
        let arc = tangent.arc(a, b, Point::new(-5.0, 10.0), true);
        let end = arc.segments().last().unwrap();
        assert!(end.end().distance(b) < 1e-8);
        let (d, _) = derivatives(end, 1.0);
        assert!(d.y.abs() < 1e-8);
    }

    #[test]
    fn leo_geometry_fixed_and_anchored_tangency_stay_on_finite_edges() {
        let a = PathSeg::Line(Line::new((-30.0, 0.0), (30.0, 0.0)));
        let b = PathSeg::Line(Line::new((0.0, -30.0), (0.0, 30.0)));
        let choices = fixed(a, 0.6, b, 0.6, 10.0);
        assert_eq!(choices.len(), 4);
        for s in &choices {
            assert!((s.circle.center.distance(a.eval(s.first)) - 10.0).abs() < 1e-7);
            assert!((s.circle.center.distance(b.eval(s.second)) - 10.0).abs() < 1e-7);
        }
        assert!(fixed(a, 0.6, b, 0.6, 40.0).is_empty());
        assert_eq!(anchored(a, 2.0 / 3.0, b, 0.7).len(), 2);
    }

    #[test]
    fn leo_geometry_cubic_curvature_and_line_cubic_tangency() {
        let curve = PathSeg::Cubic(CubicBez::new((0.0, 20.0), (10.0, 20.0), (20.0, 10.0), (30.0, 20.0)));
        let circle = curvature(curve, 0.5).unwrap();
        let (d, _) = derivatives(curve, 0.5);
        assert!((circle.center - curve.eval(0.5)).dot(d).abs() < 1e-7);
        let line = PathSeg::Line(Line::new((-50.0, 0.0), (70.0, 0.0)));
        let candidates = fixed(line, 0.5, curve, 0.6, 10.0);
        assert!(!candidates.is_empty());
        for s in candidates {
            let tangent = derivatives(curve, s.second).0;
            assert!((s.circle.center - curve.eval(s.second)).dot(tangent).abs() < 1e-4);
        }
    }
}

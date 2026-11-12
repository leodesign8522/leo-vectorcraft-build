//! Click-driven circle and circular-arc construction tools for the canvas.
#[path = "leo_circle_geometry.rs"]
mod geometry;

use std::f64::consts::TAU;
use serde_json::{Value, json};
use vectorcraft_doc::NodeId;
use vectorcraft_geom::{BezPath, ParamCurve, PathSeg, Point};
use crate::guides::{DrawSnap, Leave};
use crate::{Action, Cursor, Mods, Overlay, PointerEvent, PointerKind, Tool, ToolContext, ToolKey};
use geometry::{Circle, Tangent, anchored, curvature, fixed};

const PREVIEW: [u8; 3] = [40, 142, 248];
const CONTACT: [u8; 3] = [248, 151, 34];

pub(crate) const IDS: [&str; 4] = ["leoCirclePoints", "leoTangentCircle", "leoCurvatureCircle", "leoArcPoints"];

#[derive(Clone, Copy)]
struct Pick {
    id: NodeId,
    sub: usize,
    index: usize,
    segment: PathSeg,
    t: f64,
}

impl Pick {
    fn at(cx: &ToolContext, p: Point) -> Option<Self> {
        let (id, sub, index, t) = crate::draw2::hit_segment(cx, p, cx.pick_tol().max(cx.tol(8.0)))?;
        let path = cx.doc.node(id)?.path_data()?.subpaths.get(sub)?;
        let cubic = path.segment(index);
        let segment = if path.segment_is_line(index) {
            PathSeg::Line(vectorcraft_geom::Line::new(cubic.p0, cubic.p3))
        } else {
            PathSeg::Cubic(cubic)
        };
        // A straight cubic with nonuniform handles uses a different parameter
        // from a Line. Use geometric distance to pick, cubic t when splitting.
        let t = if let PathSeg::Line(line) = segment {
            let direction = line.p1 - line.p0;
            if direction.hypot2() <= 1e-18 { return None; }
            ((cubic.eval(t) - line.p0).dot(direction) / direction.hypot2()).clamp(0.0, 1.0)
        } else { t };
        Some(Self { id, sub, index, segment, t })
    }

    fn key(self) -> (u64, usize, usize) { (self.id.0, self.sub, self.index) }
    fn point(self, t: f64) -> Point { self.segment.eval(t) }

    fn split(self, cx: &ToolContext, t: f64) -> Option<(Self, f64)> {
        if !(1e-6..1.0 - 1e-6).contains(&t) { return None; }
        let original = cx.doc.node(self.id)?.path_data()?.subpaths.get(self.sub)?.segment(self.index);
        let t = if matches!(self.segment, PathSeg::Line(_)) {
            use vectorcraft_geom::kurbo::ParamCurveNearest;
            original.nearest(self.point(t), 1e-9).t
        } else { t };
        (1e-6..1.0 - 1e-6).contains(&t).then_some((self, t))
    }
}

struct Preview {
    circle: Circle,
    arc: Option<BezPath>,
    contacts: Vec<Point>,
    cuts: Vec<(Pick, f64)>,
    alternatives: Option<(usize, usize)>,
}

impl Preview {
    fn new(circle: Circle, contacts: Vec<Point>) -> Self {
        Self { circle, arc: None, contacts, cuts: vec![], alternatives: None }
    }

    fn finish(self, cx: &ToolContext, insert: bool) -> Action {
        if let Some(arc) = self.arc {
            return Action::Exec("path.create".into(), json!({"d": arc.to_svg()}));
        }
        let r = self.circle.radius;
        let c = self.circle.center;
        let params = json!({"x": c.x - r, "y": c.y - r, "width": r * 2.0, "height": r * 2.0});
        if !insert || self.cuts.is_empty() {
            return Action::Exec("shape.ellipse".into(), params);
        }
        let mut cuts: Vec<_> = self.cuts.into_iter().filter_map(|(pick, t)| pick.split(cx, t)).collect();
        cuts.sort_by(|a, b| b.0.key().cmp(&a.0.key()).then_with(|| b.1.total_cmp(&a.1)));
        let mut commands = Vec::new();
        let mut prior: Option<((u64, usize, usize), f64)> = None;
        for (pick, t) in cuts {
            let key = pick.key();
            let local = if let Some((_, prev_t)) = prior.filter(|(prev, _)| *prev == key) {
                if (t - prev_t).abs() <= 1e-6 { continue; }
                t / prev_t
            } else { t };
            commands.push(json!({"command": "path.insertAnchor", "params": {
                "id": pick.id.0, "subpath": pick.sub, "segment": pick.index, "t": local
            }}));
            prior = Some((key, t));
        }
        commands.push(json!({"command": "shape.ellipse", "params": params}));
        Action::Exec("command.batch".into(), json!({"commands": commands, "atomic": true, "label": "Leo Design 相切圆"}))
    }
}

pub(crate) struct CircleTool {
    id: &'static str,
    points: Vec<Point>,
    first: Option<Pick>,
    hover: Option<Pick>,
    preview: Option<Preview>,
    snap: DrawSnap,
    alternate: bool,
    index: usize,
    revision: Option<(u64, u64)>,
    radius: f64,
    radius_input: String,
    cursor_point: Option<Point>,
    last_mods: Mods,
    cache: Option<((u64, usize, usize), f64, Vec<Tangent>)>,
}

impl CircleTool {
    pub fn new(id: &str) -> Self {
        let id = IDS.into_iter().find(|candidate| *candidate == id).unwrap_or(IDS[0]);
        Self { id, points: vec![], first: None, hover: None, preview: None, snap: DrawSnap::default(),
            alternate: false, index: 0, revision: None, radius: 24.0, radius_input: String::new(),
            cursor_point: None, last_mods: Mods::default(), cache: None }
    }

    fn reset(&mut self) {
        self.points.clear();
        self.first = None;
        self.hover = None;
        self.preview = None;
        self.snap.clear();
        self.radius_input.clear();
        self.index = 0;
        self.cache = None;
    }

    fn point_preview(&self, p: Point) -> Option<Preview> {
        match (self.id, self.points.as_slice()) {
            ("leoCirclePoints", [a]) if !self.alternate => Some(Preview::new(Circle::diameter(*a, p)?, vec![*a, p])),
            ("leoCirclePoints", [a, b]) => Some(Preview::new(Circle::through(*a, *b, p)?, vec![*a, *b, p])),
            ("leoArcPoints", [a, b]) => {
                let circle = if self.alternate { Circle::tangent(*b, *a, p - *b)? } else { Circle::through(*a, *b, p)? };
                let mut result = Preview::new(circle, if self.alternate { vec![*a, *b] } else { vec![*a, *b, p] });
                result.arc = Some(circle.arc(*a, *b, p, self.alternate));
                Some(result)
            }
            _ => None,
        }
    }

    fn tangent_preview(&mut self, mods: Mods) -> Option<Preview> {
        let (first, second) = (self.first?, self.hover?);
        let radius = if self.alternate || mods.shift { self.radius } else { 0.0 };
        if self.cache.as_ref().is_none_or(|(key, r, _)| *key != second.key() || *r != radius) {
            let solutions = if radius > 0.0 {
                fixed(first.segment, first.t, second.segment, second.t, radius)
            } else {
                anchored(first.segment, first.t, second.segment, second.t)
            };
            self.cache = Some((second.key(), radius, solutions));
            self.index = 0;
        }
        let mut sorted = self.cache.as_ref()?.2.clone();
        let dist = |s: &Tangent| {
            first.point(s.first).distance(first.point(first.t)) + second.point(s.second).distance(second.point(second.t))
        };
        sorted.sort_by(|a, b| dist(a).total_cmp(&dist(b)));
        let index = self.index % sorted.len().max(1);
        let hit = *sorted.get(index)?;
        let mut preview = Preview::new(hit.circle, vec![first.point(hit.first), second.point(hit.second)]);
        preview.cuts = vec![(first, hit.first), (second, hit.second)];
        preview.alternatives = Some((index + 1, sorted.len()));
        Some(preview)
    }

    fn update(&mut self, cx: &ToolContext, raw: Point, mods: Mods) -> Point {
        self.cursor_point = Some(raw);
        self.last_mods = mods;
        self.preview = match self.id {
            "leoTangentCircle" | "leoCurvatureCircle" => {
                self.hover = Pick::at(cx, raw);
                if self.id == "leoTangentCircle" { self.tangent_preview(mods) }
                else {
                    self.hover.and_then(|hit| {
                        let p = hit.point(hit.t);
                        let mut circle = curvature(hit.segment, hit.t)?;
                        if mods.shift { circle.center = p + (p - circle.center); }
                        Some(Preview::new(circle, vec![p]))
                    })
                }
            }
            _ => {
                let from = self.points.last().map(|p| Leave::segment(cx, *p, mods.shift));
                let mut point = self.snap.hover(cx, raw, &[], from.as_ref());
                self.hover = None;
                if point == raw && !mods.shift && !cx.snap_to_grid && !cx.snap_to_pixel && (cx.smart_guides || cx.snap_to_point) {
                    self.hover = Pick::at(cx, raw);
                    if let Some(hit) = self.hover { point = hit.point(hit.t); }
                }
                self.preview = self.point_preview(point);
                return point;
            }
        };
        raw
    }
}

impl Tool for CircleTool {
    fn id(&self) -> &'static str { self.id }

    fn pointer(&mut self, cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        if !ev.pos.is_finite() || !matches!(ev.kind, PointerKind::Move | PointerKind::Down) { return vec![]; }
        if self.revision != Some(cx.revision) { self.reset(); self.revision = Some(cx.revision); }
        let p = self.update(cx, ev.pos, ev.mods);
        if ev.kind != PointerKind::Down { return vec![]; }
        if let Some(preview) = self.preview.take() {
            self.reset();
            return vec![preview.finish(cx, ev.mods.alt)];
        }
        match self.id {
            "leoTangentCircle" if self.first.is_none() => { self.first = self.hover; }
            "leoCirclePoints" if self.points.len() < if self.alternate { 2 } else { 1 } => {
                if self.points.last().is_none_or(|last| last.distance(p) > 1e-8) { self.points.push(p); }
            }
            "leoArcPoints" if self.points.len() < 2 => {
                if self.points.last().is_none_or(|last| last.distance(p) > 1e-8) { self.points.push(p); }
            }
            _ => {}
        }
        vec![]
    }

    fn key(&mut self, cx: &ToolContext, key: ToolKey, mods: Mods) -> Vec<Action> {
        if self.revision != Some(cx.revision) { self.reset(); self.revision = Some(cx.revision); }
        match key {
            ToolKey::Escape => { self.reset(); return vec![]; }
            ToolKey::Tab if self.id == "leoTangentCircle" => {
                if mods.shift {
                    let count = self.cache.as_ref().map_or(1, |cache| cache.2.len().max(1));
                    self.index = (self.index % count + count - 1) % count;
                } else { self.index = self.index.wrapping_add(1); }
            }
            ToolKey::Tab => { self.alternate = !self.alternate; self.points.truncate(if self.id == "leoCirclePoints" && !self.alternate { 1 } else { 2 }); }
            ToolKey::Letter('f') => { self.alternate = !self.alternate; self.radius_input.clear(); }
            ToolKey::Digit(d) if d <= 9 && self.radius_input.len() < 7 => {
                self.radius_input.push(char::from(b'0' + d));
                if let Ok(radius) = self.radius_input.parse::<f64>() && radius > 0.0 {
                    self.radius = radius;
                    self.alternate = true;
                }
            }
            ToolKey::Enter if self.id == "leoTangentCircle" => {
                if !self.radius_input.is_empty() { self.radius_input.clear(); return vec![]; }
                if let Some(preview) = self.preview.take() {
                    self.reset();
                    return vec![preview.finish(cx, mods.alt)];
                }
            }
            _ => return vec![],
        }
        if let Some(p) = self.cursor_point { self.update(cx, p, self.last_mods); }
        vec![]
    }

    fn claims_key(&self, _cx: &ToolContext, key: ToolKey) -> bool {
        matches!(key, ToolKey::Tab) && self.id != "leoCurvatureCircle"
            || self.id == "leoTangentCircle" && matches!(key, ToolKey::Digit(_) | ToolKey::Letter('f') | ToolKey::Enter)
    }

    fn ends_on_escape(&self, _cx: &ToolContext) -> bool { self.first.is_some() || !self.points.is_empty() }
    fn hover_ended(&mut self, _cx: &ToolContext) { self.hover = None; self.preview = None; self.snap.clear(); self.cursor_point = None; }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: Mods) -> Cursor { Cursor::Crosshair }

    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {
        if self.revision != Some(cx.revision) { return vec![]; }
        let mark = |p, color| Overlay::CurvePoint { p, color, filled: false, size: 9.0 };
        let mut out = self.snap.guides().to_vec();
        out.extend(self.points.iter().map(|p| mark(*p, PREVIEW)));
        if let Some(first) = self.first { out.push(mark(first.point(first.t), CONTACT)); }
        if let Some(hit) = self.hover { out.push(mark(hit.point(hit.t), CONTACT)); }
        if let Some(preview) = &self.preview {
            let circle = preview.circle;
            let path = preview.arc.clone().unwrap_or_else(|| circle.path(0.0, TAU));
            out.push(Overlay::Path { path, color: PREVIEW, width: 1.7, dashed: false });
            out.push(mark(circle.center, PREVIEW));
            out.extend(preview.contacts.iter().map(|p| mark(*p, CONTACT)));
            let alternatives = preview.alternatives.map_or(String::new(), |(n, total)| format!("  {n}/{total} · Tab"));
            out.push(Overlay::Measure { p: self.cursor_point.unwrap_or(circle.center),
                text: format!("R {}{alternatives}", cx.unit.format(circle.radius)) });
        } else if let Some(p) = self.cursor_point {
            let text = match self.id {
                "leoCirclePoints" if self.alternate => "三点定圆 · Tab 切换",
                "leoCirclePoints" => "两点定直径 · Tab 切换",
                "leoArcPoints" if self.alternate => "端点切线 · Tab 切换",
                "leoArcPoints" => "三点定弧 · Tab 切换",
                "leoCurvatureCircle" => "选择曲线 · Shift 翻转",
                _ if self.alternate => "固定半径 · 数字输入 pt · F 切换",
                _ => "依次点选两条边 · Shift 固定半径",
            };
            out.push(Overlay::Measure { p, text: text.into() });
        }
        out
    }

    fn options(&self) -> Value {
        if self.id == "leoTangentCircle" {
            json!({"radius": self.radius, "fixedRadius": self.alternate})
        } else { Value::Null }
    }

    fn set_option(&mut self, key: &str, value: &Value) {
        if self.id != "leoTangentCircle" { return; }
        match key {
            "radius" => {
                if let Some(radius) = value.as_f64().filter(|r| r.is_finite() && *r > 0.0) {
                    self.radius = radius;
                    self.cache = None;
                    self.preview = None;
                }
            }
            "fixedRadius" => {
                if let Some(fixed) = value.as_bool() { self.alternate = fixed; self.preview = None; }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{cx, paint};
    use vectorcraft_doc::{Document, Selection};

    #[test]
    fn leo_geometry_canvas_clicks_preview_commit_and_escape() {
        let doc = Document::new(600.0, 600.0);
        let selection = Selection::default();
        let paint = paint();
        let context = ToolContext { smart_guides: false, snap_to_point: false, ..cx(&doc, &selection, &paint) };
        let mut circle = crate::create("leoCirclePoints");
        assert_eq!(circle.id(), "leoCirclePoints");
        circle.pointer(&context, &PointerEvent::new(PointerKind::Down, 100.0, 100.0));
        circle.pointer(&context, &PointerEvent::new(PointerKind::Move, 200.0, 100.0));
        assert!(circle.overlays(&context).iter().any(|o| matches!(o, Overlay::Path { .. })));
        assert_eq!(circle.pointer(&context, &PointerEvent::new(PointerKind::Down, 200.0, 100.0)),
            [Action::Exec("shape.ellipse".into(), json!({"x": 100.0, "y": 50.0, "width": 100.0, "height": 100.0}))]);
        circle.pointer(&context, &PointerEvent::new(PointerKind::Down, 100.0, 100.0));
        assert!(circle.ends_on_escape(&context));
        circle.key(&context, ToolKey::Escape, Mods::default());
        assert!(!circle.ends_on_escape(&context));
    }
}

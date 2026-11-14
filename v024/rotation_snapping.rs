/// R-tool rotation targets are edge directions, not snapped mouse positions.
/// Geometry is immutable for the drag; the current screen controls acquisition.
pub(crate) struct LeoRotationSnap {
    origin: Point,
    sources: Vec<Line>,
    targets: Vec<(f64, LeoExtension)>,
    rulers: Vec<LeoExtension>,
    radius: f64,
    held: Cell<Option<RotationHit>>,
}

#[derive(Clone,Copy)]
struct RotationHit {
    source: Line,
    target: LeoExtension,
    angle: f64,
    ruler: bool,
}

// A long selected edge can be many pixels from a guide at its far end
// even when the edge is only a fraction of a degree from coinciding.
const RULER_ACQUIRE_DEG: f64 = 1.0;

fn rotation_overlap(a: Point, b: Point, target: LeoExtension) -> Option<(Point, Point)> {
    if (a - target.at).cross(target.dir).abs() > 1e-7
        || (b - target.at).cross(target.dir).abs() > 1e-7
    {
        return None;
    }
    let (a, b) = ((a - target.at).dot(target.dir), (b - target.at).dot(target.dir));
    let from = a.min(b).max(0.0);
    let to = a.max(b).min(target.at.distance(target.end));
    (to - from > 1e-7).then(|| (target.at + target.dir * from, target.at + target.dir * to))
}

fn rotation_guide_segment(cx: &ToolContext, guide: &vectorcraft_doc::Guide, origin: Point, radius: f64) -> Option<LeoExtension> {
    let (along, across) = if guide.vertical { (origin.y, origin.x) } else { (origin.x, origin.y) };
    if (guide.pos - across).abs() > radius + cx.tol(ACQUIRE_PX) {
        return None;
    }
    let (lo, hi) = (along - radius, along + radius);
    let (lo, hi) = match cx.doc.guide_span(guide) {
        Some((from, to)) => (lo.max(from), hi.min(to)),
        None if guide.artboard.is_none() => (lo, hi),
        None => return None,
    };
    if hi <= lo {
        return None;
    }
    let point = |along| if guide.vertical { Point::new(guide.pos, along) } else { Point::new(along, guide.pos) };
    LeoExtension::new(point(lo), point(hi))
}

fn push_rotation_line(guides: &mut Vec<Overlay>, a: Point, b: Point) {
    let mut path = BezPath::new();
    path.move_to(a);
    path.line_to(b);
    guides.push(Overlay::Path { path, color: [0x20, 0x24, 0x2b], width: 3.5, dashed: true });
    guides.push(Overlay::Line { a, b, color: EXTENSION_HIT, dashed: true });
}

fn line_angle(d: Vec2) -> f64 {d.y.atan2(d.x).rem_euclid(std::f64::consts::PI)}
fn turn_near(angle: f64,reference: f64) -> f64 {
    reference+(angle-reference+std::f64::consts::FRAC_PI_2).rem_euclid(std::f64::consts::PI)-std::f64::consts::FRAC_PI_2
}

impl LeoRotationSnap {
    pub(crate) fn new(cx: &ToolContext,origin: Point) -> Option<Self> {
        if !(cx.smart_guides || cx.snap_to_point) || !origin.is_finite() {return None;}
        // A full anchor selection is a rigid path too. Partial selections keep
        // the native point-transform behaviour rather than treating fixed edges as moving.
        for (id,set) in &cx.selection.anchors {
            let path=cx.doc.node(*id)?.path_data()?;
            if path.anchors().any(|(si,ai,_)| !set.contains(&(si,ai))) {return None;}
        }
        let selected: HashSet<NodeId>=cx.selection.objects.iter().copied().collect();
        let mut pending: Vec<(&Arc<Node>,bool)>=cx.doc.layers.iter().rev().map(|n| (n,false)).collect();
        let mut sources=Vec::new();
        while let Some((node,inherited))=pending.pop() {
            if !node.visible {continue;}
            let include=inherited || selected.contains(&node.id);
            if include && let Some(path)=node.path_data() {
                for sp in &path.subpaths {for i in 0..sp.segment_count() {
                    if !sp.segment_is_line(i) {continue;}
                    let c=sp.segment(i);
                    if c.p0.is_finite() && c.p3.is_finite() && c.p0.distance(c.p3)>1e-9 {sources.push(Line::new(c.p0,c.p3));}
                }}
            }
            if let Some(children)=node.children() {
                pending.extend(children.get(usize::from(node.extras().shaper.is_some())..).unwrap_or_default().iter().rev().map(|n| (n,include)));
            }
        }
        if sources.is_empty() {return None;}
        let radius=sources.iter().flat_map(|l| [l.p0,l.p1]).map(|p| p.distance(origin)).fold(0.0,f64::max);
        let scene=LeoScene::new(cx,cx.selection,false);
        let index=LeoExtensionIndex::new(&scene,origin);
        let mut targets: Vec<_>=index.buckets.into_iter().flat_map(|b| b.entries.into_iter().map(|(_,l)| (line_angle(l.dir),l))).collect();
        targets.sort_by(|a,b| a.0.total_cmp(&b.0));
        let rulers = if cx.leo_ruler_guides_visible {
            cx.doc.ruler_guides().iter()
                .filter(|guide| cx.doc.guide_shown(guide))
                .filter_map(|guide| rotation_guide_segment(cx, guide, origin, radius))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if targets.is_empty() && rulers.is_empty() { return None; }
        Some(Self { origin, sources, targets, rulers, radius, held: Cell::new(None) })
    }

    pub(crate) fn snap(&self,cx: &ToolContext,raw: f64,pointer: Point,shift: bool) -> Option<(Affine,f64,Vec<Overlay>)> {
        if shift || !raw.is_finite() || !pointer.is_finite() {
            self.held.set(None);return None;
        }
        let lever=self.radius.max(pointer.distance(self.origin));
        if !lever.is_finite() || lever<=1e-9 {self.held.set(None);return None;}
        let acquire=cx.tol(ACQUIRE_PX);let release=acquire*RELEASE_MULTIPLIER;
        let travel=|angle: f64| 2.0*lever*((turn_near(angle,raw)-raw)*0.5).sin().abs();
        let ruler_acquire = RULER_ACQUIRE_DEG.to_radians();
        let angle_gap = |angle| (turn_near(angle, raw) - raw).abs();
        let mut hit = self.held.get().filter(|hit| {
            if hit.ruler { angle_gap(hit.angle) <= ruler_acquire * RELEASE_MULTIPLIER }
            else { travel(hit.angle) <= release }
        });
        if hit.is_none() {
            let width=2.0*(acquire/(2.0*lever)).clamp(0.0,1.0).asin();
            let mut best: Option<(f64, f64, RotationHit)> = None;
            let mut best_ruler: Option<(f64, RotationHit)> = None;
            let raw_matrix = self.matrix(raw);
            for source in &self.sources {
                if !LeoExtension::new(raw_matrix*source.p0,raw_matrix*source.p1).is_some_and(|l| l.reference_in_view(cx.screen.as_ref())) {continue;}
                let source_angle=line_angle(source.p1-source.p0);
                for &target in &self.rulers {
                    let angle = turn_near(line_angle(target.dir) - source_angle, raw);
                    let gap = angle_gap(angle);
                    if gap > ruler_acquire || best_ruler.as_ref().is_some_and(|(best, _)| gap >= *best) {
                        continue;
                    }
                    let matrix = self.matrix(angle);
                    let Some((a, b)) = rotation_overlap(matrix * source.p0, matrix * source.p1, target) else {
                        continue;
                    };
                    if LeoExtension::new(a, b).is_some_and(|overlap| overlap.reference_in_view(cx.screen.as_ref())) {
                        best_ruler = Some((gap, RotationHit { source: *source, target, angle, ruler: true }));
                    }
                }
                let wanted=line_angle(Vec2::new((source_angle+raw).cos(),(source_angle+raw).sin()));
                // Three wrapped intervals avoid dropping targets across 0/180 degrees.
                for center in [wanted-std::f64::consts::PI,wanted,wanted+std::f64::consts::PI] {
                    let lo=self.targets.partition_point(|(a,_)| *a<center-width);
                    let hi=self.targets.partition_point(|(a,_)| *a<=center+width);
                    for &(angle,target) in self.targets.get(lo..hi).unwrap_or_default() {
                        if !target.reference_in_view(cx.screen.as_ref()) {continue;}
                        let angle=turn_near(angle-source_angle,raw);
                        let distance=travel(angle);if distance>acquire {continue;}
                        let matrix=self.matrix(angle);
                        let p=matrix*source.p0;
                        let offset=(p-target.at).cross(target.dir).abs();
                        if best.as_ref().is_none_or(|b| (distance,offset)<(b.0,b.1)) {
                            best=Some((distance,offset,RotationHit {source:*source,target,angle,ruler:false}));
                        }
                    }
                }
            }
            hit = match (best_ruler, best) {
                (Some((gap, _)), Some((_, _, path))) if angle_gap(path.angle) < gap => Some(path),
                (Some((_, ruler)), _) => Some(ruler),
                (None, path) => path.map(|(_, _, hit)| hit),
            };
        }
        self.held.set(hit);
        let hit=hit?;
        let angle=turn_near(hit.angle,raw);let matrix=self.matrix(angle);
        let a=matrix*hit.source.p0;let b=matrix*hit.source.p1;
        let collinear=(a-hit.target.at).cross(hit.target.dir).abs()<=1e-7;
        let mut guides=vec![];
        if cx.alignment_guides {
            // Draw real source/target segments; never generate an infinite dashed path.
            let span=cx.tol(512.0);
            if hit.ruler {
                if let Some((p, q)) = rotation_overlap(a, b, hit.target) {
                    push_rotation_line(&mut guides, p, q);
                }
            } else {
                for (p, q) in [(a, b), (hit.target.at, hit.target.end)] {
                    let length = p.distance(q);
                    let q = if length > span { p + (q - p) * (span / length) } else { q };
                    push_rotation_line(&mut guides, p, q);
                }
            }
        }
        if cx.anchor_path_labels {
            let at = if hit.ruler {
                let (from, to) = rotation_overlap(a, b, hit.target).unwrap_or((a, b));
                let edge = to - from;
                from + edge * ((pointer - from).dot(edge) / edge.hypot2()).clamp(0.0, 1.0)
            } else if a.distance(self.origin) > b.distance(self.origin) { a } else { b };
            guides.push(Overlay::Label {p:at,text:if hit.ruler {"旋转吸附 · 参考线"} else if collinear {"旋转吸附 · 共线"} else {"旋转吸附 · 平行"}.into(),color:EXTENSION_HIT});
        }
        Some((matrix,angle,guides))
    }

    fn matrix(&self,angle: f64) -> Affine {
        Affine::translate(self.origin.to_vec2())*Affine::rotate(angle)*Affine::translate(-self.origin.to_vec2())
    }
}


#[cfg(test)]
mod leo_rotation_ruler_tests {
    use super::*;
    use crate::testutil::{leo_cx, paint};
    use vectorcraft_doc::{Appearance, Guide};
    use vectorcraft_geom::shapes;

    #[test]
    fn leo_snapping_engine_rotation_long_edge_lands_on_locked_ruler() {
        let mut doc = Document::new(1800.0, 1200.0);
        let layer = doc.layers[0].id;
        let source = doc.alloc_id();
        let pivot = Point::new(1550.0, 600.0);
        let far = Point::new(250.0, 600.0);
        let near = Point::new(350.0, 600.0);
        doc.insert(Some(layer), 0, Node::path(
            source, shapes::rectangle(Rect::new(far.x, 450.0, pivot.x, pivot.y)), Appearance::default_art(),
        )).unwrap();
        let guide = doc.add_ruler_guide(&Guide { locked: true, ..Guide::new(false, pivot.y) });
        doc.add_ruler_guide(&Guide::new(false, pivot.y + 80.0));
        let mut selection = Selection::default();
        selection.set([source]);
        let paint = paint();
        // The guide is visible and locked, as when View > Lock Guides is active.
        let cx = ToolContext { guides: false, ..leo_cx(&doc, &selection, &paint) };
        let snap = LeoRotationSnap::new(&cx, pivot).unwrap();
        let raw = 0.6_f64.to_radians();
        let cursor = snap.matrix(raw) * near;
        assert!(cursor.distance(near) > ACQUIRE_PX);
        let (matrix, angle, guides) = snap.snap(&cx, raw, cursor, false).unwrap();
        assert!(angle.abs() < 1e-10 && (matrix * pivot).distance(pivot) < 1e-8);
        assert!((matrix * far).distance(far) < 1e-8);
        assert!(guides.iter().any(|o| matches!(o, Overlay::Line { a, b, color, .. }
            if (a.y - pivot.y).abs() < 1e-8 && (b.y - pivot.y).abs() < 1e-8
                && a.distance(*b) > 1000.0 && *color == EXTENSION_HIT)));
        assert!(guides.iter().any(|o| matches!(o, Overlay::Label { text, .. } if text == "旋转吸附 · 参考线")));
        let released = 1.65_f64.to_radians();
        assert!(snap.snap(&cx, released, snap.matrix(released) * near, false).is_none());
        assert!(snap.snap(&cx, raw, cursor, true).is_none());

        let hidden_view = ToolContext { leo_ruler_guides_visible: false, ..cx };
        assert!(LeoRotationSnap::new(&hidden_view, pivot).is_none());
        doc.node_mut(guide).unwrap().visible = false;
        let cx = leo_cx(&doc, &selection, &paint);
        let snap = LeoRotationSnap::new(&cx, pivot).unwrap();
        assert!(snap.snap(&cx, raw, cursor, false).is_none(), "parallel ruler must not fake contact");
    }
}

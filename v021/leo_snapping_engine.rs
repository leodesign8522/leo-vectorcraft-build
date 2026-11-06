//! Leo Design Snapping Engine v1.2: shared geometry for node dragging and object translation.
//! Node drags use exact anchors and finite path geometry; object moves add selected bounds,
//! multi-object/group source anchors, alignment axes and artboard edges. Capture is in SCREEN px.
use super::*;
use std::cell::Cell;

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
    /// Gather real path geometry plus the four finite edges of each artboard.
    /// Artboards are painted as white rectangles but are NOT Path nodes: without
    /// these edges a dragged node beside the page had no geometric snap target.
    pub(in crate::guides) fn leo_for_anchor_drag(cx: &ToolContext) -> Self {
        let mut targets = Self::for_anchor_drag(cx.doc, cx.selection).styled(cx);
        if cx.snap_to_point { targets = targets.with_rulers(cx); }
        if cx.spacing_guides { targets.boxes = super::object_boxes(cx.doc, &cx.selection.objects); }
        push_artboard_edges(cx.doc, &mut targets);
        targets
    }

    /// Full geometry target set for a translated selection. The selected objects are excluded
    /// for a normal move; pass an empty slice for an Alt-copy so the original remains a target.
    pub(in crate::guides) fn leo_for_object_move(cx: &ToolContext, exclude: &[NodeId]) -> Self {
        let mut selection = Selection::default();
        selection.set(exclude.iter().copied());
        let mut targets = Self::for_anchor_drag(cx.doc, &selection).styled(cx);
        if cx.snap_to_point { targets = targets.with_rulers(cx); }
        push_artboard_edges(cx.doc, &mut targets);
        targets
    }

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

fn push_artboard_edges(doc: &Document, targets: &mut Targets) {
    for artboard in &doc.artboards {
        let r = artboard.rect;
        let tl = Point::new(r.x0, r.y0);
        let tr = Point::new(r.x1, r.y0);
        let br = Point::new(r.x1, r.y1);
        let bl = Point::new(r.x0, r.y1);
        for (a, b) in [(tl, tr), (tr, br), (br, bl), (bl, tl)] {
            targets.segments.push((Rect::from_points(a, b), PathSeg::Line(Line::new(a, b))));
        }
    }
}

fn push_unique(points: &mut Vec<Point>, p: Point) {
    if p.is_finite() && points.iter().all(|q| q.distance(p) > 1e-7) {
        points.push(p);
    }
}

fn selected_anchor_points(cx: &ToolContext) -> Vec<Point> {
    fn visit(nodes: &[Arc<Node>], selected: &HashSet<NodeId>, inherited: bool, out: &mut Vec<Point>) {
        for node in nodes.iter().filter(|n| n.visible) {
            let include = inherited || selected.contains(&node.id);
            if include && out.len() < 256 {
                if let Some(path) = node.path_data() {
                    for (_, _, anchor) in path.anchors().take(256 - out.len()) {
                        push_unique(out, anchor.p);
                    }
                }
            }
            if let Some(children) = node.children() {
                // Shaper Groups keep their hidden source shapes as the first child; only
                // the visible generated face participates in object-move snap features.
                let shown = children.get(usize::from(node.extras().shaper.is_some())..).unwrap_or_default();
                visit(shown, selected, include, out);
            }
        }
    }
    let selected: HashSet<NodeId> = cx.selection.objects.iter().copied().collect();
    let mut points = Vec::new();
    visit(&cx.doc.layers, &selected, false, &mut points);
    points
}

fn rect_features(r: Rect) -> [Point; 9] {
    let cx = (r.x0 + r.x1) * 0.5;
    let cy = (r.y0 + r.y1) * 0.5;
    [
        Point::new(r.x0, r.y0), Point::new(cx, r.y0), Point::new(r.x1, r.y0),
        Point::new(r.x1, cy), Point::new(r.x1, r.y1), Point::new(cx, r.y1),
        Point::new(r.x0, r.y1), Point::new(r.x0, cy), Point::new(cx, cy),
    ]
}

#[derive(Clone, Copy, Debug)]
enum MoveTarget {
    Point { point: Point, kind: Kind },
    Segment(usize),
}

#[derive(Clone, Copy, Debug)]
struct MoveLock {
    source: Point,
    target: MoveTarget,
}

#[derive(Clone, Copy, Debug)]
struct AxisLock {
    source: Point,
    value: f64,
    from: Point,
    kind: Kind,
}

/// The shared object-translation snapper. It uses the same anchor/segment geometry and pixel
/// hysteresis as node drags, plus object bounds alignment. All points in a selection receive one
/// translation, so groups and multi-object selections never deform.
pub(crate) struct ObjectMoveSnap {
    bounds: Option<Rect>,
    sources: Vec<Point>,
    targets: Targets,
    copy_targets: Targets,
    lock: Cell<Option<MoveLock>>,
    x_lock: Cell<Option<AxisLock>>,
    y_lock: Cell<Option<AxisLock>>,
    last_copy: Cell<Option<bool>>,
}

impl ObjectMoveSnap {
    pub(crate) fn new(cx: &ToolContext, bounds: Option<Rect>) -> Self {
        let mut sources = Vec::new();
        if let Some(bounds) = bounds {
            for p in rect_features(bounds) {
                push_unique(&mut sources, p);
            }
        }
        for p in selected_anchor_points(cx) {
            push_unique(&mut sources, p);
        }
        Self {
            bounds,
            sources,
            targets: Targets::leo_for_object_move(cx, &cx.selection.objects),
            copy_targets: Targets::leo_for_object_move(cx, &[]),
            lock: Cell::new(None),
            x_lock: Cell::new(None),
            y_lock: Cell::new(None),
            last_copy: Cell::new(None),
        }
    }

    fn reset_locks(&self) {
        self.lock.set(None);
        self.x_lock.set(None);
        self.y_lock.set(None);
    }

    fn targets(&self, copy: bool) -> &Targets {
        if copy { &self.copy_targets } else { &self.targets }
    }

    pub(crate) fn snap(&self, cx: &ToolContext, start: Point, d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {
        self.snap_with_tolerance(start, d, copy, cx.tol(ACQUIRE_PX), cx.tol(ACQUIRE_PX * RELEASE_MULTIPLIER), None)
    }

    pub(crate) fn snap_along(&self, cx: &ToolContext, start: Point, d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {
        if d.hypot() <= 1e-9 { return (d, vec![]); }
        self.snap_with_tolerance(start, d, copy, cx.tol(ACQUIRE_PX), cx.tol(ACQUIRE_PX * RELEASE_MULTIPLIER), Some(d))
    }

    fn snap_with_tolerance(
        &self, start: Point, d: Vec2, copy: bool, acquire: f64, release: f64, direction: Option<Vec2>,
    ) -> (Vec2, Vec<Overlay>) {
        if !d.x.is_finite() || !d.y.is_finite() || !acquire.is_finite() || !release.is_finite() || acquire <= 0.0 || release < acquire {
            self.reset_locks();
            return (d, vec![]);
        }
        if self.last_copy.replace(Some(copy)).is_some_and(|previous| previous != copy) {
            self.reset_locks();
        }
        let targets = self.targets(copy);
        if direction.is_some() {
            self.x_lock.set(None);
            self.y_lock.set(None);
            if self.lock.get().is_some_and(|lock| lock.source != start || matches!(lock.target, MoveTarget::Segment(_))) {
                self.lock.set(None);
            }
        }
        let mut sources = self.sources.clone();
        push_unique(&mut sources, start);
        if sources.is_empty() {
            return (d, vec![]);
        }
        // Shift constrains the grabbed point to its drag line. Other moving anchors must not
        // pull the whole selection sideways; bounds features remain available for edge alignment.
        let (geometry_sources, alignment_sources) = if direction.is_some() {
            let bounds_points = self.bounds.map(|r| rect_features(r).to_vec()).unwrap_or_else(|| vec![start]);
            (vec![start], bounds_points)
        } else {
            (sources.clone(), sources.clone())
        };

        // A captured path stays locked until release, unless a true node enters the
        // acquisition radius. This preserves the node-over-path priority of node drags.
        if let Some(mut lock) = self.lock.get() {
            if matches!(lock.target, MoveTarget::Segment(_) | MoveTarget::Point { kind: Kind::Center, .. }) {
                if let Some(anchor) = find_move_geometry(targets, &geometry_sources, self.bounds, d, acquire, direction)
                    .filter(|candidate| matches!(candidate.target, MoveTarget::Point { kind: Kind::Anchor, .. }))
                {
                    lock = anchor;
                    self.lock.set(Some(lock));
                }
            }
            if let Some((next, q, label, color)) = resolve_move_lock(targets, lock, d, release, direction) {
                self.x_lock.set(None);
                self.y_lock.set(None);
                return (next, move_hit_guides(targets, q, label, color));
            }
            self.lock.set(None);
        }

        // True anchors first, then finite paths/Bezier curves. Alignment never steals a geometry hit.
        if let Some(lock) = find_move_geometry(targets, &geometry_sources, self.bounds, d, acquire, direction) {
            if let Some((next, q, label, color)) = resolve_move_lock(targets, lock, d, release, direction) {
                self.lock.set(Some(lock));
                self.x_lock.set(None);
                self.y_lock.set(None);
                return (next, move_hit_guides(targets, q, label, color));
            }
        }

        self.lock.set(None);
        snap_move_alignment(targets, &alignment_sources, self.bounds, d, acquire, release, &self.x_lock, &self.y_lock, direction, Some(start))
    }
}

fn move_hit_guides(targets: &Targets, q: Point, label: &'static str, color: [u8; 3]) -> Vec<Overlay> {
    let mut guides = Vec::new();
    if targets.style.lines {
        guides.push(Overlay::CurvePoint { p: q, color, filled: false, size: 8.0 });
    }
    if targets.style.labels {
        guides.push(Overlay::Label { p: q, text: label.into(), color });
    }
    guides
}

fn resolve_move_lock(
    targets: &Targets, lock: MoveLock, d: Vec2, release: f64, direction: Option<Vec2>,
) -> Option<(Vec2, Point, &'static str, [u8; 3])> {
    let current = lock.source + d;
    let (q, label, color) = match lock.target {
        MoveTarget::Point { point, kind } => {
            if current.distance(point) > release { return None; }
            let (label, color) = match kind {
                Kind::Anchor => ("anchor", ANCHOR_HIT),
                Kind::Center => ("center", PATH_HIT),
                _ => return None,
            };
            (point, label, color)
        }
        MoveTarget::Segment(index) => {
            let (_, segment) = targets.segments.get(index)?;
            let nearest = segment.nearest(current, 1e-8);
            if !nearest.distance_sq.is_finite() || nearest.distance_sq > release * release { return None; }
            (segment.eval(nearest.t), "path", PATH_HIT)
        }
    };
    let next = if let Some(dir) = direction.filter(|u| u.hypot() > 1e-9) {
        let u = dir / dir.hypot();
        let shift = (q - current).dot(u);
        let moved = current + u * shift;
        if moved.distance(q) > release { return None; }
        d + u * shift
    } else {
        q - lock.source
    };
    Some((next, q, label, color))
}

fn find_move_geometry(
    targets: &Targets, sources: &[Point], bounds: Option<Rect>, d: Vec2, acquire: f64, direction: Option<Vec2>,
) -> Option<MoveLock> {
    let moving = bounds.map(|b| b + d).or_else(|| {
        let mut it = sources.iter().map(|p| *p + d);
        let first = it.next()?;
        Some(it.fold(Rect::from_points(first, first), |r, p| r.union_pt(p)))
    })?;
    let search = moving.inflate(acquire, acquire);
    let points: Vec<(Point, Kind)> = targets.points.iter()
        .filter(|(p, k)| search.contains(*p) && matches!(k, Kind::Anchor | Kind::Center))
        .map(|(p, k)| (*p, *k)).collect();
    let segments: Vec<usize> = targets.segments.iter().enumerate()
        .filter(|(_, (b, _))| b.inflate(acquire, acquire).intersect(search).area() > 0.0 || search.contains(b.center()))
        .map(|(i, _)| i).collect();

    let mut best: Option<(u8, f64, MoveLock)> = None;
    for source in sources {
        let current = *source + d;
        for (point, kind) in &points {
            let distance = current.distance(*point);
            if distance > acquire { continue; }
            let (priority, label) = match kind {
                Kind::Anchor => (0, "anchor"),
                Kind::Center => (2, "center"),
                _ => continue,
            };
            let _ = label;
            let candidate = MoveLock { source: *source, target: MoveTarget::Point { point: *point, kind: *kind } };
            let score = distance;
            if best.is_none_or(|b| (priority, score).lt(&(b.0, b.1))) {
                best = Some((priority, score, candidate));
            }
        }
        if direction.is_some() { continue; }
        for index in &segments {
            let (bounds, segment) = &targets.segments[*index];
            if !bounds.inflate(acquire, acquire).contains(current) { continue; }
            let nearest = segment.nearest(current, 1e-8);
            if !nearest.distance_sq.is_finite() || nearest.distance_sq > acquire * acquire { continue; }
            let candidate = MoveLock { source: *source, target: MoveTarget::Segment(*index) };
            let score = nearest.distance_sq.sqrt();
            if best.is_none_or(|b| (1u8, score).lt(&(b.0, b.1))) {
                best = Some((1, score, candidate));
            }
        }
    }
    best.map(|(_, _, lock)| lock)
}

fn snap_move_alignment(
    targets: &Targets, sources: &[Point], bounds: Option<Rect>, mut d: Vec2, acquire: f64, release: f64,
    x_lock: &Cell<Option<AxisLock>>, y_lock: &Cell<Option<AxisLock>>, direction: Option<Vec2>, ruler_source: Option<Point>,
) -> (Vec2, Vec<Overlay>) {
    if let Some(dir) = direction.filter(|u| u.hypot() > 1e-9) {
        let u = dir / dir.hypot();
        let mut best: Option<(f64, bool, AxisLock)> = None;
        for source in sources {
            let current = *source + d;
            for (is_x, lines, v, component) in [
                (true, &targets.xs, current.x, u.x),
                (false, &targets.ys, current.y, u.y),
            ] {
                if component.abs() <= 1e-9 { continue; }
                for (value, from, kind) in super::lines_near(lines, v, acquire) {
                    let across = (value - v).abs();
                    if across > acquire { continue; }
                    let amount = (value - v) / component;
                    let candidate = AxisLock { source: *source, value: *value, from: *from, kind: *kind };
                    if best.is_none_or(|b| amount.abs() < b.0.abs()) {
                        best = Some((amount, is_x, candidate));
                    }
                }
                for ruler in targets.rulers.iter().filter(|r| r.vertical == is_x) {
                    if let Some((value, from, kind)) = ruler.line_at(current, acquire) {
                        let amount = (value - v) / component;
                        let candidate = AxisLock { source: *source, value, from, kind };
                        if best.is_none_or(|b| amount.abs() < b.0.abs()) {
                            best = Some((amount, is_x, candidate));
                        }
                    }
                }
            }
        }
        if let Some(source) = ruler_source {
            let current = source + d;
            for ruler in &targets.rulers {
                let Some((value, from, kind)) = ruler.line_at(current, acquire) else { continue };
                let component = if ruler.vertical { u.x } else { u.y };
                if component.abs() <= 1e-9 { continue; }
                let axis = if ruler.vertical { current.x } else { current.y };
                let amount = (value - axis) / component;
                let candidate = AxisLock { source, value, from, kind };
                if best.is_none_or(|b| amount.abs() < b.0.abs()) {
                    best = Some((amount, ruler.vertical, candidate));
                }
            }
        }
        if let Some((amount, is_x, lock)) = best {
            d += u * amount;
            let moved = bounds.map(|b| b + d);
            let mut guides = Vec::new();
            if targets.style.lines {
                if let Some(r) = moved {
                    let (a, b) = if is_x {
                        (Point::new(lock.value, r.y0.min(lock.from.y)), Point::new(lock.value, r.y1.max(lock.from.y)))
                    } else {
                        (Point::new(r.x0.min(lock.from.x), lock.value), Point::new(r.x1.max(lock.from.x), lock.value))
                    };
                    guides.push(Overlay::Line { a, b, color: targets.style.color, dashed: false });
                }
            }
            if targets.style.labels {
                guides.push(Overlay::Label { p: lock.from, text: if lock.kind == Kind::Guide { "guide" } else { "align" }.into(), color: targets.style.color });
            }
            return (d, guides);
        }
        if let Some(r) = bounds.map(|b| b + d) {
            if !targets.boxes.is_empty() {
                let (shift, guides) = targets.snap_rects_along(&[r], u, acquire);
                return (d + shift, guides);
            }
        }
        return (d, vec![]);
    }

    let mut guides = Vec::new();
    let mut x = x_lock.get();
    if x.is_some_and(|l| (l.source.x + d.x - l.value).abs() > release) { x = None; }
    if x.is_none() {
        x = sources.iter().flat_map(|source| {
            super::lines_near(&targets.xs, source.x + d.x, acquire).iter().filter_map(move |(value, from, kind)| {
                let distance = (value - (source.x + d.x)).abs();
                (distance <= acquire).then_some((distance, AxisLock { source: *source, value: *value, from: *from, kind: *kind }))
            })
        }).chain(sources.iter().flat_map(|source| {
            let current = *source + d;
            targets.rulers.iter().filter(|r| r.vertical).filter_map(move |ruler| {
                ruler.line_at(current, acquire).map(|(value, from, kind)| ((value - current.x).abs(), AxisLock { source: *source, value, from, kind }))
            })
        })).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, l)| l);
    }
    if let Some(lock) = x {
        d.x = lock.value - lock.source.x;
        x_lock.set(Some(lock));
        if targets.style.lines {
            if let Some(r) = bounds.map(|b| b + d) {
                guides.push(Overlay::Line {
                    a: Point::new(lock.value, r.y0.min(lock.from.y)),
                    b: Point::new(lock.value, r.y1.max(lock.from.y)),
                    color: targets.style.color, dashed: false,
                });
            }
        }
        if targets.style.labels {
            guides.push(Overlay::Label { p: lock.from, text: if lock.kind == Kind::Guide { "guide" } else { "align" }.into(), color: targets.style.color });
        }
    } else {
        x_lock.set(None);
    }

    let mut y = y_lock.get();
    if y.is_some_and(|l| (l.source.y + d.y - l.value).abs() > release) { y = None; }
    if y.is_none() {
        y = sources.iter().flat_map(|source| {
            super::lines_near(&targets.ys, source.y + d.y, acquire).iter().filter_map(move |(value, from, kind)| {
                let distance = (value - (source.y + d.y)).abs();
                (distance <= acquire).then_some((distance, AxisLock { source: *source, value: *value, from: *from, kind: *kind }))
            })
        }).chain(sources.iter().flat_map(|source| {
            let current = *source + d;
            targets.rulers.iter().filter(|r| !r.vertical).filter_map(move |ruler| {
                ruler.line_at(current, acquire).map(|(value, from, kind)| ((value - current.y).abs(), AxisLock { source: *source, value, from, kind }))
            })
        })).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, l)| l);
    }
    if let Some(lock) = y {
        d.y = lock.value - lock.source.y;
        y_lock.set(Some(lock));
        if targets.style.lines {
            if let Some(r) = bounds.map(|b| b + d) {
                guides.push(Overlay::Line {
                    a: Point::new(r.x0.min(lock.from.x), lock.value),
                    b: Point::new(r.x1.max(lock.from.x), lock.value),
                    color: targets.style.color, dashed: false,
                });
            }
        }
        if targets.style.labels {
            guides.push(Overlay::Label { p: lock.from, text: if lock.kind == Kind::Guide { "guide" } else { "align" }.into(), color: targets.style.color });
        }
    } else {
        y_lock.set(None);
    }
    // Spacing Guides are a separate geometric relationship: apply them only on axes
    // where no point/edge alignment has already won.
    if let Some(base) = bounds {
        let aligned = [x.is_some(), y.is_some()];
        let mut spaced = false;
        for (is_x, lined_up) in [true, false].into_iter().zip(aligned) {
            if lined_up { continue; }
            if let Some((shift, _)) = targets.even_spacing(base + d, is_x, acquire) {
                if is_x { d.x += shift; } else { d.y += shift; }
                spaced = true;
            }
        }
        if spaced {
            let done = base + d;
            for (is_x, lined_up) in [true, false].into_iter().zip(aligned) {
                if lined_up { continue; }
                if let Some((_, gaps)) = targets.even_spacing(done, is_x, acquire) {
                    guides.extend(gaps.into_iter().flat_map(|gap| targets.gap_marks(gap, is_x, acquire)));
                }
            }
        }
    }
    (d, guides)
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
    fn leo_snapping_engine_artboard_edges_are_real_finite_geometry_targets() {
        use vectorcraft_doc::Document;
        let doc = Document::new(500.0, 500.0);
        let selection = Selection::default();
        let paint = crate::testutil::paint();
        let cx = crate::testutil::cx(&doc, &selection, &paint);
        let t = Targets::leo_for_anchor_drag(&cx);
        assert_eq!(snap(&t, Point::new(504.0, 235.0), 1.0).unwrap().0,
            Point::new(500.0, 235.0));
        t.clear_leo_snap();
        assert_eq!(snap(&t, Point::new(-4.0, 200.0), 1.0).unwrap().0,
            Point::new(0.0, 200.0));
        t.clear_leo_snap();
        assert!(snap(&t, Point::new(500.0, 515.0), 1.0).is_none(),
            "beyond the end of an artboard side, no infinite edge extension");
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
    fn move_snap(bounds: Rect, sources: Vec<Point>, targets: Targets) -> ObjectMoveSnap {
        ObjectMoveSnap {
            bounds: Some(bounds),
            sources,
            targets,
            copy_targets: Targets::default(),
            lock: Cell::new(None),
            x_lock: Cell::new(None),
            y_lock: Cell::new(None),
            last_copy: Cell::new(None),
        }
    }

    #[test]
    fn leo_object_move_snaps_source_anchor_to_target_anchor_exactly() {
        let mut targets = Targets::default();
        targets.points.push((Point::new(300.0, 100.0), Kind::Anchor));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 100.0)],
            targets,
        );
        let (d, guides) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(99.0, 0.0), false, 8.0, 12.8, None,
        );
        assert_eq!(d, Vec2::new(100.0, 0.0));
        assert!(guides.iter().any(|g| matches!(g, Overlay::Label { text, .. } if text == "anchor")));
    }

    #[test]
    fn leo_object_move_snaps_bounds_or_nodes_to_a_finite_path_segment() {
        let mut targets = Targets::default();
        targets.segments.push((
            Rect::new(300.0, 100.0, 300.0, 200.0),
            PathSeg::Line(Line::new((300.0, 100.0), (300.0, 200.0))),
        ));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 150.0)],
            targets,
        );
        let (d, guides) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(99.0, 0.0), false, 8.0, 12.8, None,
        );
        assert_eq!(d, Vec2::new(100.0, 0.0));
        assert!(guides.iter().any(|g| matches!(g, Overlay::Label { text, .. } if text == "path")));
    }

    #[test]
    fn leo_object_move_snaps_object_edge_to_artboard_edge() {
        let mut targets = Targets::default();
        targets.segments.push((
            Rect::new(500.0, 0.0, 500.0, 500.0),
            PathSeg::Line(Line::new((500.0, 0.0), (500.0, 500.0))),
        ));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 100.0), Point::new(200.0, 200.0)],
            targets,
        );
        let (d, _) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(299.0, 0.0), false, 8.0, 12.8, None,
        );
        assert_eq!(d, Vec2::new(300.0, 0.0));
    }

    #[test]
    fn leo_object_move_keeps_capture_until_twelve_point_eight_pixels_then_releases() {
        let mut targets = Targets::default();
        targets.points.push((Point::new(300.0, 100.0), Kind::Anchor));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 100.0)],
            targets,
        );
        assert_eq!(snapper.snap_with_tolerance(Point::new(150.0, 150.0), Vec2::new(99.0, 0.0), false, 8.0, 12.8, None).0.x, 100.0);
        assert_eq!(snapper.snap_with_tolerance(Point::new(150.0, 150.0), Vec2::new(108.0, 0.0), false, 8.0, 12.8, None).0.x, 100.0);
        assert_eq!(snapper.snap_with_tolerance(Point::new(150.0, 150.0), Vec2::new(113.0, 0.0), false, 8.0, 12.8, None).0.x, 113.0);
    }

    #[test]
    fn leo_object_move_uses_screen_pixel_capture_distance_at_multiple_zooms() {
        for zoom in [0.25, 1.0, 4.0] {
            let mut targets = Targets::default();
            targets.points.push((Point::new(300.0, 100.0), Kind::Anchor));
            let snapper = move_snap(
                Rect::new(100.0, 100.0, 200.0, 200.0),
                vec![Point::new(200.0, 100.0)],
                targets,
            );
            let near = 100.0 - 7.9 / zoom;
            let far = 100.0 - 8.1 / zoom;
            assert_eq!(
                snapper.snap_with_tolerance(Point::new(150.0, 150.0), Vec2::new(near, 0.0), false, 8.0 / zoom, 12.8 / zoom, None).0.x,
                100.0,
                "near capture at zoom {zoom}",
            );
            snapper.reset_locks();
            assert_eq!(
                snapper.snap_with_tolerance(Point::new(150.0, 150.0), Vec2::new(far, 0.0), false, 8.0 / zoom, 12.8 / zoom, None).0.x,
                far,
                "far release at zoom {zoom}",
            );
        }
    }

    #[test]
    fn leo_object_move_aligns_both_axes_when_no_geometry_target_is_near() {
        let mut targets = Targets::default();
        targets.xs.push((300.0, Point::new(300.0, 150.0), Kind::Edge));
        targets.ys.push((250.0, Point::new(150.0, 250.0), Kind::Edge));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 200.0)],
            targets,
        );
        let (d, guides) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(99.0, 49.0), false, 8.0, 12.8, None,
        );
        assert_eq!(d, Vec2::new(100.0, 50.0));
        assert_eq!(guides.iter().filter(|g| matches!(g, Overlay::Line { .. })).count(), 2);
    }

    #[test]
    fn leo_object_move_shift_keeps_translation_on_its_drag_line() {
        let mut targets = Targets::default();
        targets.points.push((Point::new(300.0, 100.0), Kind::Anchor));
        let snapper = move_snap(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            vec![Point::new(200.0, 100.0)],
            targets,
        );
        let (d, _) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(99.0, 0.0), false, 8.0, 12.8, Some(Vec2::new(99.0, 0.0)),
        );
        assert_eq!(d, Vec2::new(100.0, 0.0));
    }

    #[test]
    fn leo_object_move_alt_copy_uses_original_art_as_a_snap_target() {
        let mut originals = Targets::default();
        originals.points.push((Point::new(200.0, 100.0), Kind::Anchor));
        let snapper = ObjectMoveSnap {
            bounds: Some(Rect::new(100.0, 100.0, 200.0, 200.0)),
            sources: vec![Point::new(200.0, 100.0)],
            targets: Targets::default(),
            copy_targets: originals,
            lock: Cell::new(None),
            x_lock: Cell::new(None),
            y_lock: Cell::new(None),
            last_copy: Cell::new(None),
        };
        let (d, _) = snapper.snap_with_tolerance(
            Point::new(150.0, 150.0), Vec2::new(5.0, 0.0), true, 8.0, 12.8, None,
        );
        assert_eq!(d, Vec2::ZERO);
    }

}

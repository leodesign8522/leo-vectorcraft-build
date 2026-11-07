//! Leo Design Snapping Engine v1.6: finite geometry and indexed straight-edge extensions.
//! Node drags use exact anchors and finite path geometry; object moves add selected bounds,
//! multi-object/group source anchors, alignment axes and artboard edges. Capture is in SCREEN px.
use super::*;
use std::cell::Cell;

pub(super) const ACQUIRE_PX: f64 = 8.0;
pub(super) const RELEASE_MULTIPLIER: f64 = 1.6;
const ANCHOR_HIT: [u8; 3] = [0x35, 0xc2, 0xfa];
const PATH_HIT: [u8; 3] = [0xff, 0xb3, 0x3d];
const EXTENSION_HIT: [u8; 3] = [0xff, 0xd6, 0x00];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::guides) struct LeoExtension {
    at: Point,
    end: Point,
    dir: Vec2,
}

impl LeoExtension {
    fn new(at: Point, end: Point) -> Option<Self> {
        let d=end-at;
        let length=d.hypot();
        if !at.is_finite() || !end.is_finite() || !length.is_finite() || length<=1e-9 { return None; }
        let mut normal=Vec2::new(-d.y/length,d.x/length);
        if normal.y<0.0 || (normal.y==0.0 && normal.x<0.0) {normal=-normal;}
        let dir=Vec2::new(normal.y,-normal.x);
        let (at,end)=if d.dot(dir)<0.0 {(end,at)} else {(at,end)};
        Some(Self {at,end,dir})
    }

    fn project(self, p: Point, reach: f64) -> Option<Point> {
        let q=self.at+self.dir*(p-self.at).dot(self.dir);
        (q.is_finite() && q.distance(p)<=reach).then_some(q)
    }

    fn reference_in_view(self, view: Option<&crate::ScreenFrame>) -> bool {
        let Some(view)=view else {return true};
        if !view.size.0.is_finite() || !view.size.1.is_finite() || view.size.0<=0.0 || view.size.1<=0.0 {return true;}
        let (Some((ax,ay)),Some((bx,by)))=(view.to_screen(self.at),view.to_screen(self.end)) else {return false};
        // Clip the actual segment, not its bounding box: diagonal off-screen
        // edges can have a box spanning the whole view without being visible.
        let mut lo: f64=0.0;
        let mut hi: f64=1.0;
        for (start,delta,min,max) in [(ax,bx-ax,-8.0,view.size.0+8.0),(ay,by-ay,-8.0,view.size.1+8.0)] {
            if !start.is_finite() || !delta.is_finite() {return false;}
            if delta.abs()<1e-12 {if start<min || start>max {return false;}continue;}
            let a=(min-start)/delta;let b=(max-start)/delta;
            lo=lo.max(a.min(b));hi=hi.min(a.max(b));
            if lo>hi {return false;}
        }
        true
    }
}

struct ExtensionBucket {
    normal: Vec2,
    entries: Vec<(f64,LeoExtension)>,
}

/// Sort straight lines by normal direction and signed offset. Queries use a
/// conservative angular band, followed by exact projection: distant segment
/// bounds must not hide a valid infinite extension, and no 20k cutoff is used.
pub(in crate::guides) struct LeoExtensionIndex {
    origin: Point,
    buckets: Vec<ExtensionBucket>,
    angular_error: f64,
}

impl LeoExtensionIndex {
    const COUNT: usize=64;

    fn new(scene: &LeoScene, origin: Point) -> Self {
        let width=std::f64::consts::PI/Self::COUNT as f64;
        let mut out=Self {origin,angular_error:2.0*(width/4.0).sin(),
            buckets:(0..Self::COUNT).map(|i| {
                let angle=(i as f64+0.5)*width;
                ExtensionBucket {normal:Vec2::new(angle.cos(),angle.sin()),entries:vec![]}
            }).collect()};
        for entry in &scene.paths {
            let Some(path)=entry.node.path_data() else {continue};
            for (si,sp) in path.subpaths.iter().enumerate() {
                let n=sp.anchors.len();
                if n<2 {continue;}
                for seg in 0..sp.segment_count() {
                    if entry.moving.contains(&(si,seg)) || entry.moving.contains(&(si,(seg+1)%n)) || !sp.segment_is_line(seg) {continue;}
                    let c=sp.segment(seg);
                    if let Some(line)=LeoExtension::new(c.p0,c.p3) {out.insert(line);}
                }
            }
        }
        for bucket in &mut out.buckets {
            bucket.entries.sort_by(|a,b| a.0.total_cmp(&b.0)
                .then(a.1.dir.x.total_cmp(&b.1.dir.x)).then(a.1.dir.y.total_cmp(&b.1.dir.y))
                .then(a.1.at.x.total_cmp(&b.1.at.x)).then(a.1.at.y.total_cmp(&b.1.at.y))
                .then(a.1.end.x.total_cmp(&b.1.end.x)).then(a.1.end.y.total_cmp(&b.1.end.y)));
            // Keep distinct collinear segments: one can be on screen while
            // another supporting the same infinite line is outside the view.
            bucket.entries.dedup_by(|a,b| a.0==b.0 && a.1==b.1);
        }
        out
    }

    fn insert(&mut self, line: LeoExtension) {
        let normal=Vec2::new(-line.dir.y,line.dir.x);
        let offset=(line.at-self.origin).dot(normal);
        let angle=normal.y.atan2(normal.x);
        if !offset.is_finite() || !angle.is_finite() || angle<0.0 {return;}
        let i=((angle/std::f64::consts::PI*Self::COUNT as f64) as usize).min(Self::COUNT-1);
        if let Some(bucket)=self.buckets.get_mut(i) {bucket.entries.push((offset,line));}
    }

    #[cfg(test)]
    fn nearest(&self, p: Point, reach: f64) -> Option<(LeoExtension,Point)> {
        self.nearest_in_view(p,reach,None)
    }

    fn nearest_in_view(&self, p: Point, reach: f64, view: Option<&crate::ScreenFrame>) -> Option<(LeoExtension,Point)> {
        let d=p-self.origin;
        let radius=d.hypot();
        if !p.is_finite() || !radius.is_finite() || !reach.is_finite() || reach<=0.0 {return None;}
        let margin=reach+radius*self.angular_error+1e-9*(1.0+radius);
        let mut best: Option<(LeoExtension,Point)>=None;
        for bucket in &self.buckets {
            let center=d.dot(bucket.normal);
            let lo=bucket.entries.partition_point(|(offset,_)| *offset<center-margin);
            let hi=bucket.entries.partition_point(|(offset,_)| *offset<=center+margin);
            for (_,line) in bucket.entries.get(lo..hi).unwrap_or_default() {
                if !line.reference_in_view(view) {continue;}
                if let Some(q)=line.project(p,reach) {
                    if best.is_none_or(|(_,old)| q.distance(p)<old.distance(p)) {best=Some((*line,q));}
                }
            }
        }
        best
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::guides) enum SnapLock {
    Anchor(Point),
    Segment(PathSeg),
}

/// Immutable path references from the drag-start scene. Query all intersecting
/// geometry locally instead of silently truncating the document's first 20k anchors.
pub(in crate::guides) struct LeoScene {
    paths: Vec<LeoPath>,
    rectangles: Vec<(Rect, Kind)>,
    centers: bool,
}

struct LeoPath {
    node: Arc<Node>,
    bounds: Rect,
    moving: HashSet<(usize, usize)>,
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && a.x1 >= b.x0 && a.y0 <= b.y1 && a.y1 >= b.y0
}

impl LeoScene {
    fn new(cx: &ToolContext, selected: &Selection, object_bounds: bool) -> Self {
        let mut scene = Self { paths: vec![], rectangles: cx.doc.artboards.iter().map(|a| (a.rect, Kind::Artboard)).collect(), centers: object_bounds };
        let skip: HashSet<NodeId> = selected.objects.iter().copied().collect();
        let mut pending: Vec<&Arc<Node>> = cx.doc.layers.iter().rev().collect();
        while let Some(node) = pending.pop() {
            if !node.visible || skip.contains(&node.id) { continue; }
            if node.path_data().is_some() {
                if let Some(bounds) = node.geometric_bounds().filter(|b| b.is_finite()) {
                    scene.paths.push(LeoPath { node: Arc::clone(node), bounds, moving: HashSet::new() });
                }
            } else if object_bounds && !node.is_container() {
                if let Some(bounds) = node.geometric_bounds().filter(|b| b.is_finite()) {
                    scene.rectangles.push((bounds, Kind::Edge));
                }
            }
            if let Some(children) = node.children() {
                let shown = children.get(usize::from(node.extras().shaper.is_some())..).unwrap_or_default();
                pending.extend(shown.iter().rev());
            }
        }
        // Stationary anchors/segments of a partly edited path remain valid targets.
        // Descendants of a selected whole group move with it and must be excluded.
        for (id, moving) in &selected.anchors {
            if cx.doc.ancestry(*id).is_some_and(|a| a.iter().rev().skip(1).any(|p| skip.contains(p))) { continue; }
            let Some(node) = cx.doc.node(*id).filter(|n| n.visible && n.path_data().is_some()) else { continue };
            if let Some(bounds) = node.geometric_bounds().filter(|b| b.is_finite()) {
                scene.paths.push(LeoPath { node: Arc::new(node.clone()), bounds, moving: moving.iter().copied().collect() });
            }
        }
        scene
    }

    fn alignment_targets(&self) -> Targets {
        let mut out = Targets::default();
        for (r, kind) in self.paths.iter().filter(|e| e.moving.is_empty()).map(|e| (e.bounds, Kind::Edge))
            .chain(self.rectangles.iter().copied()) {
            for p in [Point::new(r.x0,r.y0), Point::new(r.x1,r.y0), Point::new(r.x1,r.y1), Point::new(r.x0,r.y1)] {
                out.xs.push((p.x,p,kind));
                out.ys.push((p.y,p,kind));
            }
            let p = r.center();
            out.xs.push((p.x,p,Kind::Center));
            out.ys.push((p.y,p,Kind::Center));
        }
        out.index()
    }

    fn near(&self, query: Rect) -> Targets {
        let mut out = Targets::default();
        if !query.is_finite() { return out; }
        for entry in self.paths.iter().filter(|e| overlaps(e.bounds, query)) {
            let Some(path) = entry.node.path_data() else { continue };
            if self.centers && entry.moving.is_empty() && query.contains(entry.bounds.center()) {
                out.points.push((entry.bounds.center(), Kind::Center));
            }
            for (si, sp) in path.subpaths.iter().enumerate() {
                for (ai, anchor) in sp.anchors.iter().enumerate() {
                    if !entry.moving.contains(&(si, ai)) && query.contains(anchor.p) {
                        out.points.push((anchor.p, Kind::Anchor));
                    }
                }
                let n = sp.anchors.len();
                for i in 0..sp.segment_count() {
                    if n == 0 || entry.moving.contains(&(si, i)) || entry.moving.contains(&(si, (i + 1) % n)) { continue; }
                    let c = sp.segment(i);
                    let bounds = Rect::from_points(c.p0, c.p1).union_pt(c.p2).union_pt(c.p3);
                    if overlaps(bounds, query) {
                        let segment = if sp.segment_is_line(i) { PathSeg::Line(Line::new(c.p0, c.p3)) } else { PathSeg::Cubic(c) };
                        out.segments.push((bounds, segment));
                    }
                }
            }
        }
        for (r, _) in self.rectangles.iter().filter(|(r, _)| overlaps(*r, query)) {
            if self.centers && query.contains(r.center()) { out.points.push((r.center(),Kind::Center)); }
            let corners = [Point::new(r.x0,r.y0),Point::new(r.x1,r.y0),Point::new(r.x1,r.y1),Point::new(r.x0,r.y1)];
            for i in 0..4 {
                let line = Line::new(corners[i], corners[(i+1)%4]);
                let bounds = Rect::from_points(line.p0,line.p1);
                if overlaps(bounds,query) { out.segments.push((bounds,PathSeg::Line(line))); }
            }
        }
        out
    }
}

impl Targets {
    pub(in crate::guides) fn leo_scale_targets(cx: &ToolContext) -> Self {
        Self::leo_for_object_move(cx,&cx.selection.objects)
    }

    /// Resize has its own legal degrees of freedom. Projecting the pointer like
    /// a node would move the opposite side or break proportional/centred scaling.
    pub(in crate::guides) fn leo_resize(
        &self,cx: &ToolContext,bx: &OrientedBox,handle: Handle,a: Affine,proportional: bool,from_center: bool,
    ) -> Option<(Affine,Vec<Overlay>)> {
        self.leo_scene.as_ref()?;
        let acquire=cx.tol(ACQUIRE_PX);let release=acquire*RELEASE_MULTIPLIER;
        let to_doc=bx.to_doc();let raw=to_doc*(a*handle.pos(bx.rect));
        let restricted=!handle.is_corner() || proportional;
        let held=self.leo_scale.try_borrow().ok().and_then(|v| v.clone());
        if let Some(held)=held.filter(|l| restricted && l.handle==handle && l.proportional==proportional && l.from_center==from_center) {
            if raw.distance(to_doc*(held.matrix*handle.pos(bx.rect)))<=release {
                return Some((held.matrix,held.guides));
            }
        }
        if let Ok(mut held)=self.leo_scale.try_borrow_mut() {*held=None;}
        let bounds=to_doc.transform_rect_bbox(a.transform_rect_bbox(bx.rect)).inflate(release,release);
        let nearby=self.leo_near(bounds)?;
        let origin=if from_center {bx.rect.center()} else {handle.opposite().pos(bx.rect)};
        let [sx,_,_,sy,_,_]=a.as_coeffs();
        let valid=|nx: f64,ny: f64| nx.is_finite() && ny.is_finite() && nx*sx>0.0 && ny*sy>0.0 && nx.abs()>=1e-4 && ny.abs()>=1e-4;
        let matrix=|nx,ny| Affine::translate(origin.to_vec2())*Affine::scale_non_uniform(nx,ny)*Affine::translate(-origin.to_vec2());
        let mut best: Option<(u8,f64,Affine,Point,&'static str,[u8;3])>=None;
        let mut consider=|candidate: Affine,q: Point,kind: Kind| {
            let travel=raw.distance(to_doc*(candidate*handle.pos(bx.rect)));
            if !travel.is_finite() || travel>acquire {return;}
            let (priority,label,color)=match kind {Kind::Anchor=>(0,"anchor",ANCHOR_HIT),Kind::Center=>(2,"center",PATH_HIT),_=>(1,"path",PATH_HIT)};
            if best.as_ref().is_none_or(|b| (priority,travel)<(b.0,b.1)) {best=Some((priority,travel,candidate,q,label,color));}
        };
        if restricted {
            let (ax,_)=handle.axes();
            let (e,c,s0)=match (handle.is_corner(),ax,proportional) {
                (true,_,_)=>((sx.signum(),sy.signum()),(0.0,0.0),sx.abs()),
                (false,true,true)=>((1.0,sx.signum()),(0.0,0.0),sx),
                (false,true,false)=>((1.0,0.0),(0.0,1.0),sx),
                (false,false,true)=>((sy.signum(),1.0),(0.0,0.0),sy),
                (false,false,false)=>((0.0,1.0),(1.0,0.0),sy),
            };
            let ends=if handle.is_corner() {vec![handle]} else {vec![handle,Handle::ALL[(handle as usize+1)%8],Handle::ALL[(handle as usize+7)%8]]};
            for h in ends {
                let v=h.pos(bx.rect)-origin;
                let base=to_doc*(origin+Vec2::new(c.0*v.x,c.1*v.y));
                let dir=to_doc*Point::new(e.0*v.x,e.1*v.y)-to_doc*Point::ORIGIN;
                let len2=dir.hypot2();if !len2.is_finite() || len2<=1e-18 {continue;}
                let p=base+dir*s0;
                let mut accept=|q: Point,kind| {
                    let s=(q-base).dot(dir)/len2;
                    let (nx,ny)=(c.0+e.0*s,c.1+e.1*s);
                    if valid(nx,ny) && (base+dir*s).distance(q)<=1e-7 {consider(matrix(nx,ny),q,kind);}
                };
                for &(q,kind) in &nearby.points {
                    if p.distance(q)<=acquire && matches!(kind,Kind::Anchor|Kind::Center) {accept(q,kind);}
                }
                for (_,seg) in &nearby.segments {
                    if let Some(q)=along_segment(*seg,p,dir,acquire) {accept(q,Kind::Edge);}
                }
            }
        } else {
            let d0=handle.pos(bx.rect)-origin;
            if d0.x.abs()>1e-9 && d0.y.abs()>1e-9 {
                let mut accept=|q: Point,kind| {
                    let local=to_doc.inverse()*q;
                    let (nx,ny)=((local.x-origin.x)/d0.x,(local.y-origin.y)/d0.y);
                    if valid(nx,ny) {consider(matrix(nx,ny),q,kind);}
                };
                for &(q,kind) in &nearby.points {
                    if raw.distance(q)<=acquire && matches!(kind,Kind::Anchor|Kind::Center) {accept(q,kind);}
                }
                for (_,seg) in &nearby.segments {
                    let nearest=seg.nearest(raw,1e-8);
                    if nearest.distance_sq<=acquire*acquire {accept(seg.eval(nearest.t),Kind::Edge);}
                }
            }
        }
        let result=if let Some((_,_,a,q,label,color))=best {
            Some((a,move_hit_guides(self,q,label,color)))
        } else {
            // Bounds and ruler alignment retain the official oriented-box math.
            let rulers=Targets::rulers(cx);
            let alignment=if cx.smart_guides {self} else {&rulers};
            let (next,guides)=alignment.snap_scale(bx,handle,a,proportional,from_center,acquire);
            let travel=raw.distance(to_doc*(next*handle.pos(bx.rect)));
            ((travel>1e-9 || !guides.is_empty()) && travel<=acquire).then_some((next,guides))
        };
        if let Some((matrix,guides))=result {
            if restricted && let Ok(mut held)=self.leo_scale.try_borrow_mut() {
                *held=Some(LeoScaleLock {handle,proportional,from_center,matrix,guides:guides.clone()});
            }
            return Some((matrix,guides));
        }
        // Geometry wins over grid/pixels. With no capture their native fallback
        // remains available; otherwise do not re-enter the legacy resize query.
        (!(cx.snap_to_grid || cx.snap_to_pixel)).then_some((a,vec![]))
    }

    fn leo_near(&self, query: Rect) -> Option<Self> {
        let mut targets = self.leo_scene.as_ref()?.near(query);
        targets.style = self.style;
        targets.tangent = self.tangent;
        targets.perpendicular = self.perpendicular;
        Some(targets)
    }
    /// Shift captures only geometry that the locked drag line really reaches.
    pub(in crate::guides) fn leo_snap_anchor_along(&self, p: Point, at: Point, acquire: f64, release: f64) -> Option<(Point, Vec<Overlay>)> {
        let dir = p - at;
        if !p.is_finite() || dir.hypot() <= 1e-9 || !acquire.is_finite() || acquire <= 0.0 || release < acquire {
            self.clear_leo_snap();
            return None;
        }
        let nearby = self.leo_near(Rect::from_points(p,p).inflate(release,release));
        let geometry = nearby.as_ref().unwrap_or(self);
        let anchor = geometry.points.iter().filter(|(_, k)| *k == Kind::Anchor)
            .filter(|(q, _)| q.distance(p) <= acquire && on_direction(*q - at, dir))
            .min_by(|a, b| a.0.distance(p).total_cmp(&b.0.distance(p)))
            .map(|(q, _)| (SnapLock::Anchor(*q), *q));
        let held = self.leo_snap.get().and_then(|lock| {
            let q = match lock {
                SnapLock::Anchor(q) => (q.distance(p) <= release && on_direction(q - at, dir)).then_some(q),
                SnapLock::Segment(segment) => along_segment(segment, p, dir, release),
            }?;
            Some((lock, q))
        });
        let choice = held.filter(|(l, _)| matches!(l, SnapLock::Anchor(_))).or(anchor).or(held)
            .or_else(|| geometry.segments.iter().filter_map(|(_, s)| {
                along_segment(*s, p, dir, acquire).map(|q| (SnapLock::Segment(*s), q))
            }).min_by(|a, b| a.1.distance(p).total_cmp(&b.1.distance(p))));
        self.leo_snap.set(choice.map(|(l, _)| l));
        let (lock, q) = choice?;
        let (label, color) = if matches!(lock, SnapLock::Anchor(_)) { ("anchor", ANCHOR_HIT) } else { ("path", PATH_HIT) };
        Some((q, move_hit_guides(self, q, label, color)))
    }

    /// Gather real path geometry plus the four finite edges of each artboard.
    /// Artboards are painted as white rectangles but are NOT Path nodes: without
    /// these edges a dragged node beside the page had no geometric snap target.
    pub(in crate::guides) fn leo_for_anchor_drag(cx: &ToolContext) -> Self {
        let mut targets = Self::default().styled(cx);
        targets.leo_scene = Some(LeoScene::new(cx, cx.selection, false));
        if cx.smart_guides { targets.leo_collect_extensions(cx); }
        targets.tangent = cx.smart_guides && cx.snap_tangent;
        targets.perpendicular = cx.smart_guides && cx.snap_perpendicular;
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
        let scene = LeoScene::new(cx, &selection, true);
        let mut targets = scene.alignment_targets().styled(cx);
        targets.leo_scene = Some(scene);
        if cx.snap_to_point { targets = targets.with_rulers(cx); }
        if cx.spacing_guides { targets.boxes = super::object_boxes(cx.doc,exclude); }
        push_artboard_edges(cx.doc, &mut targets);
        targets
    }

    pub(in crate::guides) fn clear_leo_snap(&self) {
        self.leo_snap.set(None);
        self.leo_extension.set(None);
    }

    /// Freeze the edited anchor's straight neighbours and an indexed snapshot
    /// of other stationary straight edges at drag start, not at each preview.
    fn leo_collect_extensions(&mut self, cx: &ToolContext) {
        let selection = cx.selection;
        if !selection.segments.is_empty() || selection.anchors.len() != 1 { return; }
        let Some((&id, moving)) = selection.anchors.iter().next() else { return };
        if moving.len() != 1 || selection.objects.iter().any(|other| *other != id) { return; }
        let Some(&(si, ai)) = moving.iter().next() else { return };
        let Some(sp) = cx.doc.node(id).and_then(|n| n.path_data()).and_then(|p| p.subpaths.get(si)) else { return };
        let n = sp.anchors.len();
        if n < 2 || ai >= n { return; }
        if let Some(origin)=sp.anchors.get(ai).map(|a| a.p).filter(|p| p.is_finite()) {
            self.leo_extension_index=self.leo_scene.as_ref().map(|scene| LeoExtensionIndex::new(scene,origin));
        }
        let previous = if ai > 0 { Some(ai - 1) } else if sp.closed { Some(n - 1) } else { None };
        for seg in [previous, (ai < sp.segment_count()).then_some(ai)].into_iter().flatten() {
            if seg >= sp.segment_count() || !sp.segment_is_line(seg) { continue; }
            let c = sp.segment(seg);
            let (fixed, dragged) = if seg == ai { (c.p3,c.p0) } else { (c.p0,c.p3) };
            let d = dragged - fixed;
            let len = d.hypot();
            if fixed.is_finite() && dragged.is_finite() && len.is_finite() && len > 1e-9 {
                self.extensions.push((fixed,d / len));
            }
        }
    }

    /// Bidirectional collinearity is a distinct constraint, not a finite path hit.
    /// Shift keeps its existing constraint without a second extension capture.
    pub(in crate::guides) fn leo_snap_extension(&self, p: Point, acquire: f64, release: f64, direction: Option<Vec2>) -> Option<(Point,Vec<Overlay>)> {
        self.leo_snap_extension_in_view(p,acquire,release,direction,None)
    }

    pub(in crate::guides) fn leo_snap_extension_in_view(&self, p: Point, acquire: f64, release: f64, direction: Option<Vec2>, view: Option<&crate::ScreenFrame>) -> Option<(Point,Vec<Overlay>)> {
        if direction.is_some() || !p.is_finite() || !acquire.is_finite() || !release.is_finite() || acquire <= 0.0 || release < acquire {
            self.leo_extension.set(None);
            return None;
        }
        let hit = self.leo_extension.get().and_then(|line| line.project(p,release).map(|q| (line,q)))
            .or_else(|| {
                let local=self.extensions.iter().filter_map(|&(at,dir)| {
                    let line=LeoExtension {at,end:at,dir};
                    line.project(p,acquire).map(|q| (line,q))
                }).min_by(|a,b| a.1.distance(p).total_cmp(&b.1.distance(p)));
                let external=self.leo_extension_index.as_ref().and_then(|index| index.nearest_in_view(p,acquire,view));
                match (local,external) {
                    (Some(a),Some(b))=>Some(if b.1.distance(p)<a.1.distance(p) {b} else {a}),
                    (a,b)=>a.or(b),
                }
            });
        self.leo_extension.set(hit.map(|(line,_)| line));
        let (line,q) = hit?;
        // Infinite supporting lines can start far outside the viewport. Bound
        // painted dash count, without bounding the mathematical snap target.
        let span=(acquire*64.0).min(1e9);
        let at=if line.at.distance(q)>span {
            let sign=if (q-line.at).dot(line.dir)<0.0 {-1.0} else {1.0};
            q-line.dir*(sign*span)
        } else {line.at};
        let mut guides = vec![];
        if self.style.lines {
            let mut outline=BezPath::new(); outline.move_to(at); outline.line_to(q);
            guides.push(Overlay::Path {path:outline,color:[0x20,0x24,0x2b],width:3.5,dashed:true});
            guides.push(Overlay::Line { a:at,b:q,color:EXTENSION_HIT,dashed:true });
            guides.push(Overlay::CurvePoint { p:q,color:[0x20,0x24,0x2b],filled:true,size:12.0 });
            guides.push(Overlay::CurvePoint { p:q,color:EXTENSION_HIT,filled:false,size:10.0 });
        }
        if self.style.labels { guides.push(Overlay::Label { p:q,text:"延长线吸附".into(),color:EXTENSION_HIT }); }
        Some((q,guides))
    }

    /// Explicit native geometric-guide options still land on real finite geometry.
    pub(in crate::guides) fn leo_snap_contact(&self, p: Point, at: Point, acquire: f64) -> Option<(Point, Vec<Overlay>)> {
        let nearby = self.leo_near(Rect::from_points(p,p).inflate(acquire,acquire));
        let geometry = nearby.as_ref().unwrap_or(self);
        if geometry.leo_nearest_anchor(p, acquire).is_some() { return None; }
        if self.leo_snap.get().is_some_and(|lock| matches!(lock, SnapLock::Anchor(q) if q.distance(p) <= acquire * RELEASE_MULTIPLIER)) { return None; }
        let (q, perpendicular, dir) = geometry.touching(at, p, acquire)?;
        let (_, segment) = geometry.segments.iter().find(|(_, s)| s.nearest(q, 1e-8).distance_sq < 1e-12)?;
        self.leo_snap.set(Some(SnapLock::Segment(*segment)));
        Some((q, self.touch_guides(at, q, perpendicular, dir, acquire)))
    }

    fn leo_nearest_anchor(&self, p: Point, reach: f64) -> Option<Point> {
        self.points
            .iter()
            .filter(|(_, kind)| *kind == Kind::Anchor)
            .filter(|(q, _)| q.distance(p) <= reach)
            .min_by(|(a, _), (b, _)| a.distance(p).total_cmp(&b.distance(p)))
            .map(|(q, _)| *q)
    }

    fn leo_nearest_segment(&self, p: Point, reach: f64) -> Option<(PathSeg, Point)> {
        self.segments
            .iter()
            .filter(|(bounds, _)| bounds.inflate(reach, reach).contains(p))
            .filter_map(|(_, segment)| {
                // Kurbo's nearest-point solver projects onto the actual finite
                // line or Bezier, never its bounding rectangle or guide line.
                let nearest = segment.nearest(p, 1e-8);
                (nearest.distance_sq.is_finite() && nearest.distance_sq <= reach * reach)
                    .then_some((*segment, segment.eval(nearest.t), nearest.distance_sq))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(index, q, _)| (index, q))
    }

    fn leo_locked_point(&self, lock: SnapLock, p: Point, release: f64) -> Option<Point> {
        match lock {
            SnapLock::Anchor(q) => (q.distance(p) <= release).then_some(q),
            SnapLock::Segment(segment) => {
                let nearest = segment.nearest(p, 1e-8);
                (nearest.distance_sq.is_finite() && nearest.distance_sq <= release * release)
                    .then(|| segment.eval(nearest.t))
            },
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

        let nearby = self.leo_near(Rect::from_points(p,p).inflate(release,release));
        let geometry = nearby.as_ref().unwrap_or(self);
        let anchor = geometry.leo_nearest_anchor(p, acquire);
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
        .or_else(|| geometry.leo_nearest_segment(p, acquire).map(|(segment, q)| (SnapLock::Segment(segment), q)));

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

#[derive(Clone)]
pub(in crate::guides) struct LeoScaleLock {
    handle: Handle,
    proportional: bool,
    from_center: bool,
    matrix: Affine,
    guides: Vec<Overlay>,
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

fn on_direction(v: Vec2, dir: Vec2) -> bool {
    (v.x * dir.y - v.y * dir.x).abs() <= 1e-7 * dir.hypot()
}

fn along_segment(segment: PathSeg, p: Point, dir: Vec2, reach: f64) -> Option<Point> {
    if dir.hypot() <= 1e-9 { return None; }
    let u = dir / dir.hypot();
    let line = Line::new(p - u * reach, p + u * reach);
    // Collinear finite lines need nearest(), since intersect_line omits overlaps.
    let nearest = segment.nearest(p, 1e-8);
    let q = segment.eval(nearest.t);
    if q.distance(p) <= reach && on_direction(q - p, dir) { return Some(q); }
    segment.intersect_line(line).iter().map(|i| line.eval(i.line_t))
        .filter(|q| q.distance(p) <= reach && on_direction(*q - p, dir))
        .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
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

fn selected_edges(cx: &ToolContext) -> Vec<PathSeg> {
    fn visit(nodes: &[Arc<Node>], selected: &[NodeId], inherited: bool, out: &mut Vec<PathSeg>, depth: usize) {
        if depth > 128 || out.len() >= 2048 { return; }
        for node in nodes.iter().filter(|n| n.visible) {
            let include = inherited || selected.contains(&node.id);
            if include && let Some(path) = node.path_data() {
                for sp in &path.subpaths {
                    for i in 0..sp.segment_count().min(2048usize.saturating_sub(out.len())) {
                        let c = sp.segment(i);
                        out.push(if sp.segment_is_line(i) { PathSeg::Line(Line::new(c.p0, c.p3)) } else { PathSeg::Cubic(c) });
                    }
                }
            }
            if let Some(children) = node.children() {
                visit(children.get(usize::from(node.extras().shaper.is_some())..).unwrap_or_default(), selected, include, out, depth + 1);
            }
        }
    }
    let mut out = Vec::new();
    visit(&cx.doc.layers, &cx.selection.objects, false, &mut out, 0);
    out
}

/// Project targets onto the continuous moving contour, rather than only its nine box features.
fn edge_sources(edges: &[PathSeg], targets: &Targets, d: Vec2, reach: f64, out: &mut Vec<Point>) {
    let mut pairs = 0usize;
    for edge in edges {
        let search = (vectorcraft_geom::kurbo::ParamCurveExtrema::bounding_box(edge) + d).inflate(reach, reach);
        for (q, kind) in &targets.points {
            if *kind != Kind::Anchor || !search.contains(*q) { continue; }
            let nearest = edge.nearest(*q - d, 1e-8);
            if nearest.distance_sq <= reach * reach { push_unique(out, edge.eval(nearest.t)); }
        }
        for (bounds, target) in &targets.segments {
            if search.intersect(bounds.inflate(reach, reach)).area() <= 0.0 { continue; }
            // Bound work in dense artwork; anchor and box snapping remain available.
            if pairs >= 2048 { return; }
            pairs += 1;
            for seed in 0..=8 {
                let mut p = edge.eval(seed as f64 / 8.0);
                for _ in 0..8 {
                    let t = target.nearest(p + d, 1e-8);
                    let q = target.eval(t.t);
                    let s = edge.nearest(q - d, 1e-8);
                    p = edge.eval(s.t);
                }
                let t = target.nearest(p + d, 1e-8);
                if t.distance_sq <= reach * reach { push_unique(out, p); }
            }
        }
    }
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
    Segment(PathSeg),
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
    edges: Vec<PathSeg>,
    targets: Targets,
    copy_targets: Targets,
    lock: Cell<Option<MoveLock>>,
    x_lock: Cell<Option<AxisLock>>,
    y_lock: Cell<Option<AxisLock>>,
    last_copy: Cell<Option<bool>>,
}

impl ObjectMoveSnap {
    pub(crate) fn captured(&self) -> bool {
        self.lock.get().is_some() || self.x_lock.get().is_some() || self.y_lock.get().is_some()
    }
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
            edges: selected_edges(cx),
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
        if d.hypot() <= 1e-9 { self.reset_locks(); return (d, vec![]); }
        self.snap_with_tolerance(start, d, copy, cx.tol(ACQUIRE_PX), cx.tol(ACQUIRE_PX * RELEASE_MULTIPLIER), Some(d))
    }

    fn snap_with_tolerance(
        &self, start: Point, d: Vec2, copy: bool, acquire: f64, release: f64, direction: Option<Vec2>,
    ) -> (Vec2, Vec<Overlay>) {
        if !start.is_finite() || !d.x.is_finite() || !d.y.is_finite() || !acquire.is_finite() || !release.is_finite() || acquire <= 0.0 || release < acquire {
            self.reset_locks();
            return (d, vec![]);
        }
        if self.last_copy.replace(Some(copy)).is_some_and(|previous| previous != copy) {
            self.reset_locks();
        }
        let targets = self.targets(copy);
        let moved = self.bounds.map(|r| r + d).unwrap_or_else(|| Rect::from_points(start+d,start+d)).union_pt(start+d);
        let nearby = targets.leo_near(moved.inflate(release,release));
        let geometry = nearby.as_ref().unwrap_or(targets);
        if direction.is_some() {
            if self.lock.get().is_some_and(|lock| lock.source != start) {
                self.lock.set(None);
            }
        }
        let mut sources = self.sources.clone();
        push_unique(&mut sources, start);
        if direction.is_none() { edge_sources(&self.edges, geometry, d, acquire, &mut sources); }
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
                if let Some(anchor) = find_move_geometry(geometry, &geometry_sources, self.bounds, d, acquire, direction)
                    .filter(|candidate| matches!(candidate.target, MoveTarget::Point { kind: Kind::Anchor, .. }))
                {
                    lock = anchor;
                    self.lock.set(Some(lock));
                }
            }
            if let Some((next, q, label, color)) = resolve_move_lock(geometry, lock, d, release, direction) {
                self.x_lock.set(None);
                self.y_lock.set(None);
                return (next, move_hit_guides(geometry, q, label, color));
            }
            self.lock.set(None);
        }

        // True anchors first, then finite paths/Bezier curves. Alignment never steals a geometry hit.
        if let Some(lock) = find_move_geometry(geometry, &geometry_sources, self.bounds, d, acquire, direction) {
            if let Some((next, q, label, color)) = resolve_move_lock(geometry, lock, d, release, direction) {
                self.lock.set(Some(lock));
                self.x_lock.set(None);
                self.y_lock.set(None);
                return (next, move_hit_guides(geometry, q, label, color));
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
    _targets: &Targets, lock: MoveLock, d: Vec2, release: f64, direction: Option<Vec2>,
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
        MoveTarget::Segment(segment) => {
            if let Some(dir) = direction {
                let q = along_segment(segment, current, dir, release)?;
                return Some((d + (q - current), q, "path", PATH_HIT));
            }
            let nearest = segment.nearest(current, 1e-8);
            if !nearest.distance_sq.is_finite() || nearest.distance_sq > release * release { return None; }
            (segment.eval(nearest.t), "path", PATH_HIT)
        }
    };
    let next = if let Some(dir) = direction.filter(|u| u.hypot() > 1e-9) {
        let u = dir / dir.hypot();
        let shift = (q - current).dot(u);
        let moved = current + u * shift;
        if moved.distance(q) > 1e-7 { return None; }
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
            if direction.is_some_and(|dir| !on_direction(*point - current, dir)) { continue; }
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
        for index in &segments {
            let (bounds, segment) = &targets.segments[*index];
            if !bounds.inflate(acquire, acquire).contains(current) { continue; }
            let distance = if let Some(dir) = direction {
                let Some(q) = along_segment(*segment, current, dir, acquire) else { continue };
                q.distance(current)
            } else {
                let nearest = segment.nearest(current, 1e-8);
                if !nearest.distance_sq.is_finite() || nearest.distance_sq > acquire * acquire { continue; }
                nearest.distance_sq.sqrt()
            };
            let candidate = MoveLock { source: *source, target: MoveTarget::Segment(*segment) };
            let score = distance;
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
        // A held alignment also gets the 12.8px release radius along the drag line.
        for (is_x, cell, component) in [(true, x_lock, u.x), (false, y_lock, u.y)] {
            if component.abs() <= 1e-9 { cell.set(None); continue; }
            if let Some(lock) = cell.get() {
                let current = lock.source + d;
                let amount = (lock.value - if is_x { current.x } else { current.y }) / component;
                if amount.abs() <= release && best.is_none_or(|b| amount.abs() < b.0.abs()) {
                    best = Some((amount, is_x, lock));
                } else { cell.set(None); }
            }
        }
        let held = best.is_some();
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
                    if !held && amount.abs() <= acquire && best.is_none_or(|b| amount.abs() < b.0.abs()) {
                        best = Some((amount, is_x, candidate));
                    }
                }
                for ruler in targets.rulers.iter().filter(|r| r.vertical == is_x) {
                    if let Some((value, from, kind)) = ruler.line_at(current, acquire) {
                        let amount = (value - v) / component;
                        let candidate = AxisLock { source: *source, value, from, kind };
                        if !held && amount.abs() <= acquire && best.is_none_or(|b| amount.abs() < b.0.abs()) {
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
                if !held && amount.abs() <= acquire && best.is_none_or(|b| amount.abs() < b.0.abs()) {
                    best = Some((amount, ruler.vertical, candidate));
                }
            }
        }
        if let Some((amount, is_x, lock)) = best {
            x_lock.set(is_x.then_some(lock));
            y_lock.set((!is_x).then_some(lock));
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
        x_lock.set(None);
        y_lock.set(None);
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
                ruler.line_at(current, acquire).filter(|(value,_,_)| (value-current.x).abs()<=acquire)
                    .map(|(value, from, kind)| ((value - current.x).abs(), AxisLock { source: *source, value, from, kind }))
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
                ruler.line_at(current, acquire).filter(|(value,_,_)| (value-current.y).abs()<=acquire)
                    .map(|(value, from, kind)| ((value - current.y).abs(), AxisLock { source: *source, value, from, kind }))
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
            edges: vec![],
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
        // Shift's geometry source is the grabbed point; grab the moving vertex itself.
        let (d, _) = snapper.snap_with_tolerance(
            Point::new(200.0, 100.0), Vec2::new(99.0, 0.0), false, 8.0, 12.8, Some(Vec2::new(99.0, 0.0)),
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
            edges: vec![],
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

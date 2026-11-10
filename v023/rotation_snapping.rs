/// R-tool rotation targets are edge directions, not snapped mouse positions.
/// Geometry is immutable for the drag; the current screen controls acquisition.
pub(crate) struct LeoRotationSnap {
    origin: Point,
    sources: Vec<Line>,
    targets: Vec<(f64,LeoExtension)>,
    radius: f64,
    held: Cell<Option<RotationHit>>,
}

#[derive(Clone,Copy)]
struct RotationHit {
    source: Line,
    target: LeoExtension,
    angle: f64,
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
        if targets.is_empty() {return None;}
        Some(Self {origin,sources,targets,radius,held:Cell::new(None)})
    }

    pub(crate) fn snap(&self,cx: &ToolContext,raw: f64,pointer: Point,shift: bool) -> Option<(Affine,f64,Vec<Overlay>)> {
        if shift || !raw.is_finite() || !pointer.is_finite() {
            self.held.set(None);return None;
        }
        let lever=self.radius.max(pointer.distance(self.origin));
        if !lever.is_finite() || lever<=1e-9 {self.held.set(None);return None;}
        let acquire=cx.tol(ACQUIRE_PX);let release=acquire*RELEASE_MULTIPLIER;
        let travel=|angle: f64| 2.0*lever*((turn_near(angle,raw)-raw)*0.5).sin().abs();
        let mut hit=self.held.get().filter(|h| travel(h.angle)<=release);
        if hit.is_none() {
            let width=2.0*(acquire/(2.0*lever)).clamp(0.0,1.0).asin();
            let mut best: Option<(f64,f64,RotationHit)>=None;
            for source in &self.sources {
                let raw_matrix=self.matrix(raw);
                if !LeoExtension::new(raw_matrix*source.p0,raw_matrix*source.p1).is_some_and(|l| l.reference_in_view(cx.screen.as_ref())) {continue;}
                let source_angle=line_angle(source.p1-source.p0);
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
                            best=Some((distance,offset,RotationHit {source:*source,target,angle}));
                        }
                    }
                }
            }
            hit=best.map(|(_,_,h)| h);
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
            for (p,q) in [(a,b),(hit.target.at,hit.target.end)] {
                let length=p.distance(q);
                let q=if length>span {p+(q-p)*(span/length)} else {q};
                let mut path=BezPath::new();path.move_to(p);path.line_to(q);
                guides.push(Overlay::Path {path,color:[0x20,0x24,0x2b],width:3.5,dashed:true});
                guides.push(Overlay::Line {a:p,b:q,color:EXTENSION_HIT,dashed:true});
            }
        }
        if cx.anchor_path_labels {
            let at=if a.distance(self.origin)>b.distance(self.origin) {a} else {b};
            guides.push(Overlay::Label {p:at,text:if collinear {"旋转吸附 · 共线"} else {"旋转吸附 · 平行"}.into(),color:EXTENSION_HIT});
        }
        Some((matrix,angle,guides))
    }

    fn matrix(&self,angle: f64) -> Affine {
        Affine::translate(self.origin.to_vec2())*Affine::rotate(angle)*Affine::translate(-self.origin.to_vec2())
    }
}

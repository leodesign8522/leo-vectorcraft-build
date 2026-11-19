"""Expose endpoint snap/axis lock and a bounded slide tolerance on native tools."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
def edit(rel,a,b):
    p=root/rel;s=p.read_text(encoding='utf-8')
    if s.count(a)!=1:raise SystemExit('Unexpected interaction anchor: '+rel+' '+a[:70])
    p.write_text(s.replace(a,b,1),encoding='utf-8',newline='\n')
ext='crates/tools/src/addons/extend_path.rs'
edit(ext,'    cx.doc.walk(|n| {','    cx.doc.walk(|n| {\n        if !cx.doc.is_editable(n.id) || !crate::direct::in_scope(cx.doc,cx.isolation,n.id) {return;}')
edit(ext,'    mode: Mode,','    mode: Mode,\n    direction_locked: bool,\n    snap_point: Option<Point>,\n    snap_distance: f64,')
edit(ext,'                let length = self.length_at(cx, &end, ev.pos, ev.mods);','''                let axis=extension_axis(&end);
                let locked=self.direction_locked || ev.mods.shift;
                let pointer=if locked {end.p+axis*((ev.pos-end.p).dot(axis))} else {ev.pos};
                let length = self.length_at(cx, &end, pointer, ev.mods);
                self.snap_distance=cx.point_tol();
                self.snap_point=extend_open(&end.sp,end.start,length,self.mode.0)
                    .and_then(|sp|if end.start {sp.anchors.first().copied()} else {sp.anchors.last().copied()})
                    .and_then(|a|endpoint_target(cx,&end,a.p,locked,axis));''')
edit(ext,'    fn params(&self, end: &End, length: f64) -> Value {\n        json!({','    fn params(&self, end: &End, length: f64) -> Value {\n        let mut params=json!({')
edit(ext,'            "mode": self.mode.0.id(),\n        })','''            "mode": self.mode.0.id(),
        });
        if let Some(p)=self.snap_point {
            params["snapTo"]=json!([p.x,p.y]);
            params["snapDistance"]=json!(self.snap_distance);
        }
        params''')
edit(ext,'                self.drag = Some((end, 0.0));','                self.snap_point=None;\n                self.drag = Some((end, 0.0));')
edit(ext,'            return vec![\n                Overlay::Anchor { p: end.p, color: BLUE, filled: false, size },','''            let mut out=vec![
                Overlay::Anchor { p: end.p, color: BLUE, filled: false, size },''')
edit(ext,'            ];\n        }\n        let Some(end) = &self.hover','''            ];
            if let Some(p)=self.snap_point {out.push(Overlay::Anchor{p,color:crate::guides::MAGENTA,filled:true,size:9.0});}
            return out;
        }
        let Some(end) = &self.hover''')
edit(ext,'        json!({ "mode": self.mode.0.id() })','        json!({ "mode": self.mode.0.id(), "directionLocked":self.direction_locked })')
edit(ext,'    fn set_option(&mut self, key: &str, v: &Value) {','    fn set_option(&mut self, key: &str, v: &Value) {\n        if key=="directionLocked" {self.direction_locked=v.as_bool().unwrap_or(self.direction_locked);}')
edit(ext,'/// The length of `sp` (pt).','''fn extension_axis(end:&End)->vectorcraft_geom::Vec2 {
    let pair=if end.start {end.sp.anchors.first().zip(end.sp.anchors.get(1))}
        else {end.sp.anchors.last().zip(end.sp.anchors.get(end.sp.anchors.len().saturating_sub(2)))};
    let Some((a,b))=pair else {return vectorcraft_geom::Vec2::new(1.,0.)};
    let candidates=if end.start {[a.p-a.h_out,a.p-b.h_in,a.p-b.p]} else {[a.p-a.h_in,a.p-b.h_out,a.p-b.p]};
    candidates.into_iter().find_map(|v|{let n=v.hypot();(n.is_finite() && n>1e-9).then(||v/n)}).unwrap_or(vectorcraft_geom::Vec2::new(1.,0.))
}
fn endpoint_target(cx:&ToolContext,end:&End,p:Point,locked:bool,axis:vectorcraft_geom::Vec2)->Option<Point> {
    let mut best:Option<(f64,Point)>=None;
    cx.doc.walk(|node| {
        if node.id==end.id || !cx.doc.is_editable(node.id) || !crate::direct::in_scope(cx.doc,cx.isolation,node.id) {return;}
        let NodeKind::Path{path,guide:false,..}=&node.kind else {return;};
        for sp in path.subpaths.iter().filter(|sp|!sp.closed) {
            for a in [sp.anchors.first(),sp.anchors.last()].into_iter().flatten() {
                if locked && ((a.p-end.p)-axis*((a.p-end.p).dot(axis))).hypot()>1e-6 {continue;}
                let d=a.p.distance(p);
                if d<=cx.point_tol() && best.is_none_or(|(old,_)|d<old) {best=Some((d,a.p));}
            }
        }
    });
    best.map(|(_,p)|p)
}

/// The length of `sp` (pt).''')
slide='crates/tools/src/addons/slide_anchor.rs'
edit(slide,'    cx.doc.walk(|n| {','    cx.doc.walk(|n| {\n        if !editable(n.id) || !crate::direct::in_scope(cx.doc,cx.isolation,n.id) {return;}')
edit(slide,'#[derive(Default)]\npub struct SlideAnchorTool {','pub struct SlideAnchorTool {\n    tolerance:f64,')
edit(slide,'/// The anchor of an editable path nearest `p` within the point tolerance that can slide.','''impl Default for SlideAnchorTool {fn default()->Self{Self{tolerance:0.2,hover:None,drag:None}}}

fn bounded_slide(g:&Grip,t:f64,tolerance:f64)->f64 {
    let n=g.sp.anchors.len();
    if n<3 || g.anchor>=n {return 0.;}
    let prev=if g.anchor==0 {n-1} else {g.anchor-1};
    let next=if g.anchor+1==n {0} else {g.anchor+1};
    let Some((a,b,c))=g.sp.anchors.get(prev).zip(g.sp.anchors.get(g.anchor)).zip(g.sp.anchors.get(next)).map(|((a,b),c)|(*a,*b,*c)) else {return 0.;};
    let local=SubPath::new(vec![a,b,c],false);
    let allowed=|t|vectorcraft_pathops::addons::pathtools::slide_anchor(&local,1,t).is_some_and(|(_,error)|error<=tolerance);
    if allowed(t) {return t;}
    let (mut lo,mut hi)=(0.,t);
    for _ in 0..16 {let mid=(lo+hi)*0.5;if allowed(mid) {lo=mid;} else {hi=mid;}}
    lo
}

/// The anchor of an editable path nearest `p` within the point tolerance that can slide.''')
edit(slide,'                let t = slide_to(cx, &g, ev.pos, ev.mods);','                let t = bounded_slide(&g,slide_to(cx, &g, ev.pos, ev.mods),self.tolerance*0.999999);')
edit(slide,'"anchor": g.anchor, "t": t','"anchor": g.anchor, "t": t, "maxError":self.tolerance')
edit(slide,'    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: Mods) -> Cursor {','''    fn options(&self)->serde_json::Value {json!({"tolerance":self.tolerance})}
    fn set_option(&mut self,key:&str,value:&serde_json::Value) {
        if key=="tolerance" && let Some(n)=value.as_f64().filter(|n|n.is_finite()) {self.tolerance=n.clamp(0.001,10.);}
    }
    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: Mods) -> Cursor {''')
engine='crates/engine/src/cmd/addons/pathtools.rs'
edit(engine,'(\\"straight\\")} lengthen an open path','(\\"straight\\"), snapTo?: [x,y] (snap the new end within snapDistance), snapDistance?: pt 0..10000 (4)} lengthen an open path')
edit(engine,'(where to: positive, a curve parameter on the segment after the anchor; negative, back from the end of the one before)} move an anchor','(where to: positive, a curve parameter on the segment after the anchor; negative, back from the end of the one before), maxError?: pt (reject a refit beyond this error)} move an anchor')
edit(engine,'    let new = pt::extend_open(&sp, at_start, length, mode).ok_or_else(|| bad(C, "the path is too short to shorten that much"))?;','''    let mut new = pt::extend_open(&sp, at_start, length, mode).ok_or_else(|| bad(C, "the path is too short to shorten that much"))?;
    if let Some(target)=p.get("snapTo") {
        let [x,y]=finite_numbers::<2>(target).ok_or_else(||bad(C,"invalid snap point"))?;
        let distance=number_or(p,"snapDistance",4.,C)?;
        if !distance.is_finite() || !(0.0..=10000.).contains(&distance) {return Err(bad(C,"invalid snap distance"));}
        let a=if at_start {new.anchors.first_mut()} else {new.anchors.last_mut()}.ok_or_else(||bad(C,"missing endpoint"))?;
        let to=Point::new(x,y);let delta=to-a.p;
        if !delta.hypot().is_finite() || delta.hypot()>distance {return Err(bad(C,"snap point is too far away"));}
        a.p=to;a.h_in+=delta;a.h_out+=delta;
        if !a.h_in.is_finite() || !a.h_out.is_finite() {return Err(bad(C,"invalid snapped handle"));}
    }''')
edit(engine,'    set_subpath(s, "Slide Anchor Along Path", (id, si, sp), new)?;','''    if p.get("maxError").is_some() {
        let max=number_or(p,"maxError",0.2,C)?;
        if !max.is_finite() || max<0. || strayed>max {return Err(bad(C,"slide would exceed the shape tolerance"));}
    }
    set_subpath(s, "Slide Anchor Along Path", (id, si, sp), new)?;''')

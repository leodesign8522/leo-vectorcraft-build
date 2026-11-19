//! Conservative command adapters. Interactive brush/slide gestures are separate work.
use super::*;
#[allow(dead_code)]
#[path = "leo_geometry.rs"]
mod geometry;
use geometry::{Cubic, P};

fn point(p: P) -> Point { Point::new(p.x,p.y) }
fn cubic(a: Anchor,b: Anchor) -> Cubic {
    Cubic([a.p,a.h_out,b.h_in,b.p].map(|p|P::new(p.x,p.y)))
}

fn edit_path(s: &mut Session,p: &Value,name: &str,op: impl FnOnce(&mut SubPath)->Result<()>) -> Result<Value> {
    let id=id_param(p,"id").ok_or_else(||bad(name,"missing id"))?;
    let si=usize_opt(p,"subpath",name)?.unwrap_or(0);
    s.edit(name,|d,selection| {
        let n=d.node(id).filter(|_|d.editable()(id)).ok_or_else(||bad(name,"path not editable"))?;
        let NodeKind::Path{path,guide:false,live:None,..}=&n.kind else {
            return Err(bad(name,"requires an expanded artwork path"));
        };
        let mut sp=path.subpaths.get(si).cloned().ok_or_else(||bad(name,"no such subpath"))?;
        if sp.closed {return Err(bad(name,"closed paths are not supported yet"));}
        op(&mut sp)?;
        let dest=path_mut(d,id)?.subpaths.get_mut(si).ok_or_else(||bad(name,"no such subpath"))?;
        *dest=sp;
        // A topology edit must not retain references to shifted anchor indices.
        selection.set([id]);
        Ok(())
    })?;
    ok()
}

pub(super) fn extend(s: &mut Session,p: &Value)->Result<Value> {
    const C:&str="path.leoExtend";
    let length=number_or(p,"length",20.,C)?;
    let mode=match p.get("mode") {None=>"tangent",Some(v)=>v.as_str().ok_or_else(||bad(C,"invalid extension mode"))?};
    let amount=number_or(p,"amount",0.2,C)?;
    if !matches!(mode,"tangent"|"cubic") {return Err(bad(C,"invalid extension mode"));}
    edit_path(s,p,C,|sp| {
        let n=sp.anchors.len();
        let b=sp.anchors.last().copied().ok_or_else(||bad(C,"requires two anchors"))?;
        let a=sp.anchors.get(n.saturating_sub(2)).copied().filter(|_|n>=2).ok_or_else(||bad(C,"requires two anchors"))?;
        let curve=cubic(a,b);
        let result=if mode=="cubic" {
            if amount<=0. || amount>10. {return Err(bad(C,"amount must be in (0,10]"));}
            curve.interval(1.,1.+amount)
        } else {geometry::extend_tangent(curve,length)}.ok_or_else(||bad(C,"degenerate extension"))?;
        let last=sp.anchors.last_mut().ok_or_else(||bad(C,"missing endpoint"))?;
        last.h_out=point(result.0[1]);
        last.kind=AnchorKind::Corner;
        sp.anchors.push(Anchor::with_handles(point(result.0[3]),point(result.0[2]),point(result.0[3])));
        Ok(())
    })
}

fn join(s:&mut Session,p:&Value,reposition:bool)->Result<Value> {
    let name=if reposition {"path.leoReposition"} else {"path.leoSmartRemove"};
    let ai=usize_opt(p,"anchor",name)?.ok_or_else(||bad(name,"missing anchor"))?;
    let tolerance=number_or(p,"tolerance",0.01,name)?;
    let t=number_or(p,"t",0.5,name)?;
    edit_path(s,p,name,|sp| {
        if ai==0 {return Err(bad(name,"requires an interior anchor"));}
        let a=sp.anchors.get(ai-1).copied().ok_or_else(||bad(name,"no previous anchor"))?;
        let b=sp.anchors.get(ai).copied().ok_or_else(||bad(name,"no such anchor"))?;
        let c=sp.anchors.get(ai.saturating_add(1)).copied().ok_or_else(||bad(name,"requires an interior anchor"))?;
        let left=cubic(a,b); let right=cubic(b,c);
        let (l,r)=if reposition {
            geometry::slide_join(left,right,t,tolerance).map(|(l,r)|(l,Some(r)))
        } else {geometry::try_join(left,right,tolerance).map(|(c,_)|(c,None))}
            .ok_or_else(||bad(name,"curves cannot be joined within tolerance"))?;
        // Work on a cloned subpath; validation failures cannot leave partial edits.
        if let Some(previous)=sp.anchors.get_mut(ai-1) { previous.h_out=point(l.0[1]); previous.kind=AnchorKind::Corner; }
        if let Some(r)=r {
            if let Some(middle)=sp.anchors.get_mut(ai) { *middle=Anchor::with_handles(point(l.0[3]),point(l.0[2]),point(r.0[1])); }
            if let Some(next)=sp.anchors.get_mut(ai+1) { next.h_in=point(r.0[2]); next.kind=AnchorKind::Corner; }
        } else {
            if let Some(next)=sp.anchors.get_mut(ai+1) {next.h_in=point(l.0[2]); next.kind=AnchorKind::Corner;}
            sp.anchors.remove(ai);
        }
        Ok(())
    })
}
pub(super) fn remove(s:&mut Session,p:&Value)->Result<Value>{join(s,p,false)}
pub(super) fn reposition(s:&mut Session,p:&Value)->Result<Value>{join(s,p,true)}

pub(super) fn transform_handles(s:&mut Session,p:&Value)->Result<Value> {
    const C:&str="path.leoTransformHandles";
    let operation=str_param(p,"operation").unwrap_or("rotate");
    if !matches!(operation,"rotate"|"scale"|"equal"|"symmetric") {return Err(bad(C,"invalid operation"));}
    let angle=number_or(p,"angle",0.,C)?.to_radians();
    let scale=number_or(p,"scale",1.,C)?;
    if !angle.is_finite() || !scale.is_finite() || !(0.0..=100.0).contains(&scale) {return Err(bad(C,"invalid angle or scale"));}
    let items=p.get("handles").and_then(Value::as_array).filter(|a|!a.is_empty() && a.len()<=50_000).ok_or_else(||bad(C,"select handles first"))?;
    let state=s.doc()?;
    if let Some(value)=p.get("revision") {
        let revision=value.as_u64().ok_or_else(||bad(C,"invalid revision"))?;
        if revision!=state.revision {return Err(bad(C,"handle selection is stale; select again"));}
    }
    let d=&state.doc;
    let mut seen=BTreeSet::new();
    let mut anchors=BTreeSet::new();
    let mut target_length=None;
    let mut changes=Vec::new();
    for item in items {
        let id=id_param(item,"id").ok_or_else(||bad(C,"missing id"))?;
        let si=usize_opt(item,"subpath",C)?.unwrap_or(0);
        let ai=usize_opt(item,"anchor",C)?.ok_or_else(||bad(C,"missing anchor"))?;
        let side=match str_param(item,"which") {Some("in")=>false,Some("out")=>true,_=>return Err(bad(C,"invalid handle side"))};
        if !seen.insert((id,si,ai,side)) {return Err(bad(C,"duplicate handle"));}
        let n=d.node(id).filter(|_|d.is_editable(id)).ok_or_else(||bad(C,"path not editable"))?;
        let NodeKind::Path{path,guide:false,..}=&n.kind else {return Err(bad(C,"not an artwork path"));};
        let a=path.subpaths.get(si).and_then(|sp|sp.anchors.get(ai)).ok_or_else(||bad(C,"no such anchor"))?;
        let q=if side {a.h_out} else {a.h_in};
        if let (Some(x),Some(y))=(item.get("x"),item.get("y")) {
            let old=Point::new(x.as_f64().ok_or_else(||bad(C,"invalid x"))?,y.as_f64().ok_or_else(||bad(C,"invalid y"))?);
            if !old.is_finite() || old.distance(q)>1e-6 {return Err(bad(C,"handle selection is stale; select again"));}
        }
        let v=q-a.p;
        let length=v.hypot();
        let equal=*target_length.get_or_insert(length);
        let to=match operation {
            "rotate"=>a.p+Vec2::new(v.x*angle.cos()-v.y*angle.sin(),v.x*angle.sin()+v.y*angle.cos()),
            "scale"=>a.p+v*scale,
            "equal" if length>1e-12=>a.p+v*(equal/length),
            "equal" if equal<=1e-12=>a.p,
            "equal"=>return Err(bad(C,"a zero handle has no direction")),
            _=>q,
        };
        if !to.is_finite() {return Err(bad(C,"handle result is not finite"));}
        if operation=="symmetric" && !anchors.insert((id,si,ai)) {continue;}
        changes.push(json!({"id":id.0,"subpath":si,"anchor":ai,"which":if side {"out"} else {"in"},"x":to.x,"y":to.y}));
        if operation=="symmetric" {
            let other=a.p+(a.p-to);
            if !other.is_finite() {return Err(bad(C,"handle result is not finite"));}
            changes.push(json!({"id":id.0,"subpath":si,"anchor":ai,"which":if side {"in"} else {"out"},"x":other.x,"y":other.y}));
        }
    }
    super::leo_set_handles(s,&json!({"handles":changes,"independent":operation!="symmetric"}))
}

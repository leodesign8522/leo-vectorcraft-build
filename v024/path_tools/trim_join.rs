//! Intersection-based trimming. Uncut cubic pieces are subdivided, never refitted.
use super::*;
use kurbo::ParamCurve;
use vectorcraft_pathops::addons::pathtools as pt;

const C: &str = "path.leoTrimJoin";
const EPS: f64 = 1e-6;
const JOIN: f64 = 1e-3;
const MAX_SEGMENTS: usize = 512;

#[derive(Clone)]
struct Piece { owner: NodeId, sp: SubPath, cut: [bool;2] }

fn within(d:&Document,root:Option<NodeId>,mut id:NodeId)->bool {
    let Some(root)=root else {return true};
    for _ in 0..256 {
        if id==root {return true;}
        let Some(parent)=d.parent_of(id) else {return false};
        id=parent;
    }
    false
}

/// Extract exactly the original cubics over [from,to].
fn interval(sp:&SubPath,from:f64,to:f64)->SubPath {
    let mut anchors:Vec<Anchor>=vec![];
    for i in 0..sp.segment_count() {
        let lo=(from-i as f64).clamp(0.,1.);
        let hi=(to-i as f64).clamp(0.,1.);
        if hi-lo<=EPS {continue;}
        let c=sp.segment(i).subsegment(lo..hi);
        if sp.segment_is_line(i) {
            if anchors.is_empty() {anchors.push(Anchor::corner(c.p0));}
            anchors.push(Anchor::corner(c.p3));
        } else {
            if let Some(a)=anchors.last_mut() {a.h_out=c.p1;}
            else {anchors.push(Anchor::with_handles(c.p0,c.p0,c.p1));}
            anchors.push(Anchor::with_handles(c.p3,c.p2,c.p3));
        }
    }
    SubPath::new(anchors,false)
}

fn compatible(d:&Document,a:NodeId,b:NodeId)->bool {
    if a==b {return true;}
    let (Some(a_node),Some(b_node))=(d.node(a),d.node(b)) else {return false};
    d.parent_of(a)==d.parent_of(b) && a_node.appearance==b_node.appearance
        && a_node.opacity==b_node.opacity && a_node.blend==b_node.blend
        && a_node.extras==b_node.extras
}

fn join_pieces(d:&Document,pieces:&mut Vec<Piece>)->usize {
    let mut joined=0;
    loop {
        let mut chosen=None;
        'find: for (i,a) in pieces.iter().enumerate() {
            if a.sp.closed {continue;}
            for ea in 0..2 {
                if !a.cut[ea] {continue;}
                let Some(anchor)=(if ea==0 {a.sp.anchors.first()} else {a.sp.anchors.last()}) else {continue};
                // A junction must have exactly two remaining ends. Never guess at a branch.
                let mut ends=vec![];
                for (j,b) in pieces.iter().enumerate().filter(|(_,p)|!p.sp.closed) {
                    for (eb,end) in [(0,b.sp.anchors.first()),(1,b.sp.anchors.last())] {
                        if let Some(end)=end {if end.p.distance(anchor.p)<=JOIN {ends.push((j,eb));}}
                    }
                }
                if ends.len()!=2 {continue;}
                let Some(&(j,eb))=ends.iter().find(|(j,_)|*j!=i) else {continue};
                let Some(b)=pieces.get(j) else {continue};
                if !compatible(d,a.owner,b.owner) {continue;}
                chosen=Some((i,ea,j,eb));break 'find;
            }
        }
        let Some((i,ea,j,eb))=chosen else {break};
        let (Some(mut a),Some(mut b))=(pieces.get(i).cloned(),pieces.get(j).cloned()) else {break};
        if ea==0 {a.sp.reverse();a.cut.reverse();}
        if eb==1 {b.sp.reverse();b.cut.reverse();}
        let (Some(last),Some(first))=(a.sp.anchors.last_mut(),b.sp.anchors.first()) else {break};
        last.h_out=first.h_out+(last.p-first.p);last.kind=AnchorKind::Corner;
        a.sp.anchors.extend(b.sp.anchors.into_iter().skip(1));a.cut[1]=b.cut[1];
        pieces.remove(i.max(j));pieces.remove(i.min(j));pieces.push(a);joined+=1;
    }
    joined
}

pub(super) fn run(s:&mut Session,p:&Value)->Result<Value> {
    let stroke=p.get("points").and_then(Value::as_array).filter(|a|a.len()<=512)
        .ok_or_else(||bad(C,"points must contain at most 512 coordinate pairs"))?;
    let points:Vec<Point>=stroke.iter().map(|v| {
        let pair=v.as_array().filter(|a|a.len()==2).ok_or_else(||bad(C,"invalid stroke point"))?;
        let xy=pair.iter().map(|v|v.as_f64().filter(|x|x.is_finite() && x.abs()<=1e8)).collect::<Option<Vec<_>>>()
            .ok_or_else(||bad(C,"invalid stroke coordinate"))?;
        match xy.as_slice() {[x,y]=>Ok(Point::new(*x,*y)),_=>Err(bad(C,"invalid stroke point"))}
    }).collect::<Result<_>>()?;
    let join=switch_or(p,"join",true,C)?;
    let prefer=switch_or(p,"preferSelected",true,C)?;
    if points.len()<2 {return Ok(json!({"trimmed":0,"joined":0}));}
    let state=s.doc()?;let d=&state.doc;
    let mut ids=vec![];
    {
    let editable=d.editable();
    d.walk(|n| {
        if !editable(n.id) || !within(d,state.isolation,n.id) {return;}
        // Width/brush profiles and live effects need separate style-preserving splitting.
        let plain=n.appearance.effects.is_empty() && n.appearance.items.iter().all(|item|match item {
            vectorcraft_doc::AppearanceItem::Stroke(st)=>st.profile.is_none() && st.brush.is_none()
                && st.pressure.is_none() && st.tilt.is_none() && st.bearing.is_none() && st.rotation.is_none()
                && st.effects.is_empty() && st.start_arrow.is_none() && st.end_arrow.is_none(),
            vectorcraft_doc::AppearanceItem::Fill(fill)=>fill.effects.is_empty(),
        });
        if plain {if let NodeKind::Path{guide:false,live:None,clipping:false,..}=&n.kind {ids.push(n.id);}}
    });
    } // Release the artboard-lock cache before taking mutable document access.
    let selected=|id:NodeId|state.selection.objects.iter().any(|root|within(d,Some(*root),id));
    // Selection limits what is trimmed, while other editable artwork still supplies boundaries.
    let selected_only=prefer && ids.iter().any(|id|selected(*id));
    let paths:Vec<PathData>=ids.iter().filter_map(|id|d.node(*id)?.path_data().cloned()).collect();
    let count:usize=paths.iter().flat_map(|p|&p.subpaths).map(SubPath::segment_count).sum();
    if count>MAX_SEGMENTS {return Err(bad(C,"preview supports at most 512 editable path segments"));}
    let refs:Vec<&PathData>=paths.iter().collect();
    let places=pt::crossings(&refs,true).ok_or_else(||bad(C,"intersection complexity limit exceeded"))?;
    if places.iter().map(Vec::len).sum::<usize>()>8192 {return Err(bad(C,"too many intersections for this preview"));}
    let gesture=PathData::single(SubPath::polyline(&points,false));
    let mut pieces=vec![];let mut trimmed=0;
    for (index,path) in paths.iter().enumerate() {
        let Some(&owner)=ids.get(index) else {return Err(bad(C,"path enumeration changed"));};
        let hits=pt::crossings(&[path,&gesture],false).ok_or_else(||bad(C,"stroke complexity limit exceeded"))?;
        for (si,sp) in path.subpaths.iter().enumerate() {
            let end=sp.segment_count() as f64;
            let mut cuts:Vec<f64>=places.get(index).into_iter().flatten().filter(|(sub,_,_)|*sub==si)
                .map(|(_,seg,t)|*seg as f64+t).filter(|t|*t>EPS && *t<end-EPS).collect();
            cuts.sort_by(f64::total_cmp);cuts.dedup_by(|a,b|(*a-*b).abs()<EPS);
            if sp.closed || cuts.is_empty() || (selected_only && !selected(owner)) {
                pieces.push(Piece{owner,sp:sp.clone(),cut:[false,false]});continue;
            }
            cuts.insert(0,0.);cuts.push(end);
            let stroke_hits:Vec<f64>=hits.first().into_iter().flatten().filter(|(sub,_,_)|*sub==si)
                .map(|(_,seg,t)|*seg as f64+t).collect();
            let gone:Vec<bool>=cuts.windows(2).map(|w|match w {
                [a,b]=>stroke_hits.iter().any(|t|*t>*a+EPS && *t<*b-EPS),_=>false
            }).collect();
            if !gone.contains(&true) {pieces.push(Piece{owner,sp:sp.clone(),cut:[false,false]});continue;}
            trimmed+=gone.iter().filter(|v|**v).count();
            let mut k=0;
            while k<gone.len() {
                if gone.get(k)==Some(&true) {k+=1;continue;}
                let start=k;
                while k<gone.len() && gone.get(k)==Some(&false) {k+=1;}
                if let (Some(&from),Some(&to))=(cuts.get(start),cuts.get(k)) {
                    let part=interval(sp,from,to);
                    if part.anchors.len()>=2 {pieces.push(Piece{owner,sp:part,cut:[from>EPS,to<end-EPS]});}
                }
            }
        }
    }
    if trimmed==0 {return Ok(json!({"trimmed":0,"joined":0}));}
    // Endpoint ambiguity checks scan remaining pieces; bound that work before editing.
    if pieces.len()>2048 || (join && pieces.len()>256) {
        return Err(bad(C,"too many remaining pieces for this preview; disable joining or work on a smaller drawing"));
    }
    let joined=if join {join_pieces(d,&mut pieces)} else {0};
    let mut updates=vec![];
    for (id,old) in ids.iter().zip(paths.iter()) {
        let new=PathData::new(pieces.iter().filter(|piece|piece.owner==*id).map(|piece|piece.sp.clone()).collect());
        if new!=*old {updates.push((*id,new));}
    }
    s.edit("Trim and Join",|d,sel| {
        let mut result=vec![];
        for (id,path) in updates {
            if path.subpaths.is_empty() {d.remove(id)?;}
            else {*super::path_mut(d,id)?=path;result.push(id);}
        }
        sel.set(result);Ok(())
    })?;
    Ok(json!({"trimmed":trimmed,"joined":joined}))
}

"""Extend the native handle feedback patch with additive multi-handle editing."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
path=root/"crates/tools/src/direct.rs"
s=path.read_text(encoding="utf-8")
def edit(a,b):
    global s
    if s.count(a)!=1: raise SystemExit("Unexpected multi-handle anchor: "+a)
    s=s.replace(a,b,1)
edit("    leo_handle_selected: Option<((NodeId, usize, usize, bool), Point, u64)>,",
"""    leo_handles: Vec<((NodeId,usize,usize,bool),Point)>,
    leo_handle_revision: u64,
    leo_handle_toggle: bool,
    leo_handle_dragged: bool,""")
edit("            leo_handle_selected: None,",
"""            leo_handles: vec![],
            leo_handle_revision: 0,
            leo_handle_toggle: false,
            leo_handle_dragged: false,""")
edit("            self.leo_handle_selected = None;\n            self.leo_handle_hover = None;\n        }\n        // Up and Down",
"""            self.leo_handles.clear();
            self.leo_handle_hover = None;
        }
        // Up and Down""")
edit("""            self.leo_handle_selected = None;
            self.leo_handle_hover = None;
        }
        if ev.kind == PointerKind::Move""",
"""            self.leo_handle_hover = None;
            self.leo_handle_toggle = false;
            self.leo_handle_dragged = false;
            if ev.kind == PointerKind::DoubleClick || self.group
                || self.leo_handle_revision != cx.revision.1
                || self.leo_handles.iter().any(|(key,p)| leo_handle_position(cx,*key).is_none_or(|q|q.distance(*p)>1e-6))
                || hit_handle(cx,ev.pos,cx.point_tol()).is_none() {
                self.leo_handles.clear();
            }
        }
        if ev.kind == PointerKind::Move""")
edit("""                if let Some((id, si, ai, out)) = hit_handle(cx, p, point) {
                    self.state = State::Handle { id, si, ai, out };""",
"""                if let Some((id, si, ai, out)) = hit_handle(cx, p, point) {
                    let key=(id,si,ai,out);
                    let ordinary=|id|cx.doc.node(id).is_some_and(|n|matches!(&n.kind,NodeKind::Path{guide:false,..}));
                    let selected=self.leo_handles.iter().any(|(k,_)|*k==key);
                    // Shift-click adds/removes handles. Dragging a selected handle
                    // retains all picks; Shift during drag still locks its angle.
                    self.leo_handle_toggle=ev.mods.shift && selected;
                    if (!ev.mods.shift && !selected) || !ordinary(id)
                        || self.leo_handles.iter().any(|(k,_)|!ordinary(k.0)) {
                        self.leo_handles.clear();
                    }
                    if let Some(q)=leo_handle_position(cx,key) {
                        if !self.leo_handles.iter().any(|(k,_)|*k==key) {
                            self.leo_handles.push((key,q));
                        }
                    }
                    self.leo_handle_revision=cx.revision.1;
                    self.state = State::Handle { id, si, ai, out };""")
edit("""                let cmd = if cx.doc.node(id).is_some_and(is_path_type) { "text.setPathHandle" } else { "path.setHandle" };""",
"""                self.leo_handle_dragged=true;
                if self.leo_handles.len()>1 {
                    if let Some((_,origin))=self.leo_handles.iter().find(|(k,_)|*k==(id,si,ai,out)) {
                        let delta=q-*origin;
                        let handles:Vec<Value>=self.leo_handles.iter().map(|((id,si,ai,out),from)|{
                            let p=*from+delta;
                            json!({"id":id.0,"subpath":si,"anchor":ai,"which":if *out {"out"} else {"in"},"x":p.x,"y":p.y})
                        }).collect();
                        return vec![Action::Preview("path.leoSetHandles".into(),json!({"handles":handles,"independent":ev.mods.alt}))];
                    }
                }
                let cmd = if cx.doc.node(id).is_some_and(is_path_type) { "text.setPathHandle" } else { "path.setHandle" };""")
edit("""                self.leo_handle_selected = leo_handle_position(cx,key).map(|p|(key,p,cx.revision.1));""",
"""                if self.leo_handle_toggle && !self.leo_handle_dragged {
                    self.leo_handles.retain(|(k,_)|*k!=key);
                }
                self.leo_handles=self.leo_handles.iter().filter_map(|(k,_)|leo_handle_position(cx,*k).map(|p|(*k,p))).collect();
                self.leo_handle_revision=cx.revision.1;""")
edit("""                    let active = match self.state {
                        State::Handle { id, si, ai, out } => leo_handle_position(cx,(id,si,ai,out)).map(|p|((id,si,ai,out),p)),
                        State::Idle => self.leo_handle_selected.and_then(|(key,old,revision)| {
                            if revision != cx.revision.1 { return None; }
                            let p=leo_handle_position(cx,key)?;
                            (p.distance(old)<1e-6).then_some((key,p))
                        }),
                        _ => None,
                    };
                    if matches!(self.state, State::Idle) {
                        if let Some(key)=self.leo_handle_hover.filter(|k|active.is_none_or(|(a,_)| a!=*k)) {
                            if let Some(p)=leo_handle_position(cx,key) { leo_handle_marker(&mut out,p,false); }
                        }
                    }
                    if let Some((_,p))=active { leo_handle_marker(&mut out,p,true); }""",
"""                    let dragging=matches!(self.state,State::Handle{..});
                    let active:Vec<_>=if dragging || (matches!(self.state,State::Idle) && self.leo_handle_revision==cx.revision.1) {
                        self.leo_handles.iter().filter_map(|(k,old)| {
                            let p=leo_handle_position(cx,*k)?;
                            (dragging || p.distance(*old)<1e-6).then_some((*k,p))
                        }).collect()
                    } else {vec![]};
                    if matches!(self.state,State::Idle) {
                        if let Some(key)=self.leo_handle_hover.filter(|k|!active.iter().any(|(a,_)|a==k)) {
                            if let Some(p)=leo_handle_position(cx,key) { leo_handle_marker(&mut out,p,false); }
                        }
                    }
                    for (_,p) in active {leo_handle_marker(&mut out,p,true);}""")
path.write_text(s,encoding="utf-8",newline="\n")

path=root/"crates/engine/src/cmd/path.rs"
s=path.read_text(encoding="utf-8")
edit('        cmd!(\n            "path.moveAnchor",',
'''        cmd!(
            "path.leoSetHandles",
            "Leo Design Move Handles",
            [],
            None,
            "{handles: [{id,subpath,anchor,which,x,y}], independent?:bool}",
            has_doc,
            leo_set_handles
        ),
        cmd!(
            "path.moveAnchor",''')
edit('fn set_handle(s: &mut Session, p: &Value) -> Result<Value> {',
'''// One transaction per preview: all positions are absolute from the initial
// press, so there is no accumulated drift and Undo restores the full selection.
fn leo_set_handles(s:&mut Session,p:&Value)->Result<Value>{
    const C:&str="path.leoSetHandles";
    let items=p.get("handles").and_then(Value::as_array).ok_or_else(||bad(C,"missing handles"))?;
    if items.is_empty() || items.len()>100_000 {return Err(bad(C,"invalid handle count"));}
    let independent=switch_or(p,"independent",false,C)?;
    let mut changes=std::collections::BTreeMap::<(NodeId,usize,usize),[Option<Point>;2]>::new();
    for v in items {
        let id=id_param(v,"id").ok_or_else(||bad(C,"missing id"))?;
        let si=usize_opt(v,"subpath",C)?.unwrap_or(0);
        let ai=v.get("anchor").and_then(Value::as_u64).map(index).ok_or_else(||bad(C,"missing anchor"))?;
        let side=match str_param(v,"which"){Some("in")=>0,Some("out")=>1,_=>return Err(bad(C,"invalid which"))};
        let pos=Point::new(f64_req(v,"x",C)?,f64_req(v,"y",C)?);
        let a=changes.entry((id,si,ai)).or_insert([None,None]);
        if a[side].replace(pos).is_some(){return Err(bad(C,"duplicate handle"));}
    }
    s.edit("Move Handles",|d,_|{
        // Preflight every target before modifying any of them.
        {
            let editable=d.editable();
            for &(id,si,ai) in changes.keys(){
                let n=d.node(id).filter(|_|editable(id)).ok_or_else(||bad(C,"path not editable"))?;
                let NodeKind::Path{path,guide:false,..}=&n.kind else{return Err(bad(C,"not an editable artwork path"));};
                if !path.subpaths.get(si).is_some_and(|sp|ai<sp.anchors.len()){return Err(bad(C,"no such anchor"));}
            }
        }
        for ((id,si,ai),[inside,outside]) in changes {
            let a=path_mut(d,id)?.anchor_mut(si,ai).ok_or_else(||bad(C,"no such anchor"))?;
            match (inside,outside){
                (Some(i),Some(o))=>{
                    // Two explicitly selected handles have priority over smooth coupling.
                    let old_kind=a.kind;
                    *a=Anchor::with_handles(a.p,i,o);
                    if independent || old_kind==AnchorKind::Corner {a.kind=AnchorKind::Corner;}
                },
                (Some(i),None)=>a.set_handle(false,i,independent),
                (None,Some(o))=>a.set_handle(true,o,independent),
                _=>{}
            }
        }
        Ok(())
    })?;
    ok()
}

fn set_handle(s: &mut Session, p: &Value) -> Result<Value> {''')
path.write_text(s,encoding="utf-8",newline="\n")
with (root/"crates/tools/src/direct/tests.rs").open("a",encoding="utf-8") as f:
    f.write((Path(__file__).parent/"multi_handle_tests.rs").read_text(encoding="utf-8"))

//! Drag over unwanted portions; the engine cuts at artwork intersections.
use serde_json::{json,Value};
use vectorcraft_geom::{BezPath,Point};
use crate::{Action,Cursor,Mods,Overlay,PointerEvent,PointerKind,Tool,ToolContext,ToolKey,previews_at};

pub const ID:&str="leoTrimJoin";
const COMMAND:&str="path.leoTrimJoin";
const COLORS:[[u8;3];6]=[[224,48,48],[79,157,255],[220,70,200],[90,190,80],[40,200,210],[20,20,20]];

pub struct TrimJoinTool {
    points:Vec<Point>,painting:bool,rubber:Option<usize>,
    join:bool,prefer_selected:bool,force:Option<bool>,color:usize,
}
impl Default for TrimJoinTool {
    fn default()->Self {Self{points:vec![],painting:false,rubber:None,join:true,prefer_selected:true,force:None,color:0}}
}
impl TrimJoinTool {
    fn params(&self)->Value {json!({"points":self.points.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),
        "join":self.force.unwrap_or(self.join),"preferSelected":self.prefer_selected})}
    fn preview(&self)->Action {Action::Preview(COMMAND.into(),self.params())}
    fn sample(&mut self,p:Point,mods:Mods) {
        if mods.alt {
            let i=*self.rubber.get_or_insert(self.points.len().saturating_sub(1));
            let Some(&base)=self.points.get(i) else {return};
            let mut to=p;
            if mods.shift {
                let v=p-base;let step=std::f64::consts::FRAC_PI_4;
                let angle=(v.y.atan2(v.x)/step).round()*step;
                to=base+kurbo::Vec2::new(angle.cos(),angle.sin())*v.hypot();
            }
            self.points.truncate(i+1);self.points.push(to);
        } else {
            self.rubber=None;
            if self.points.last().is_none_or(|q|q.distance(p)>1e-9) {self.points.push(p);}
        }
    }
}
impl Tool for TrimJoinTool {
    fn id(&self)->&'static str {ID}
    fn busy(&self)->bool {self.painting}
    fn wants_samples(&self)->bool {true}
    fn pointer(&mut self,_cx:&ToolContext,ev:&PointerEvent)->Vec<Action> {
        match ev.kind {
            PointerKind::Down=>{
                self.points=vec![ev.pos];self.painting=true;self.rubber=None;self.force=None;
                vec![Action::Begin("Trim and Join".into())]
            },
            PointerKind::Drag|PointerKind::Up if self.painting=>{
                // Refuse an oversized gesture as a whole, never commit a truncated stroke.
                if self.points.len()>=512 {self.painting=false;self.points.clear();return vec![Action::Cancel];}
                self.sample(ev.pos,ev.mods);
                if ev.kind==PointerKind::Up {
                    self.painting=false;let actions=vec![self.preview(),Action::Commit];self.points.clear();actions
                } else if previews_at(self.points.len()) {vec![self.preview()]} else {vec![]}
            },
            _=>vec![],
        }
    }
    fn key(&mut self,_cx:&ToolContext,key:ToolKey,_mods:Mods)->Vec<Action> {
        match key {
            ToolKey::Escape if self.painting=>{self.painting=false;self.points.clear();vec![Action::Cancel]},
            ToolKey::Letter('j') if self.painting=>{self.force=Some(true);vec![self.preview()]},
            ToolKey::Letter('n') if self.painting=>{self.force=Some(false);vec![self.preview()]},
            ToolKey::Letter('c') if self.painting=>{self.color=(self.color+1)%COLORS.len();vec![]},
            _=>vec![],
        }
    }
    fn claims_key(&self,_cx:&ToolContext,key:ToolKey)->bool {
        self.painting && matches!(key,ToolKey::Escape|ToolKey::Letter('j'|'n'|'c'))
    }
    fn overlays(&self,_cx:&ToolContext)->Vec<Overlay> {
        let Some((&first,rest))=self.points.split_first().filter(|_|self.painting) else {return vec![]};
        let mut path=BezPath::new();path.move_to(first);for &p in rest {path.line_to(p);}
        vec![Overlay::Path{path,color:COLORS.get(self.color).copied().unwrap_or(COLORS[0]),width:1.5,dashed:true}]
    }
    fn cursor(&self,_cx:&ToolContext,_p:Point,_mods:Mods)->Cursor {Cursor::Crosshair}
    fn options(&self)->Value {json!({"join":self.join,"preferSelected":self.prefer_selected,"forceJoin":self.force})}
    fn set_option(&mut self,key:&str,v:&Value) {
        match key {"join"=>self.join=v.as_bool().unwrap_or(self.join),
            "preferSelected"=>self.prefer_selected=v.as_bool().unwrap_or(self.prefer_selected),_=>{}}
    }
}

#[cfg(test)]
mod tests {
    use super::*;use crate::testutil::*;use vectorcraft_doc::Selection;
    #[test]
    fn leo_path_tools_trim_gesture_join_override_rubberband_and_cancel() {
        let (d,_)=doc_with_rect();let sel=Selection::default();let paint=paint();let cx=cx(&d,&sel,&paint);
        let mut tool=TrimJoinTool::default();
        assert!(matches!(tool.pointer(&cx,&PointerEvent::new(PointerKind::Down,0.,0.)).as_slice(),[Action::Begin(_)]));
        let mods=Mods{alt:true,shift:true,..Mods::default()};
        tool.pointer(&cx,&PointerEvent::new(PointerKind::Drag,100.,40.).with_mods(mods));
        assert_eq!(tool.points.len(),2);assert!(tool.points[1].y.abs()<1e-8);
        tool.pointer(&cx,&PointerEvent::new(PointerKind::Drag,200.,60.).with_mods(mods));
        assert_eq!(tool.points.len(),2);
        assert!(tool.claims_key(&cx,ToolKey::Letter('n')));
        tool.key(&cx,ToolKey::Letter('n'),Mods::default());assert_eq!(tool.params()["join"],false);
        tool.key(&cx,ToolKey::Letter('j'),Mods::default());assert_eq!(tool.params()["join"],true);
        assert_eq!(tool.key(&cx,ToolKey::Escape,Mods::default()),vec![Action::Cancel]);assert!(!tool.busy());
        tool.pointer(&cx,&PointerEvent::new(PointerKind::Down,0.,0.));
        assert!(matches!(tool.pointer(&cx,&PointerEvent::new(PointerKind::Up,100.,100.)).as_slice(),[Action::Preview(_,p),Action::Commit] if p["join"]==true));
    }
}


#[cfg(test)]
mod leo_path_tools_trim_join_regressions {
    use super::*;
    use kurbo::ParamCurve;
    fn line(s:&mut Session,a:[f64;2],b:[f64;2])->NodeId {
        NodeId(s.execute("path.create",&json!({"anchors":[{"x":a[0],"y":a[1]},{"x":b[0],"y":b[1]}],"closed":false})).unwrap()["id"].as_u64().unwrap())
    }
    fn path(s:&Session,id:NodeId)->vectorcraft_geom::PathData {
        s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone()
    }
    fn cross(s:&mut Session)->(NodeId,NodeId) {
        (line(s,[-100.,-100.],[100.,100.]),line(s,[-100.,100.],[100.,-100.]))
    }
    fn trim(s:&mut Session,join:bool)->serde_json::Value {
        s.execute("path.leoTrimJoin",&json!({"points":[[50,-120],[50,120]],"join":join,"preferSelected":false})).unwrap()
    }
    #[test]
    fn x_trims_at_artwork_intersection_joins_and_undoes_once() {
        let mut s=session();let (a,b)=cross(&mut s);let before=s.doc().unwrap().doc.clone();
        let history=s.doc().unwrap().history.undo.len();let r=trim(&mut s,true);
        assert_eq!(r["trimmed"],2);assert_eq!(r["joined"],1);
        let nodes:Vec<_>=[a,b].into_iter().filter_map(|id|s.doc().unwrap().doc.node(id)).collect();
        assert_eq!(nodes.len(),1);let sp=&nodes[0].path_data().unwrap().subpaths[0];
        assert_eq!(sp.anchors.len(),3);assert!(sp.anchors[1].p.distance(Point::ZERO)<1e-3);
        assert!(sp.anchors.first().unwrap().p.x<0. && sp.anchors.last().unwrap().p.x<0.);
        assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(s.doc().unwrap().doc,before);
        s.execute("edit.redo",&json!({})).unwrap();assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
    }
    #[test]
    fn no_join_keeps_two_paths_and_selected_preference_uses_unselected_boundaries() {
        let mut s=session();let(a,b)=cross(&mut s);trim(&mut s,false);
        for id in [a,b] {assert!(path(&s,id).subpaths[0].anchors.last().unwrap().p.distance(Point::ZERO)<1e-3);}
        s.execute("edit.undo",&json!({})).unwrap();
        s.execute("select.set",&json!({"ids":[a.0]})).unwrap();let old_b=path(&s,b);
        let r=s.execute("path.leoTrimJoin",&json!({"points":[[50,-120],[50,120]],"preferSelected":true})).unwrap();
        assert_eq!(r["trimmed"],1);assert_eq!(r["joined"],0);assert_eq!(path(&s,b),old_b);
    }
    #[test]
    fn cubic_is_exactly_subdivided_and_middle_interval_becomes_two_subpaths() {
        let mut s=session();let id=NodeId(s.execute("path.create",&json!({"anchors":[
            {"x":0,"y":0,"out":[0,100]},{"x":100,"y":0,"in":[100,100]}],"closed":false})).unwrap()["id"].as_u64().unwrap());
        line(&mut s,[50.,-20.],[50.,100.]);let original=path(&s,id).subpaths[0].segment(0);
        s.execute("path.leoTrimJoin",&json!({"points":[[75,0],[75,100]],"join":false,"preferSelected":false})).unwrap();
        let kept=path(&s,id).subpaths[0].segment(0);let expected=original.subsegment(0.0..0.5);
        for i in 0..=20 {let t=i as f64/20.;assert!(kept.eval(t).distance(expected.eval(t))<1e-3);}
        let mut s=session();let base=line(&mut s,[0.,0.],[300.,0.]);
        line(&mut s,[100.,-50.],[100.,50.]);line(&mut s,[200.,-50.],[200.,50.]);
        s.execute("path.leoTrimJoin",&json!({"points":[[150,-20],[150,20]],"join":false,"preferSelected":false})).unwrap();
        let p=path(&s,base);assert_eq!(p.subpaths.len(),2);
        assert!(p.subpaths[0].anchors.last().unwrap().p.distance(Point::new(100.,0.))<1e-3);
        assert!(p.subpaths[1].anchors.first().unwrap().p.distance(Point::new(200.,0.))<1e-3);
    }
    #[test]
    fn ambiguous_junction_does_not_join_and_invalid_input_is_atomic() {
        let mut s=session();let(a,b)=cross(&mut s);line(&mut s,[0.,0.],[-100.,0.]);
        let r=trim(&mut s,true);assert_eq!(r["joined"],0);assert_eq!(r["trimmed"],2);
        let before=s.doc().unwrap().doc.clone();let n=s.doc().unwrap().history.undo.len();
        assert!(s.execute("path.leoTrimJoin",&json!({"points":[[0,0],["bad",2]]})).is_err());
        assert_eq!(s.doc().unwrap().doc,before);assert_eq!(s.doc().unwrap().history.undo.len(),n);
        assert!(s.doc().unwrap().doc.node(a).is_some() && s.doc().unwrap().doc.node(b).is_some());
    }
    #[test]
    fn closed_and_locked_paths_are_preserved_and_no_hit_has_no_history() {
        let mut s=session();let (a,b)=cross(&mut s);
        s.execute("object.lock",&json!({"ids":[b.0]})).unwrap();
        let before_a=path(&s,a);let before_b=path(&s,b);let n=s.doc().unwrap().history.undo.len();
        assert_eq!(trim(&mut s,false)["trimmed"],0);
        assert_eq!(path(&s,a),before_a);assert_eq!(path(&s,b),before_b);
        assert_eq!(s.doc().unwrap().history.undo.len(),n);
        let mut s=session();
        let closed=NodeId(s.execute("path.create",&json!({"anchors":[
            {"x":100,"y":100},{"x":300,"y":100},{"x":300,"y":300},{"x":100,"y":300}],"closed":true})).unwrap()["id"].as_u64().unwrap());
        line(&mut s,[200.,0.],[200.,400.]);
        s.execute("select.set",&json!({"ids":[closed.0]})).unwrap();
        let before=path(&s,closed);let n=s.doc().unwrap().history.undo.len();
        let r=s.execute("path.leoTrimJoin",&json!({"points":[[50,150],[350,150]],"preferSelected":true})).unwrap();
        assert_eq!(r["trimmed"],0);assert_eq!(path(&s,closed),before);assert_eq!(s.doc().unwrap().history.undo.len(),n);
    }
    #[test]
    fn segment_budget_refuses_without_editing() {
        let mut s=session();
        let anchors:Vec<_>=(0..514).map(|x|json!({"x":x,"y":0})).collect();
        s.execute("path.create",&json!({"anchors":anchors,"closed":false})).unwrap();
        let before=s.doc().unwrap().doc.clone();let n=s.doc().unwrap().history.undo.len();
        assert!(s.execute("path.leoTrimJoin",&json!({"points":[[50,-10],[50,10]]})).is_err());
        assert_eq!(s.doc().unwrap().doc,before);assert_eq!(s.doc().unwrap().history.undo.len(),n);
    }
    #[test]
    fn native_trim_tool_drag_cancel_then_commit_and_force_no_join() {
        let mut s=session();let(a,b)=cross(&mut s);s.execute("select.set",&json!({"ids":[]})).unwrap();
        let view=vectorcraft_tools::ViewInfo{zoom:1.,..Default::default()};
        s.select_tool("leoTrimJoin",view).unwrap();let before=s.doc().unwrap().doc.clone();
        let history=s.doc().unwrap().history.undo.len();
        for kind in [PointerKind::Down,PointerKind::Drag] {
            let y=if kind==PointerKind::Down {-120.} else {120.};
            s.pointer(&PointerEvent::new(kind,50.,y),view).unwrap();
        }
        s.tool_key(ToolKey::Escape,vectorcraft_tools::Mods::default(),view).unwrap();
        assert_eq!(s.doc().unwrap().doc,before);assert_eq!(s.doc().unwrap().history.undo.len(),history);
        s.pointer(&PointerEvent::new(PointerKind::Down,50.,-120.),view).unwrap();
        s.pointer(&PointerEvent::new(PointerKind::Drag,50.,120.),view).unwrap();
        s.tool_key(ToolKey::Letter('n'),vectorcraft_tools::Mods::default(),view).unwrap();
        s.pointer(&PointerEvent::new(PointerKind::Up,50.,120.),view).unwrap();
        assert!(s.doc().unwrap().doc.node(a).is_some() && s.doc().unwrap().doc.node(b).is_some());
        assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
    }
}


#[test]
fn leo_path_tools_rotate_scale_equal_symmetric_are_atomic_and_undoable() {
    let mut s=session();
    let id=NodeId(s.execute("path.create",&json!({"anchors":[
        {"x":100,"y":300,"out":[100,250]},
        {"x":200,"y":300,"in":[200,280]}
    ],"closed":false})).unwrap()["id"].as_u64().unwrap());
    let read=|s:&Session|s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone();
    let before=read(&s);
    let handles=json!([
        {"id":id.0,"anchor":0,"which":"out","x":100,"y":250},
        {"id":id.0,"anchor":1,"which":"in","x":200,"y":280}
    ]);
    for operation in ["rotate","scale","equal","symmetric"] {
        let history=s.doc().unwrap().history.undo.len();
        let params=json!({"handles":handles,"operation":operation,"angle":90.,"scale":2.,"revision":s.doc().unwrap().revision});
        s.execute("path.leoTransformHandles",&params).unwrap();
        let after=read(&s);let a=&after.subpaths[0].anchors;
        match operation {
            "rotate"=>{assert!(a[0].h_out.distance(Point::new(150.,300.))<1e-8);assert!(a[1].h_in.distance(Point::new(220.,300.))<1e-8);},
            "scale"=>{assert_eq!(a[0].h_out,Point::new(100.,200.));assert_eq!(a[1].h_in,Point::new(200.,260.));},
            "equal"=>{assert_eq!(a[0].h_out.distance(a[0].p),50.);assert_eq!(a[1].h_in.distance(a[1].p),50.);},
            _=>{assert_eq!(a[0].h_in,Point::new(100.,350.));assert_eq!(a[1].h_out,Point::new(200.,320.));}
        }
        assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(read(&s),before);
        s.execute("edit.redo",&json!({})).unwrap();assert_eq!(read(&s),after);
        s.execute("edit.undo",&json!({})).unwrap();
    }
    let history=s.doc().unwrap().history.undo.len();
    let mut bad=handles.clone();bad[1]["anchor"]=json!(999);
    assert!(s.execute("path.leoTransformHandles",&json!({"handles":bad,"operation":"rotate","angle":45})).is_err());
    assert_eq!(read(&s),before);assert_eq!(s.doc().unwrap().history.undo.len(),history);
    assert!(s.execute("path.leoTransformHandles",&json!({"handles":handles,"operation":"scale","scale":2,"revision":s.doc().unwrap().revision+1})).is_err());
    assert_eq!(read(&s),before);
}

#[test]
fn leo_path_tools_native_extend_snaps_and_direction_lock_projects_drag() {
    use vectorcraft_tools::{PointerEvent,PointerKind};
    let mut s=session();
    let make=|s:&mut Session,anchors:Value|NodeId(s.execute("path.create",&json!({"anchors":anchors,"closed":false})).unwrap()["id"].as_u64().unwrap());
    let source=make(&mut s,json!([{"x":100,"y":100},{"x":200,"y":100}]));
    make(&mut s,json!([{"x":250,"y":102},{"x":280,"y":130}]));
    let read=|s:&Session|s.doc().unwrap().doc.node(source).unwrap().path_data().unwrap().clone();
    let before=read(&s);
    let view=crate::ViewInfo{smart_guides:false,..Default::default()};
    s.select_tool("extendPath",view).unwrap();
    let history=s.doc().unwrap().history.undo.len();
    for kind in [PointerKind::Down,PointerKind::Drag,PointerKind::Up] {
        let p=if kind==PointerKind::Down {Point::new(200.,100.)} else {Point::new(250.,101.)};
        s.pointer(&PointerEvent::new(kind,p.x,p.y),view).unwrap();
    }
    assert_eq!(read(&s).subpaths[0].anchors.last().unwrap().p,Point::new(250.,102.));
    assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
    s.execute("edit.undo",&json!({})).unwrap();assert_eq!(read(&s),before);
    s.set_tool_option("directionLocked",&json!(true));
    for kind in [PointerKind::Down,PointerKind::Drag,PointerKind::Up] {
        let p=if kind==PointerKind::Down {Point::new(200.,100.)} else {Point::new(250.,150.)};
        s.pointer(&PointerEvent::new(kind,p.x,p.y),view).unwrap();
    }
    let end=read(&s).subpaths[0].anchors.last().unwrap().p;
    assert!(end.distance(Point::new(250.,100.))<1e-6,"{end:?}");
    s.execute("edit.undo",&json!({})).unwrap();assert_eq!(read(&s),before);
}

#[test]
fn leo_path_tools_reposition_guard_refuses_excess_error_and_drag_is_undoable() {
    use vectorcraft_tools::{PointerEvent,PointerKind};
    let mut s=session();
    let id=NodeId(s.execute("path.create",&json!({"anchors":[
        {"x":0,"y":100},{"x":100,"y":100,"out":[155,100]},
        {"x":200,"y":200,"in":[200,145]}
    ],"closed":false})).unwrap()["id"].as_u64().unwrap());
    let read=|s:&Session|s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone();
    let before=read(&s);let history=s.doc().unwrap().history.undo.len();
    assert!(s.execute("addon.pathtools.slideAnchor",&json!({"id":id.0,"anchor":1,"t":0.5,"maxError":0.})).is_err());
    assert_eq!(read(&s),before);assert_eq!(s.doc().unwrap().history.undo.len(),history);
    let view=crate::ViewInfo{smart_guides:false,..Default::default()};
    s.select_tool("slideAnchor",view).unwrap();
    s.set_tool_option("guardShape",&json!(true));
    s.set_tool_option("tolerance",&json!(0.001));
    for kind in [PointerKind::Down,PointerKind::Drag,PointerKind::Up] {
        let p=if kind==PointerKind::Down {Point::new(100.,100.)} else {Point::new(170.,132.)};
        s.pointer(&PointerEvent::new(kind,p.x,p.y),view).unwrap();
    }
    let after=read(&s);let p=after.subpaths[0].anchors[1].p;
    assert!(p.distance(Point::new(100.,100.))>1e-6,"the guarded tool must still slide where feasible");
    let t=vectorcraft_pathops::addons::pathtools::slide_t_at(&before.subpaths[0],1,p).unwrap();
    let (_,error)=vectorcraft_pathops::addons::pathtools::slide_anchor(&before.subpaths[0],1,t).unwrap();
    assert!(error<=0.0010001,"{error}");
    assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
    s.execute("edit.undo",&json!({})).unwrap();assert_eq!(read(&s),before);
}

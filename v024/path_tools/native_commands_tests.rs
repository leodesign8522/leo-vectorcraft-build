
#[test]
fn leo_path_tools_native_extend_remove_reposition_are_undoable() {
    let mut s=session();
    // Two exact halves of a cubic arch, with a removable midpoint.
    let id=NodeId(s.execute("path.create",&json!({"anchors":[
        {"x":0,"y":0,"out":[0,50]},
        {"x":50,"y":75,"in":[25,75],"out":[75,75]},
        {"x":100,"y":0,"in":[100,50]}
    ],"closed":false})).unwrap()["id"].as_u64().unwrap());
    let read=|s:&Session|s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone();
    let before=read(&s);
    for (command,params) in [
        ("path.leoExtend",json!({"id":id.0,"length":20})),
        ("path.leoExtend",json!({"id":id.0,"mode":"cubic","amount":0.25})),
        ("path.leoSmartRemove",json!({"id":id.0,"anchor":1,"tolerance":1e-8})),
        ("path.leoReposition",json!({"id":id.0,"anchor":1,"t":0.7,"tolerance":1e-8})),
    ] {
        let history=s.doc().unwrap().history.undo.len();
        s.execute(command,&params).unwrap();
        let after=read(&s);
        assert_ne!(after,before);
        assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
        let anchors=&after.subpaths[0].anchors;
        match command {
            "path.leoExtend"=>{
                assert_eq!(anchors.len(),4);
                assert_eq!(anchors[0],before.subpaths[0].anchors[0]);
                assert_eq!(anchors[1],before.subpaths[0].anchors[1]);
                assert_eq!(anchors[2].h_in,before.subpaths[0].anchors[2].h_in);
            },
            "path.leoSmartRemove"=>{
                assert_eq!(anchors.len(),2);
                assert!(anchors[0].h_out.distance(Point::new(0.,100.))<1e-8);
                assert!(anchors[1].h_in.distance(Point::new(100.,100.))<1e-8);
            },
            _=>{
                assert_eq!(anchors.len(),3);
                assert!(anchors[1].p.distance(Point::new(78.4,63.))<1e-8);
            }
        }
        s.execute("edit.undo",&json!({})).unwrap(); assert_eq!(read(&s),before);
        s.execute("edit.redo",&json!({})).unwrap(); assert_eq!(read(&s),after);
        s.execute("edit.undo",&json!({})).unwrap();
    }
    let history=s.doc().unwrap().history.undo.len();
    for (command,params) in [
        ("path.leoExtend",json!({"id":id.0,"length":-1})),
        ("path.leoExtend",json!({"id":id.0,"mode":"unknown"})),
        ("path.leoSmartRemove",json!({"id":id.0,"anchor":0})),
        ("path.leoReposition",json!({"id":id.0,"anchor":1,"t":1})),
        ("path.leoSmartRemove",json!({"id":id.0,"anchor":999})),
    ] {
        assert!(s.execute(command,&params).is_err());
        assert_eq!(read(&s),before);
        assert_eq!(s.doc().unwrap().history.undo.len(),history);
    }
}


#[test]
fn leo_path_tools_batch_handles_is_atomic_and_undoable() {
    let mut s=session();
    let id=NodeId(s.execute("path.create",&json!({"anchors":[
        {"x":100,"y":100,"out":[100,150]},
        {"x":200,"y":100,"in":[200,150]}
    ],"closed":false})).unwrap()["id"].as_u64().unwrap());
    let read=|s:&Session|s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone();
    let before=read(&s);
    let history=s.doc().unwrap().history.undo.len();
    let items=json!({"handles":[
        {"id":id.0,"subpath":0,"anchor":0,"which":"out","x":120,"y":175},
        {"id":id.0,"subpath":0,"anchor":1,"which":"in","x":220,"y":175}
    ]});
    let mut invalid=items.clone();
    invalid["handles"][1]["anchor"]=json!(999);
    assert!(s.execute("path.leoSetHandles",&invalid).is_err());
    assert_eq!(read(&s),before);
    assert_eq!(s.doc().unwrap().history.undo.len(),history);
    s.execute("path.leoSetHandles",&items).unwrap();
    let after=read(&s);
    let a=&after.subpaths[0].anchors;
    assert_eq!(a[0].h_out,Point::new(120.,175.));
    assert_eq!(a[1].h_in,Point::new(220.,175.));
    assert_eq!(a[0].p,before.subpaths[0].anchors[0].p);
    assert_eq!(a[1].p,before.subpaths[0].anchors[1].p);
    assert_eq!(s.doc().unwrap().history.undo.len(),history+1);
    s.execute("edit.undo",&json!({})).unwrap();
    assert_eq!(read(&s),before);
    s.execute("edit.redo",&json!({})).unwrap();
    assert_eq!(read(&s),after);
}

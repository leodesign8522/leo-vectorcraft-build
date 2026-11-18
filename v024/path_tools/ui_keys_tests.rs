
#[test]
fn leo_path_tools_idle_letters_reach_marquee_and_batch_commands() {
    use vectorcraft_tools::{PointerEvent,PointerKind};
    use vectorcraft_geom::Point;
    let mut app=VectorcraftApp::new(vectorcraft_engine::Session::new(),Default::default());
    crate::shortcut_editor::sync(&app.ui);
    app.run("file.new",json!({"width":400,"height":400})).unwrap();
    app.run("prefs.set",json!({"key":"showHandlesMultipleAnchors","value":true})).unwrap();
    let id=app.run("path.create",json!({"anchors":[
        {"x":100,"y":300,"out":[100,250]},
        {"x":200,"y":300,"in":[200,280]}
    ],"closed":false})).unwrap()["id"].as_u64().unwrap();
    app.run("select.set",json!({"ids":[id]})).unwrap();
    app.select_tool("directSelection");
    let letter=|app:&mut VectorcraftApp,key,text:&str| {
        let mut events=press(key,Modifiers::NONE);
        events.push(egui::Event::Text(text.into()));
        frame(app,events);
        assert_eq!(app.session.tool_id(),"directSelection");
    };
    let marquee=|app:&mut VectorcraftApp| {
        letter(app,Key::H,"h");
        assert!(app.session.tool_busy(),"H must reach the idle tool before the global Hand shortcut");
        let view=app.view_info();
        for (kind,x,y) in [(PointerKind::Down,90.,240.),(PointerKind::Drag,210.,290.),(PointerKind::Up,210.,290.)] {
            app.session.pointer(&PointerEvent::new(kind,x,y),view).unwrap();
        }
        assert!(!app.session.tool_busy());
        assert_eq!(app.session.active().unwrap().selection.objects,vec![vectorcraft_doc::NodeId(id)]);
    };
    let read=|app:&VectorcraftApp|app.session.doc().unwrap().doc.node(vectorcraft_doc::NodeId(id)).unwrap().path_data().unwrap().clone();
    let before=read(&app);
    let history=app.session.doc().unwrap().history.undo.len();
    marquee(&mut app);
    letter(&mut app,Key::L,"l");
    let equal=read(&app);
    assert_eq!(equal.subpaths[0].anchors[1].h_in,Point::new(200.,250.));
    assert_eq!(app.session.doc().unwrap().history.undo.len(),history+1);
    app.run("edit.undo",json!({})).unwrap();assert_eq!(read(&app),before);
    app.run("edit.redo",json!({})).unwrap();assert_eq!(read(&app),equal);
    marquee(&mut app);
    letter(&mut app,Key::S,"s");
    let symmetric=read(&app);
    assert_eq!(symmetric.subpaths[0].anchors[0].h_in,Point::new(100.,350.));
    assert_eq!(symmetric.subpaths[0].anchors[1].h_out,Point::new(200.,350.));
    assert_eq!(app.session.doc().unwrap().history.undo.len(),history+2);
    app.run("edit.undo",json!({})).unwrap();assert_eq!(read(&app),equal);
}

#[test]
fn leo_path_tools_h_keeps_native_hand_shortcut_without_visible_handles() {
    let mut app=VectorcraftApp::new(vectorcraft_engine::Session::new(),Default::default());
    crate::shortcut_editor::sync(&app.ui);
    app.run("file.new",json!({"width":400,"height":400})).unwrap();
    app.select_tool("directSelection");
    let mut events=press(Key::H,Modifiers::NONE);
    events.push(egui::Event::Text("h".into()));
    frame(&mut app,events);
    assert_eq!(app.session.tool_id(),"hand");
}

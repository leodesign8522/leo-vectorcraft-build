
#[test]
fn leo_path_tools_handle_marquee_toggle_and_batch_operations() {
    let (mut d,id)=arch_doc();
    let NodeKind::Path{path,..}=&mut d.node_mut(id).unwrap().kind else {panic!("path")};
    path.subpaths[0].anchors[1].h_in=Point::new(200.,280.);
    let paint=paint();
    let mut selection=Selection::default(); selection.set([id]);
    let c=cx(&d,&selection,&paint);
    let mut tool=DirectSelectionTool::new(false);
    let marquee=|tool:&mut DirectSelectionTool,shift| {
        tool.key(&c,ToolKey::Letter('h'),Mods::default());
        let mods=Mods{shift,..Mods::default()};
        tool.pointer(&c,&PointerEvent::new(PointerKind::Down,90.,240.).with_mods(mods));
        tool.pointer(&c,&PointerEvent::new(PointerKind::Drag,210.,290.).with_mods(mods));
        assert!(tool.overlays(&c).iter().any(|o|matches!(o,Overlay::Marquee(_))));
        tool.pointer(&c,&PointerEvent::new(PointerKind::Up,210.,290.).with_mods(mods));
    };
    marquee(&mut tool,false);
    assert_eq!(tool.leo_handles.len(),2);
    let actions=tool.key(&c,ToolKey::Letter('l'),Mods::default());
    let [Action::Exec(cmd,p)]=actions.as_slice() else {panic!("{actions:?}")};
    assert_eq!(cmd,"path.leoSetHandles");
    assert_eq!(p["handles"].as_array().unwrap().len(),2);
    assert_eq!(p["handles"][1]["y"],json!(250.));
    marquee(&mut tool,false);
    let actions=tool.key(&c,ToolKey::Letter('s'),Mods::default());
    let [Action::Exec(_,p)]=actions.as_slice() else {panic!("{actions:?}")};
    assert_eq!(p["handles"].as_array().unwrap().len(),4);
    assert_eq!(p["handles"][0]["y"].as_f64().unwrap()+p["handles"][1]["y"].as_f64().unwrap(),600.);
    marquee(&mut tool,false);
    marquee(&mut tool,true);
    assert!(tool.leo_handles.is_empty());
    marquee(&mut tool,false);
    tool.key(&c,ToolKey::Escape,Mods::default());
    assert!(tool.leo_handles.is_empty());
}


#[test]
fn leo_path_tools_handle_hover_pick_release_and_clear() {
    let (d,arch)=arch_doc();
    let paint=paint();
    let mut whole=Selection::default();
    whole.set([arch]);
    let c=cx(&d,&whole,&paint);
    let mut t=DirectSelectionTool::new(false);
    let p=Point::new(100.0,250.0);
    let marked=|t:&DirectSelectionTool,c:&ToolContext,filled:bool|{
        t.overlays(c).iter().any(|o| matches!(o,Overlay::CurvePoint{p:q,filled:f,color,..}
            if *q==p && *f==filled && *color==if filled {[0,200,255]} else {[255,213,64]}))
    };
    t.pointer(&c,&PointerEvent::new(PointerKind::Move,p.x,p.y));
    assert!(marked(&t,&c,false));
    assert!(!marked(&t,&c,true));
    t.hover_ended(&c);
    assert!(!marked(&t,&c,false));
    assert_eq!(t.pointer(&c,&PointerEvent::new(PointerKind::Down,p.x,p.y)),vec![Action::Begin("Reshape".into())]);
    assert!(marked(&t,&c,true));
    assert_eq!(t.pointer(&c,&PointerEvent::new(PointerKind::Up,p.x,p.y)),vec![Action::Commit]);
    t.pointer(&c,&PointerEvent::new(PointerKind::Move,20.0,20.0));
    assert!(marked(&t,&c,true));
    let hidden=ToolContext{handles_multiple:false,..cx(&d,&whole,&paint)};
    assert!(!marked(&t,&hidden,true));
    t.key(&c,ToolKey::Escape,Mods::default());
    assert!(!marked(&t,&c,true));
    let mut group=DirectSelectionTool::new(true);
    group.pointer(&c,&PointerEvent::new(PointerKind::Move,p.x,p.y));
    assert!(!marked(&group,&c,false));
}

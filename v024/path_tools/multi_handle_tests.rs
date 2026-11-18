
#[test]
fn leo_path_tools_shift_pick_two_handles_and_drag_as_batch() {
    let (d,arch)=arch_doc();
    let paint=paint();
    let mut whole=Selection::default();whole.set([arch]);
    let c=cx(&d,&whole,&paint);
    let mut t=DirectSelectionTool::new(false);
    let shift=Mods{shift:true,..Mods::default()};
    let pick=|t:&mut DirectSelectionTool,x:f64,m:Mods|{
        t.pointer(&c,&PointerEvent::new(PointerKind::Down,x,250.).with_mods(m));
        t.pointer(&c,&PointerEvent::new(PointerKind::Up,x,250.).with_mods(m));
    };
    let count=|t:&DirectSelectionTool|t.overlays(&c).iter().filter(|o|matches!(o,Overlay::CurvePoint{color:[0,200,255],filled:true,..})).count();
    pick(&mut t,100.,Mods::default());assert_eq!(count(&t),1);
    pick(&mut t,200.,shift);assert_eq!(count(&t),2);
    t.pointer(&c,&PointerEvent::new(PointerKind::Down,100.,250.));
    let a=t.pointer(&c,&PointerEvent::new(PointerKind::Drag,120.,240.));
    let [Action::Preview(cmd,v)]=a.as_slice() else{panic!("{a:?}")};
    assert_eq!(cmd,"path.leoSetHandles");
    let h=v["handles"].as_array().unwrap();assert_eq!(h.len(),2);
    let dx=h[0]["x"].as_f64().unwrap()-100.;
    let dy=h[0]["y"].as_f64().unwrap()-250.;
    assert!((h[1]["x"].as_f64().unwrap()-200.-dx).abs()<1e-8);
    assert!((h[1]["y"].as_f64().unwrap()-250.-dy).abs()<1e-8);
    t.pointer(&c,&PointerEvent::new(PointerKind::Up,120.,240.));
    pick(&mut t,200.,shift);assert_eq!(count(&t),1);
    t.key(&c,ToolKey::Escape,Mods::default());assert_eq!(count(&t),0);
}

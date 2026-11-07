#[test]
fn leo_object_move_grid_and_pixel_options_do_not_disable_geometry() {
    let (mut d, id) = doc_with_rect();
    // Off-grid and fractional: native grid/pixel rounding cannot accidentally pass this test.
    d.artboards[0].rect.x1 = 503.25;
    let mut selected = Selection::default();
    selected.add(id);
    let p = paint();
    for (grid, pixel) in [(true, false), (false, true), (true, true)] {
        let c = ToolContext { snap_to_grid: grid, snap_to_pixel: pixel, ..cx(&d, &selected, &p) };
        let mut t = SelectionTool::default();
        t.pointer(&c, &ev(PointerKind::Down, 150.0, 150.0));
        let a = t.pointer(&c, &ev(PointerKind::Drag, 452.0, 150.0));
        let Some(Action::Preview(_, v)) = a.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else { panic!("{a:?}") };
        assert_eq!(v["matrix"][4], json!(303.25), "grid={grid}, pixel={pixel}: {v}");
    }
}

#[test]
fn leo_snapping_engine_resize_pointer_side_handle_reaches_off_grid_path() {
    let (mut d,id)=doc_with_rect();
    let target=d.alloc_id();
    d.insert(Some(d.layers[0].id),1,vectorcraft_doc::Node::path(target,
        vectorcraft_geom::shapes::line(Point::new(53.25,0.0),Point::new(53.25,400.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let mut selected=Selection::default();selected.add(id);let paint=paint();
    for (grid,pixel) in [(false,false),(true,false),(false,true)] {
        let cx=ToolContext {snap_to_grid:grid,snap_to_pixel:pixel,..cx(&d,&selected,&paint)};
        let mut t=SelectionTool::default();
        t.pointer(&cx,&ev(PointerKind::Down,100.0,150.0));
        let a=t.pointer(&cx,&ev(PointerKind::Drag,57.25,150.0));
        let Some(Action::Preview(_,v))=a.iter().find(|a| matches!(a,Action::Preview(n,_) if n=="object.transform")) else {panic!("{a:?}")};
        let c:Vec<f64>=v["matrix"].as_array().unwrap().iter().map(|n| n.as_f64().unwrap()).collect();
        let xf=vectorcraft_geom::Affine::new(c.try_into().unwrap());
        assert!((xf*Point::new(100.0,150.0)).distance(Point::new(53.25,150.0))<1e-7,"grid={grid},pixel={pixel}: {v}");
        assert!((xf*Point::new(200.0,150.0)).distance(Point::new(200.0,150.0))<1e-7);
    }
}

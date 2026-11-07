// Synthetic dense document: exhaust the former 20k-anchor budget before the
// two newly drawn lines. No user document data belongs in this regression.
fn leo_dense_background(d: &mut vectorcraft_doc::Document) {
    use vectorcraft_geom::shapes;
    let mut path = shapes::rectangle(vectorcraft_geom::kurbo::Rect::new(10000.0,10000.0,10010.0,10010.0));
    let subpath = path.subpaths[0].clone();
    path.subpaths = vec![subpath; 5001];
    let id = d.alloc_id();
    d.insert(Some(d.layers[0].id), 0, vectorcraft_doc::Node::path(id, path,
        vectorcraft_doc::Appearance::default_art())).unwrap();
}

#[test]
fn leo_snapping_engine_closed_polygon_original_edge_extends_and_shortens_both_ways() {
    use vectorcraft_geom::{PathData,SubPath};
    let mut d=vectorcraft_doc::Document::new(1000.0,1000.0);
    leo_dense_background(&mut d);
    let id=d.alloc_id();
    let path=PathData::single(SubPath::polyline(&[Point::new(300.0,300.0),Point::new(600.0,330.0),Point::new(600.0,500.0),Point::new(460.0,420.0)],true));
    d.insert(Some(d.layers[0].id),1,vectorcraft_doc::Node::path(id,path,vectorcraft_doc::Appearance::default_art())).unwrap();
    let selection=anchors_of(id,&[(0,3)]);
    let p=paint(); let c=cx(&d,&selection,&p);
    for (pointer,expected) in [((541.5,478.0),(540.0,480.0)),((381.5,358.0),(380.0,360.0)),((221.5,238.0),(220.0,240.0))] {
        let mut tool=DirectSelectionTool::new(false);
        let (v,labels)=drag(&mut tool,&c,(460.0,420.0),pointer,Mods::default());
        let hit=Point::new(460.0+v["dx"].as_f64().unwrap(),420.0+v["dy"].as_f64().unwrap());
        assert!(hit.distance(Point::new(expected.0,expected.1))<1e-8,"{v}");
        assert_eq!(labels,vec!["extension".to_string()]);
    }
    let mut tool=DirectSelectionTool::new(false);
    let (v,labels)=drag(&mut tool,&c,(460.0,420.0),(541.5,478.0),Mods {shift:true,..Mods::default()});
    assert!((v["dx"].as_f64().unwrap()-v["dy"].as_f64().unwrap()).abs()<1e-8,"{v}");
    assert!(!labels.iter().any(|s| s=="extension"));
    let c=ToolContext {smart_guides:false,snap_to_point:false,..c};
    let mut tool=DirectSelectionTool::new(false);
    let (v,labels)=drag(&mut tool,&c,(460.0,420.0),(541.5,478.0),Mods::default());
    assert_eq!(v,json!({"dx":81.5,"dy":58.0})); assert!(labels.is_empty());
}

#[test]
fn leo_snapping_engine_dense_document_tail_node_drag_with_and_without_shift() {
    use vectorcraft_geom::shapes;
    for shift in [false,true] {
        let (mut d, _) = doc_with_rect();
        leo_dense_background(&mut d);
        let layer = d.layers[0].id;
        let (source,target) = (d.alloc_id(),d.alloc_id());
        d.insert(Some(layer),2,vectorcraft_doc::Node::path(source,
            shapes::line(Point::new(300.0,340.0),Point::new(380.0,340.0)),
            vectorcraft_doc::Appearance::default_art())).unwrap();
        d.insert(Some(layer),3,vectorcraft_doc::Node::path(target,
            shapes::line(Point::new(390.0,300.0),Point::new(410.0,360.0)),
            vectorcraft_doc::Appearance::default_art())).unwrap();
        let selected=anchors_of(source,&[(0,1)]);
        let p=paint();
        let c=ToolContext {snap_to_grid:false,snap_to_pixel:false,..cx(&d,&selected,&p)};
        let mut tool=DirectSelectionTool::new(false);
        let (v,labels)=drag(&mut tool,&c,(380.0,340.0),(402.0,340.0),Mods {shift,..Mods::default()});
        let endpoint=Point::new(380.0+v["dx"].as_f64().unwrap(),340.0+v["dy"].as_f64().unwrap());
        let target=vectorcraft_geom::kurbo::Line::new((390.0,300.0),(410.0,360.0));
        use vectorcraft_geom::kurbo::ParamCurveNearest;
        assert!(target.nearest(endpoint,1e-8).distance_sq<1e-12,"shift={shift}: {v}");
        if shift {assert!((endpoint.y-340.0).abs()<1e-9);}
        assert_eq!(labels,vec!["path".to_string()]);
    }
}

#[test]
fn leo_snapping_engine_shift_node_drag_uses_real_path_with_grid_and_pixel_options() {
    use vectorcraft_geom::shapes;
    let (mut d, _) = doc_with_rect();
    let layer = d.layers[0].id;
    let (source, target) = (d.alloc_id(), d.alloc_id());
    d.insert(Some(layer), 1, vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(300.0, 340.0), Point::new(380.0, 340.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    d.insert(Some(layer), 2, vectorcraft_doc::Node::path(target,
        shapes::line(Point::new(390.0, 300.0), Point::new(410.0, 360.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let p = paint();
    let c = ToolContext { snap_to_grid: true, snap_to_pixel: true, ..cx(&d, &selected, &p) };
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &c, (380.0, 340.0), (402.0, 340.0), Mods { shift: true, ..Mods::default() });
    assert!((v["dx"].as_f64().unwrap() - 23.333333333333).abs() < 1e-6, "{v}");
    assert_eq!(v["dy"], json!(0.0));
    assert_eq!(labels, vec!["path".to_string()]);
}

#[test]
fn leo_snapping_engine_alt_copy_full_path_keeps_geometric_snap_with_pixel_option() {
    use vectorcraft_geom::shapes;
    let mut d = vectorcraft_doc::Document::new(500.0, 500.0);
    let layer = d.layers[0].id;
    let id = d.alloc_id();
    d.insert(Some(layer), 0, vectorcraft_doc::Node::path(id,
        shapes::line(Point::new(600.0, 200.0), Point::new(630.0, 200.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    let mut selected = Selection::default();
    selected.set([id]);
    let p = paint();
    let c = ToolContext { snap_to_pixel: true, ..cx(&d, &selected, &p) };
    for shift in [false, true] {
        let mut tool = DirectSelectionTool::new(false);
        let (v, labels) = drag(&mut tool, &c, (600.0, 200.0), (504.0, 235.0), Mods { alt: true, shift, ..Mods::default() });
        assert_eq!(v["copy"], json!(true));
        assert_eq!(v["matrix"], json!([1.0, 0.0, 0.0, 1.0, -100.0, if shift { 0.0 } else { 35.0 }]));
        assert_eq!(labels, vec!["path".to_string()]);
    }
}

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
fn leo_snapping_engine_all_polygon_corners_and_edges_have_two_yellow_rays() {
    use vectorcraft_geom::{PathData,SubPath};
    let mut d=vectorcraft_doc::Document::new(1200.0,1200.0);
    let points=[Point::new(300.0,300.0),Point::new(650.0,380.0),Point::new(580.0,650.0),Point::new(200.0,520.0)];
    let id=d.alloc_id();
    d.insert(Some(d.layers[0].id),0,vectorcraft_doc::Node::path(id,PathData::single(SubPath::polyline(&points,true)),vectorcraft_doc::Appearance::default_art())).unwrap();
    let p=paint();
    for ai in 0..4 {
        for neighbor in [(ai+3)%4,(ai+1)%4] {
            let u=(points[ai]-points[neighbor])/(points[ai]-points[neighbor]).hypot();
            for amount in [100.0,-80.0] {
                let selected=anchors_of(id,&[(0,ai)]); let c=cx(&d,&selected,&p);
                let expected=points[ai]+u*amount;
                let pointer=expected+vectorcraft_geom::Vec2::new(-u.y,u.x)*3.0;
                let mut tool=DirectSelectionTool::new(false);
                let (v,labels)=drag(&mut tool,&c,(points[ai].x,points[ai].y),(pointer.x,pointer.y),Mods::default());
                let actual=points[ai]+vectorcraft_geom::Vec2::new(v["dx"].as_f64().unwrap(),v["dy"].as_f64().unwrap());
                assert!(actual.distance(expected)<1e-7,"corner={ai},neighbor={neighbor},amount={amount}: {v}");
                assert_eq!(labels,vec!["延长线吸附".to_string()]);
            }
        }
    }
}

#[test]
fn leo_snapping_engine_other_objects_extend_both_ways_but_hidden_and_curves_do_not() {
    use vectorcraft_geom::{PathData,BezPath,shapes};
    for (hidden,curved) in [(false,false),(true,false),(false,true)] {
        let mut d=vectorcraft_doc::Document::new(1200.0,1200.0);
        let source=d.alloc_id();
        d.insert(Some(d.layers[0].id),0,vectorcraft_doc::Node::path(source,shapes::line(Point::new(600.0,600.0),Point::new(500.0,400.0)),vectorcraft_doc::Appearance::default_art())).unwrap();
        let target=d.alloc_id();
        let path=if curved {
            let mut b=BezPath::new();b.move_to((300.0,100.0));b.curve_to((400.0,120.0),(200.0,180.0),(300.0,200.0));PathData::from_bezpath(&b)
        } else {shapes::line(Point::new(300.0,100.0),Point::new(300.0,200.0))};
        let mut node=vectorcraft_doc::Node::path(target,path,vectorcraft_doc::Appearance::default_art());node.visible=!hidden;
        d.insert(Some(d.layers[0].id),1,node).unwrap();
        let selected=anchors_of(source,&[(0,1)]);let p=paint();let c=cx(&d,&selected,&p);
        for y in [50.0,350.0] {
            let mut tool=DirectSelectionTool::new(false);
            let (v,labels)=drag(&mut tool,&c,(500.0,400.0),(304.0,y),Mods::default());
            if hidden || curved {
                assert_eq!(v["dx"],json!(-196.0));assert!(labels.is_empty(),"{labels:?}");
            } else {
                assert_eq!(v["dx"],json!(-200.0));assert_eq!(labels,vec!["延长线吸附".to_string()]);
            }
        }
        if !hidden && !curved {
            let mut tool=DirectSelectionTool::new(false);
            let (v,labels)=drag(&mut tool,&c,(500.0,400.0),(304.0,150.0),Mods::default());
            assert_eq!(v["dx"],json!(-200.0));assert_eq!(labels,vec!["path".to_string()]);
        }
    }
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
        assert_eq!(labels,vec!["延长线吸附".to_string()]);
    }
    let mut tool=DirectSelectionTool::new(false);
    let (v,labels)=drag(&mut tool,&c,(460.0,420.0),(541.5,478.0),Mods {shift:true,..Mods::default()});
    assert!((v["dx"].as_f64().unwrap()-v["dy"].as_f64().unwrap()).abs()<1e-8,"{v}");
    assert!(!labels.iter().any(|s| s=="延长线吸附"));
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
        assert!(labels == vec!["path".to_string()] || labels == vec!["参考线交点".to_string()], "unexpected label: {labels:?}");
    }
}

#[test]
fn leo_snapping_engine_shift_node_drag_uses_real_path_with_pixel_option() {
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
    let c = ToolContext { snap_to_grid: false, snap_to_pixel: true, ..cx(&d, &selected, &p) };
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &c, (380.0, 340.0), (402.0, 340.0), Mods { shift: true, ..Mods::default() });
    assert!((v["dx"].as_f64().unwrap() - 23.333333333333).abs() < 1e-6, "{v}");
    assert_eq!(v["dy"], json!(0.0));
    assert!(labels == vec!["path".to_string()] || labels == vec!["参考线交点".to_string()], "unexpected label: {labels:?}");
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
        assert!(labels == vec!["path".to_string()] || labels == vec!["参考线交点".to_string()], "unexpected label: {labels:?}");
    }
}

#[test]
fn leo_snapping_engine_direct_anchor_crossing_preview_matches_two_stationary_paths() {
    use vectorcraft_geom::shapes;
    let mut d=vectorcraft_doc::Document::new(2000.0,1500.0);
    let layer=d.layers[0].id;let source=d.alloc_id();
    d.insert(Some(layer),0,vectorcraft_doc::Node::path(source,
        shapes::line(Point::new(1100.0,410.0),Point::new(700.0,550.0)),
        vectorcraft_doc::Appearance::default_art())).unwrap();
    for (a,b) in [((40.0,734.0),(740.0,34.0)),((40.0,59.0),(580.0,734.0))] {
        let id=d.alloc_id();d.insert(Some(layer),usize::MAX,vectorcraft_doc::Node::path(id,
            shapes::line(a.into(),b.into()),vectorcraft_doc::Appearance::default_art())).unwrap();
    }
    let selected=anchors_of(source,&[(0,1)]);let paint=paint();
    for zoom in [0.25,1.0,4.0,64.0] {
        let c=ToolContext {zoom,snap_perpendicular:false,snap_tangent:false,..cx(&d,&selected,&paint)};
        let mut tool=DirectSelectionTool::new(false);
        let (v,labels)=drag(&mut tool,&c,(700.0,550.0),(340.0+3.0/zoom,434.0+4.0/zoom),Mods::default());
        let q=Point::new(700.0+v["dx"].as_f64().unwrap(),550.0+v["dy"].as_f64().unwrap());
        assert!(q.distance(Point::new(340.0,434.0))*zoom<1e-6,"zoom={zoom}: {v}");
        assert_eq!(labels,["交点"]);
    }
}

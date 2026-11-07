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
    let mut tool = DirectSelectionTool::new(false);
    let (v, labels) = drag(&mut tool, &c, (600.0, 200.0), (504.0, 235.0), Mods { alt: true, ..Mods::default() });
    assert_eq!(v["copy"], json!(true));
    assert_eq!(v["matrix"], json!([1.0, 0.0, 0.0, 1.0, -100.0, 35.0]));
    assert_eq!(labels, vec!["path".to_string()]);
}

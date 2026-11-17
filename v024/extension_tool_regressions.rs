#[test]
fn leo_snapping_engine_alt_locks_only_the_shown_single_anchor_extension() {
    use vectorcraft_geom::shapes;
    let mut doc = vectorcraft_doc::Document::new(600.0, 600.0);
    let layer = doc.layers[0].id;
    let (source, target) = (doc.alloc_id(), doc.alloc_id());
    doc.insert(Some(layer), 0, vectorcraft_doc::Node::path(
        source, shapes::line(Point::new(100.0, 100.0), Point::new(100.0, 200.0)),
        vectorcraft_doc::Appearance::default_art(),
    )).unwrap();
    doc.insert(Some(layer), 1, vectorcraft_doc::Node::path(
        target, shapes::line(Point::new(80.0, 260.0), Point::new(150.0, 260.0)),
        vectorcraft_doc::Appearance::default_art(),
    )).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let paint = paint();
    let cx = crate::testutil::leo_cx(&doc, &selected, &paint);
    let mut tool = DirectSelectionTool::new(false);
    tool.pointer(&cx, &PointerEvent::new(PointerKind::Down, 100.0, 200.0));
    tool.pointer(&cx, &PointerEvent::new(PointerKind::Drag, 103.0, 225.0));
    let event = PointerEvent::new(PointerKind::Drag, 128.0, 256.0)
        .with_mods(Mods { alt: true, ..Mods::default() });
    let actions = tool.pointer(&cx, &event);
    let Some(Action::Preview(command, movement)) = actions.last() else { panic!("{actions:?}") };
    assert_eq!(command, "path.moveAnchors");
    assert_eq!(movement["dx"], json!(0.0));
    assert!((movement["dy"].as_f64().unwrap() - 60.0).abs() < 1e-8);
    assert!(tool.overlays(&cx).iter().any(|overlay| matches!(overlay,
        Overlay::Label { text, .. } if text == "延长线 · 命中边缘")));
    let event = PointerEvent::new(PointerKind::Drag, 128.0, 293.0);
    let actions = tool.pointer(&cx, &event);
    let Some(Action::Preview(_, movement)) = actions.last() else { panic!("{actions:?}") };
    assert!((movement["dx"].as_f64().unwrap() - 28.0).abs() < 1e-8);
}

#[test]
fn leo_snapping_engine_extension_aligns_to_the_far_corner_of_a_rectangle() {
    use vectorcraft_geom::shapes;
    let mut doc = vectorcraft_doc::Document::new(600.0, 600.0);
    let layer = doc.layers[0].id;
    let (source, target) = (doc.alloc_id(), doc.alloc_id());
    doc.insert(Some(layer), 0, vectorcraft_doc::Node::path(
        source, shapes::line(Point::new(380.0, 100.0), Point::new(300.0, 180.0)),
        vectorcraft_doc::Appearance::default_art(),
    )).unwrap();
    doc.insert(Some(layer), 1, vectorcraft_doc::Node::path(
        target, shapes::rectangle(vectorcraft_geom::kurbo::Rect::new(15.0, 195.0, 100.0, 250.0)),
        vectorcraft_doc::Appearance::default_art(),
    )).unwrap();
    let selected = anchors_of(source, &[(0, 1)]);
    let paint = paint();
    let cx = crate::testutil::leo_cx(&doc, &selected, &paint);
    let mut tool = DirectSelectionTool::new(false);
    tool.pointer(&cx, &PointerEvent::new(PointerKind::Down, 300.0, 180.0));
    tool.pointer(&cx, &PointerEvent::new(PointerKind::Drag, 275.0, 205.0));
    for event in [
        PointerEvent::new(PointerKind::Drag, 235.0, 247.0),
        PointerEvent::new(PointerKind::Drag, 252.0, 275.0)
            .with_mods(Mods { alt: true, ..Mods::default() }),
    ] {
        let actions = tool.pointer(&cx, &event);
        let Some(Action::Preview(command, movement)) = actions.last() else { panic!("{actions:?}") };
        assert_eq!(command, "path.moveAnchors");
        assert!((movement["dx"].as_f64().unwrap() + 70.0).abs() < 1e-8);
        assert_eq!(movement["dy"], json!(70.0));
        let overlays = tool.overlays(&cx);
        assert!(overlays.iter().any(|o| matches!(o, Overlay::Line { a, b, dashed: true, .. }
            if a.distance(Point::new(230.0, 250.0)) < 1e-8 && *b == Point::new(100.0, 250.0))));
        assert!(overlays.iter().any(|o| matches!(o, Overlay::CurvePoint { p, filled: false, .. }
            if *p == Point::new(100.0, 250.0))));
        assert!(overlays.iter().any(|o| matches!(o, Overlay::Label { text, .. }
            if text == "延长线 · 水平对齐锚点")));
    }
    let actions = tool.pointer(&cx, &PointerEvent::new(PointerKind::Drag, 252.0, 275.0));
    let Some(Action::Preview(_, movement)) = actions.last() else { panic!("{actions:?}") };
    assert_eq!(movement["dx"], json!(-48.0));
    assert_eq!(movement["dy"], json!(95.0));
}

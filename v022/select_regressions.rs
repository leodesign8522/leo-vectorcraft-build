#[test]
fn leo_object_move_grid_and_pixel_options_do_not_disable_geometry() {
    let (d, id) = doc_with_rect();
    let mut selected = Selection::default();
    selected.add(id);
    let p = paint();
    for (grid, pixel) in [(true, false), (false, true), (true, true)] {
        let c = ToolContext { snap_to_grid: grid, snap_to_pixel: pixel, ..cx(&d, &selected, &p) };
        let mut t = SelectionTool::default();
        t.pointer(&c, &ev(PointerKind::Down, 150.0, 150.0));
        let a = t.pointer(&c, &ev(PointerKind::Drag, 449.0, 150.0));
        let Some(Action::Preview(_, v)) = a.iter().find(|a| matches!(a, Action::Preview(n, _) if n == "object.transform")) else { panic!("{a:?}") };
        assert_eq!(v["matrix"][4], json!(300.0), "grid={grid}, pixel={pixel}: {v}");
    }
}

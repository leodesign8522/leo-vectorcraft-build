
/// A rectangle segment copied through the real commands becomes an editable open path,
/// including through SVG interchange, without changing the rectangle being copied.
#[test]
fn leo_segment_copy_roundtrip_internal_svg_and_undo() {
    let mut s = session();
    let id = rect(&mut s, 20.0, 30.0, 80.0, 60.0);
    let before = node(&s, id);
    s.execute("select.segments", &json!({"id": id.0, "segments": [[0, 1]]})).unwrap();
    s.execute("edit.copy", &json!({})).unwrap();
    assert_eq!(node(&s, id), before);
    assert_eq!(s.clipboard.nodes.len(), 1);
    let original_clip = s.clipboard.nodes[0].clone();
    let NodeKind::Path { path, live, .. } = &original_clip.kind else { panic!("path") };
    assert!(live.is_none());
    assert_eq!(path.anchor_count(), 2);
    assert!(!path.subpaths[0].closed);
    let svg = s.clipboard_svg().unwrap();
    let ids = ids_of(&s.execute("edit.pasteInPlace", &json!({})).unwrap());
    assert_eq!(ids.len(), 1);
    assert_eq!(node(&s, ids[0]).path_data().unwrap(), path);
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(doc(&s).node(ids[0]).is_none());
    assert_eq!(node(&s, id), before);
    s.execute("clipboard.importSvg", &json!({"svg": svg})).unwrap();
    let NodeKind::Path { path: svg_path, .. } = &s.clipboard.nodes[0].kind else { panic!("SVG path") };
    assert_eq!(svg_path.anchor_count(), 2);
    assert!(!svg_path.subpaths[0].closed);
}

#[test]
fn leo_segment_copy_multiple_objects_disjoint_edges_and_cut_undo() {
    let mut s = session();
    let a = rect(&mut s, 20.0, 30.0, 80.0, 60.0);
    let b = rect(&mut s, 200.0, 30.0, 80.0, 60.0);
    let originals = [node(&s, a), node(&s, b)];
    s.execute("select.segments", &json!({"id": a.0, "segments": [[0, 0], [0, 2]]})).unwrap();
    s.execute("select.segments", &json!({"id": b.0, "segments": [[0, 1]], "mode": "add"})).unwrap();
    s.execute("edit.cut", &json!({})).unwrap();
    assert_eq!(s.clipboard.nodes.len(), 2);
    assert_eq!(s.clipboard.nodes[0].path_data().unwrap().subpaths.len(), 2);
    assert_eq!(s.clipboard.nodes[1].path_data().unwrap().anchor_count(), 2);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(node(&s, a), originals[0]);
    assert_eq!(node(&s, b), originals[1]);
}

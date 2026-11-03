#!/usr/bin/env python3
"""LC-022: live selected object/group counts in Document Info > Objects."""
from pathlib import Path
import sys

root = Path(sys.argv[1])

def change(rel, old, new):
    p = root / rel
    text = p.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"Unexpected Document Info anchor: {rel}: {text.count(old)}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

engine = "crates/engine/src/cmd/docinfo.rs"
panel = "crates/ui-egui/src/panels/doc_info.rs"

# The engine owns the statistics; count selected nodes, without expanding groups.
# index_paths resolves the selection in one tree walk and deduplicates node IDs.
change(engine, '    let font_names: Vec<String> = fonts.iter().map(super::fonts::font_label).collect();', '''    let selected_paths = d.index_paths(st.selection.objects.iter().copied());
    let (selected_objects, selected_groups) = selected_paths.values()
        .filter_map(|path| d.node_at(path))
        .fold((0usize, 0usize), |(objects, groups), n| match &n.kind {
            NodeKind::Layer { .. } | NodeKind::Path { guide: true, .. } => (objects, groups),
            NodeKind::Group { .. } => (objects + 1, groups + 1),
            _ => (objects + 1, groups),
        });
    let font_names: Vec<String> = fonts.iter().map(super::fonts::font_label).collect();''')
change(engine, '        "pathLength": path_length,', '''        "pathLength": path_length,
        "selection": { "objects": selected_objects, "groups": selected_groups },''')
change(engine, 'objects: {paths, compoundPaths, groups, …}, pathLength:',
       'objects: {paths, compoundPaths, groups, …}, selection: {objects, groups} (current selected nodes; a group counts once in objects and once in groups, no descendant expansion; layers and guides excluded), pathLength:')

# Selection can change without changing the document revision (clicks, commands,
# Ctrl group-member selection). Include actual selected IDs, not just their count.
change(panel, '/// `document.info` and what it was fetched for: (document uid, revision, selection only).',
       '/// `document.info` cache: document uid, revision, scope, and current selected node IDs.')
change(panel, '    key: Option<(u64, u64, bool)>,',
       '    key: Option<(u64, u64, bool, Vec<vectorcraft_doc::NodeId>)>,')
change(panel, '    let key = Some((st.uid, st.revision, sel_only));',
       '    let key = Some((st.uid, st.revision, sel_only, st.selection.objects.clone()));')
change(panel, '''            let named = named_rows(s["id"].as_str().unwrap_or_default(), rows);''', '''            if s["id"] == "objects" {
                row(ui, "已选物体", i["selection"]["objects"].as_u64().unwrap_or(0).to_string());
                row(ui, "已选编组", i["selection"]["groups"].as_u64().unwrap_or(0).to_string());
            }
            let named = named_rows(s["id"].as_str().unwrap_or_default(), rows);''')

change(engine, '''    #[test]
    fn info_counts_objects_fonts_and_selection_only() {''', '''    #[test]
    fn leo_doc_info_selection_counts_current_nodes_without_expanding_groups() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 200, "height": 100})).unwrap();
        let mut rect = || s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].clone();
        let a = rect();
        let b = rect();
        let c = rect();
        let group = s.execute("object.group", &json!({"ids": [a, b]})).unwrap()["id"].clone();
        let count = |s: &mut Session, scope| s.execute("document.info", &json!({"selectionOnly": scope})).unwrap()["selection"].clone();
        for scope in [false, true] {
            s.execute("select.set", &json!({"ids": []})).unwrap();
            assert_eq!(count(&mut s, scope), json!({"objects": 0, "groups": 0}));
            s.execute("select.set", &json!({"ids": [group, c]})).unwrap();
            assert_eq!(count(&mut s, scope), json!({"objects": 2, "groups": 1}));
            s.execute("select.set", &json!({"ids": [a]})).unwrap();
            assert_eq!(count(&mut s, scope), json!({"objects": 1, "groups": 0}));
        }
        let outer = s.execute("object.group", &json!({"ids": [group, c]})).unwrap()["id"].clone();
        s.execute("select.set", &json!({"ids": [outer]})).unwrap();
        assert_eq!(count(&mut s, false), json!({"objects": 1, "groups": 1}));
        s.execute("edit.undo", &json!({})).unwrap();
        s.execute("select.set", &json!({"ids": [group, c]})).unwrap();
        assert_eq!(count(&mut s, false), json!({"objects": 2, "groups": 1}));
    }

    #[test]
    fn info_counts_objects_fonts_and_selection_only() {''')

change(panel, '''    #[test]
    fn draws_headless() {''', '''    #[test]
    fn leo_doc_info_selection_cache_tracks_ids_scope_and_document() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        let a = app.session.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].clone();
        let b = app.session.execute("shape.rectangle", &json!({"x": 20, "y": 0, "width": 10, "height": 10})).unwrap()["id"].clone();
        let group = app.session.execute("object.group", &json!({"ids": [a, b]})).unwrap()["id"].clone();
        let ctx = egui::Context::default();
        // Seed the cache, then change only selected IDs, preserving the revision.
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 1, "groups": 1}));
        let revision = app.session.active().unwrap().revision;
        app.session.active_mut().unwrap().selection.set([vectorcraft_doc::NodeId(a.as_u64().unwrap())]);
        assert_eq!(app.session.active().unwrap().revision, revision);
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 1, "groups": 0}));
        app.session.execute("select.set", &json!({"ids": [group]})).unwrap();
        assert_eq!(info(&mut app, &ctx, true).unwrap()["selection"], json!({"objects": 1, "groups": 1}));
        app.session.active_mut().unwrap().selection.clear();
        assert_eq!(info(&mut app, &ctx, true).unwrap()["selection"], json!({"objects": 0, "groups": 0}));
        app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 0, "groups": 0}));
        let text = crate::tests_labels::painted_text(&mut app, show);
        assert!(text.contains("已选物体") && text.contains("已选编组"), "{text}");
    }

    #[test]
    fn draws_headless() {''')
print("PASS LC-022 live selection statistics and selection-aware Document Info cache")

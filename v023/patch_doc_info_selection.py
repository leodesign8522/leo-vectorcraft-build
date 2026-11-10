#!/usr/bin/env python3
"""Live selection measurements and transparency counts in Document Info > Objects."""
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

change(engine, 'use vectorcraft_geom::{Affine, Rect};', '''use vectorcraft_geom::path::measured_arclen;
use vectorcraft_geom::{Affine, Line, PathSeg, Rect};''')
change(engine, 'const OBJECT_LABELS: [(&str, &str); 18] = [', 'const OBJECT_LABELS: [(&str, &str); 19] = [')
change(engine, '    ("opacityMasks", "Opacity Masks"),', '''    ("transparentObjects", "带透明度的物体"),
    ("opacityMasks", "Opacity Masks"),''')
change(engine, '''                    if let Some(at) = rows.iter().position(|(label, _)| label == "Paths") {
                        rows.insert(at + 1, ("Path Length".into(), u.format(info["pathLength"].as_f64().unwrap_or(0.0))));
                    }''', '''                    let paths_row = rows.iter().position(|(label, _)| label == "Paths");
                    if let Some(at) = paths_row {
                        rows.insert(at + 1, ("Path Length".into(), u.format(info["pathLength"].as_f64().unwrap_or(0.0))));
                    }
                    rows.insert(paths_row.map_or(0, |at| at + 2), ("已选路径长度".into(), u.format(info["selectedPathLength"].as_f64().unwrap_or(0.0))));''')
change(engine, 'fn info(s: &mut Session, p: &Value) -> Result<Value> {', '''/// Count transparency assigned to this node once, regardless of how many of its paints use it.
fn has_transparency(n: &Node) -> bool {
    if matches!(&n.kind, NodeKind::Layer { .. } | NodeKind::Path { guide: true, .. }) {
        return false;
    }
    n.opacity < 1.0 || n.extras().mask.is_some() || n.appearance.items.iter().any(|item| {
        let paint = item.paint();
        item.visible() && !paint.is_none() && (item.opacity() < 1.0 || match paint {
            Paint::Gradient(g) => g.freeform.as_ref()
                .filter(|f| g.gradient.kind == vectorcraft_color::GradientKind::Freeform && !f.points.is_empty())
                .map_or_else(
                || g.gradient.stops.iter().any(|stop| stop.opacity < 1.0),
                |f| f.points.iter().any(|point| point.opacity < 1.0),
            ),
            _ => false,
        })
    })
}

fn selected_node_length(n: &Node, partial: Option<&vectorcraft_doc::Selection>) -> f64 {
    let NodeKind::Path { path, guide: false, .. } = &n.kind else { return 0.0 };
    let segments = partial.and_then(|s| s.segments.get(&n.id)).filter(|s| !s.is_empty());
    let anchors = partial.and_then(|s| s.anchors.get(&n.id));
    if segments.is_none() && anchors.is_none() {
        let length = path.length();
        return if length.is_finite() { length } else { 0.0 };
    }
    let mut length = 0.0;
    for (sub, sp) in path.subpaths.iter().enumerate() {
        for edge in 0..sp.segment_count() {
            let end = (edge + 1) % sp.anchors.len();
            // Match Copy's direct-selection semantics when only endpoints were selected.
            let picked = segments.map_or_else(
                || anchors.is_some_and(|set| set.contains(&(sub, edge)) && set.contains(&(sub, end))),
                |set| set.contains(&(sub, edge)),
            );
            if picked {
                let curve = if sp.segment_is_line(edge) {
                    PathSeg::Line(Line::new(sp.anchors[edge].p, sp.anchors[end].p))
                } else {
                    PathSeg::Cubic(sp.segment(edge))
                };
                length += measured_arclen(&curve, 1e-6);
            }
        }
    }
    length
}

fn info(s: &mut Session, p: &Value) -> Result<Value> {''')
change(engine, '    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();', '''    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    counts.insert("transparentObjects", 0);''')
change(engine, '            if n.extras().mask.is_some() {', '''            if has_transparency(n) {
                *counts.entry("transparentObjects").or_default() += 1;
            }
            if n.extras().mask.is_some() {''')

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
    let mut selected_path_length = 0.0;
    for id in &st.selection.objects {
        let Some(path) = selected_paths.get(id) else { continue };
        let Some(root) = d.node_at(path) else { continue };
        // A selected ancestor already covers the whole descendant, so visit it just once.
        let ancestors = d.ancestry_at(path).unwrap_or_default();
        if ancestors.iter().take(ancestors.len().saturating_sub(1)).any(|a| selected_paths.contains_key(a)) {
            continue;
        }
        root.walk(&mut |n| {
            let partial = std::ptr::eq(n, root).then_some(&st.selection);
            selected_path_length += selected_node_length(n, partial);
        });
    }
    let font_names: Vec<String> = fonts.iter().map(super::fonts::font_label).collect();''')
change(engine, '        "pathLength": path_length,', '''        "pathLength": path_length,
        "selection": { "objects": selected_objects, "groups": selected_groups },''')
change(engine, '        "selection": { "objects": selected_objects, "groups": selected_groups },', '''        "selection": { "objects": selected_objects, "groups": selected_groups },
        "selectedPathLength": selected_path_length,''')
change(engine, 'objects: {paths, compoundPaths, groups, …}, pathLength:',
       'objects: {paths, compoundPaths, groups, transparentObjects, …}, selection: {objects, groups} (current selected nodes; a group counts once in objects and once in groups, no descendant expansion; layers and guides excluded), selectedPathLength: pt (current selection, including direct-selected segments), pathLength:')

# Selection can change without changing the document revision (clicks, commands,
# Ctrl group-member selection). Include actual selected IDs, not just their count.
change(panel, '/// `document.info` and what it was fetched for: (document uid, revision, selection only).',
       '/// `document.info` cache: document uid, revision, scope and object/segment selection.')
change(panel, '    key: Option<(u64, u64, bool)>,',
       '    key: Option<(u64, u64, bool, vectorcraft_doc::Selection)>,')
change(panel, '    let key = Some((st.uid, st.revision, sel_only));',
       '    let key = Some((st.uid, st.revision, sel_only, st.selection.clone()));')
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

change(engine, 'assert_eq!(rows, json!([["Paths", "1"], ["Path Length", "25.4 mm"]]));',
       'assert_eq!(rows, json!([["Paths", "1"], ["Path Length", "25.4 mm"], ["已选路径长度", "25.4 mm"], ["带透明度的物体", "0"]]));')
change(engine, '    fn leo_doc_info_selection_counts_current_nodes_without_expanding_groups() {', '''    fn leo_doc_info_measures_segments_and_counts_assigned_transparency() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 200, "height": 100})).unwrap();
        let rect = s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 20})).unwrap()["id"].as_u64().unwrap();
        let line = s.execute("shape.line", &json!({"x1": 0, "y1": 30, "x2": 50, "y2": 30})).unwrap()["id"].as_u64().unwrap();
        let st = s.active_mut().unwrap();
        let (rect, line) = (NodeId(rect), NodeId(line));
        let doc = std::sync::Arc::make_mut(&mut st.doc);
        let node = doc.node_mut(rect).unwrap();
        node.opacity = 0.5;
        node.appearance.fill_mut().unwrap().opacity = 0.5;
        doc.node_mut(line).unwrap().appearance.stroke_mut().unwrap().opacity = 0.25;
        st.selection.set([rect]);
        let v = s.execute("document.info", &json!({})).unwrap();
        assert_eq!(v["objects"]["transparentObjects"], 2);
        assert_eq!(v["selectedPathLength"].as_f64(), Some(60.0));
        s.active_mut().unwrap().selection.segments.insert(rect, [(0, 0)].into_iter().collect());
        let v = s.execute("document.info", &json!({"selectionOnly": true})).unwrap();
        assert_eq!(v["objects"]["transparentObjects"], 1);
        assert_eq!(v["selectedPathLength"].as_f64(), Some(10.0));
        s.active_mut().unwrap().selection.clear();
        let v = s.execute("document.info", &json!({})).unwrap();
        assert_eq!(v["selectedPathLength"].as_f64(), Some(0.0));
        let group = s.execute("object.group", &json!({"ids": [rect.0, line.0]})).unwrap()["id"].as_u64().unwrap();
        s.active_mut().unwrap().selection.set([NodeId(group), rect]);
        let v = s.execute("document.info", &json!({})).unwrap();
        assert_eq!(v["selectedPathLength"].as_f64(), Some(110.0));
    }

    #[test]
    fn leo_doc_info_selection_counts_current_nodes_without_expanding_groups() {''')

change(panel, '''    #[test]
    fn draws_headless() {''', '''    #[test]
    fn leo_doc_info_selection_cache_tracks_ids_scope_and_document() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        let a = app.session.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 20})).unwrap()["id"].clone();
        let b = app.session.execute("shape.rectangle", &json!({"x": 20, "y": 0, "width": 10, "height": 10})).unwrap()["id"].clone();
        let group = app.session.execute("object.group", &json!({"ids": [a, b]})).unwrap()["id"].clone();
        let ctx = egui::Context::default();
        // Seed the cache, then change only selected IDs, preserving the revision.
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 1, "groups": 1}));
        let revision = app.session.active().unwrap().revision;
        app.session.active_mut().unwrap().selection.set([vectorcraft_doc::NodeId(a.as_u64().unwrap())]);
        assert_eq!(app.session.active().unwrap().revision, revision);
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 1, "groups": 0}));
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selectedPathLength"].as_f64(), Some(60.0));
        let id = vectorcraft_doc::NodeId(a.as_u64().unwrap());
        let st = app.session.active_mut().unwrap();
        st.selection.segments.insert(id, [(0, 0)].into_iter().collect());
        assert_eq!(st.revision, revision);
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selectedPathLength"].as_f64(), Some(10.0));
        app.session.active_mut().unwrap().selection.segments.insert(id, [(0, 1)].into_iter().collect());
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selectedPathLength"].as_f64(), Some(20.0));
        app.session.execute("select.set", &json!({"ids": [group]})).unwrap();
        assert_eq!(info(&mut app, &ctx, true).unwrap()["selection"], json!({"objects": 1, "groups": 1}));
        app.session.active_mut().unwrap().selection.clear();
        assert_eq!(info(&mut app, &ctx, true).unwrap()["selection"], json!({"objects": 0, "groups": 0}));
        app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        assert_eq!(info(&mut app, &ctx, false).unwrap()["selection"], json!({"objects": 0, "groups": 0}));
        let text = crate::tests_labels::painted_text(&mut app, show);
        assert!(text.contains("已选物体") && text.contains("已选编组")
            && text.contains("已选路径长度") && text.contains("带透明度的物体"), "{text}");
    }

    #[test]
    fn draws_headless() {''')
print("PASS Document Info selected path length, transparency and selection-aware cache")

#[cfg(test)]
mod leo_v022_session_tests {
    use crate::{Session, ViewInfo};
    use serde_json::json;
    use vectorcraft_doc::NodeId;
    use vectorcraft_geom::{Rect, Vec2};
    use vectorcraft_tools::{PointerEvent, PointerKind};

    fn bounds(s: &Session, id: NodeId) -> Rect {
        s.doc().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap()
    }

    #[test]
    fn leo_v022_group_pointer_preview_commit_undo_redo_preserves_children() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 500, "height": 500})).unwrap();
        let mut ids = vec![];
        for (x, w) in [(100, 50), (160, 40)] {
            let v = s.execute("shape.rectangle", &json!({"x":x,"y":100,"width":w,"height":100})).unwrap();
            ids.push(NodeId(v["id"].as_u64().unwrap()));
        }
        s.execute("select.set", &json!({"ids": ids.iter().map(|id| id.0).collect::<Vec<_>>()})).unwrap();
        let group = NodeId(s.execute("object.group", &json!({})).unwrap()["id"].as_u64().unwrap());
        let before: Vec<_> = ids.iter().map(|id| bounds(&s, *id)).collect();
        let view = ViewInfo::default();
        s.select_tool("selection", view).unwrap();
        let undo = s.doc().unwrap().history.undo.len();
        s.pointer(&PointerEvent::new(PointerKind::Down, 125.0, 125.0), view).unwrap();
        // Several previews exercise the session's rollback/replay, not just tool output.
        for x in [422.0, 424.0, 426.0] {
            s.pointer(&PointerEvent::new(PointerKind::Drag, x, 125.0), view).unwrap();
            assert!((bounds(&s, group).x1 - 500.0).abs() < 1e-7);
        }
        s.pointer(&PointerEvent::new(PointerKind::Up, 426.0, 125.0), view).unwrap();
        for (id, rect) in ids.iter().zip(&before) {
            assert_eq!(bounds(&s, *id), *rect + Vec2::new(300.0, 0.0));
        }
        assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
        s.execute("edit.undo", &json!({})).unwrap();
        for (id, rect) in ids.iter().zip(&before) { assert_eq!(bounds(&s, *id), *rect); }
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(bounds(&s, group).x1, 500.0);
    }
}

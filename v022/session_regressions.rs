#[cfg(test)]
mod leo_v022_session_tests {
    use crate::{Session, ViewInfo};
    use serde_json::json;
    use vectorcraft_doc::NodeId;
    use vectorcraft_geom::{Point, Rect, Vec2};
    use vectorcraft_tools::{Mods, PointerEvent, PointerKind};

    fn bounds(s: &Session, id: NodeId) -> Rect {
        s.doc().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap()
    }

    fn endpoint(s: &Session, id: NodeId, index: usize) -> Point {
        s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().subpaths[0].anchors[index].p
    }

    #[test]
    fn leo_v022_r_rotation_fixed_pivot_commit_undo_redo_and_alt_copy() {
        let mut s=Session::new();s.execute("file.new",&json!({"width":1200,"height":1200})).unwrap();
        let source=NodeId(s.execute("path.create",&json!({"d":"M200 600L600 600L600 700L200 700Z"})).unwrap()["id"].as_u64().unwrap());
        let target=NodeId(s.execute("shape.line",&json!({"x1":600,"y1":600,"x2":200,"y2":400})).unwrap()["id"].as_u64().unwrap());
        let original=s.doc().unwrap().doc.node(source).unwrap().clone();let origin=Point::new(600.0,600.0);
        let want=0.5f64.atan();let raw=want+0.01;let p=origin+(vectorcraft_geom::Affine::rotate(raw)*Point::new(-400.0,0.0)).to_vec2();
        let expected=origin+(vectorcraft_geom::Affine::rotate(want)*Point::new(-400.0,0.0)).to_vec2();let view=ViewInfo::default();
        for alt in [false,true] {
            s.execute("select.set",&json!({"ids":[source.0]})).unwrap();s.select_tool("rotate",view).unwrap();
            for kind in [PointerKind::Down,PointerKind::Up] {s.pointer(&PointerEvent::new(kind,origin.x,origin.y),view).unwrap();}
            let mods=Mods {alt,..Mods::default()};
            for (kind,p) in [(PointerKind::Down,Point::new(200.0,600.0)),(PointerKind::Drag,p),(PointerKind::Up,p)] {s.pointer(&PointerEvent::new(kind,p.x,p.y).with_mods(mods),view).unwrap();}
            let moved=s.doc().unwrap().selection.objects[0];
            assert_eq!(moved!=source,alt);assert!(endpoint(&s,moved,0).distance(expected)<1e-7);assert!(endpoint(&s,moved,1).distance(origin)<1e-7);
            if alt {assert_eq!(s.doc().unwrap().doc.node(source).unwrap(),&original);}
            assert_eq!(endpoint(&s,target,0),origin);
            s.execute("edit.undo",&json!({})).unwrap();assert_eq!(s.doc().unwrap().doc.node(source).unwrap(),&original);
            s.execute("edit.redo",&json!({})).unwrap();assert!(endpoint(&s,moved,0).distance(expected)<1e-7);
            s.execute("edit.undo",&json!({})).unwrap();
        }
    }

    #[test]
    fn leo_v022_resize_side_handle_preview_commit_undo_redo_keeps_opposite_edge() {
        let mut s=Session::new();s.execute("file.new",&json!({"width":1400,"height":1200})).unwrap();
        let source=NodeId(s.execute("shape.rectangle",&json!({"x":400,"y":300,"width":300,"height":200})).unwrap()["id"].as_u64().unwrap());
        let target=NodeId(s.execute("shape.line",&json!({"x1":203.25,"y1":100,"x2":203.25,"y2":800})).unwrap()["id"].as_u64().unwrap());
        s.execute("select.set",&json!({"ids":[source.0]})).unwrap();
        let before=bounds(&s,source);let fixed=bounds(&s,target);
        let view=ViewInfo {snap_to_grid:false,..ViewInfo::default()};s.select_tool("selection",view).unwrap();
        s.pointer(&PointerEvent::new(PointerKind::Down,400.0,400.0),view).unwrap();
        for x in [207.25,213.25] {
            s.pointer(&PointerEvent::new(PointerKind::Drag,x,400.0),view).unwrap();
            let r=bounds(&s,source);assert!((r.x0-203.25).abs()<1e-7);assert_eq!(r.x1,700.0);assert_eq!((r.y0,r.y1),(300.0,500.0));
        }
        s.pointer(&PointerEvent::new(PointerKind::Up,213.25,400.0),view).unwrap();
        let after=bounds(&s,source);assert_eq!(bounds(&s,target),fixed);
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(bounds(&s,source),before);
        s.execute("edit.redo",&json!({})).unwrap();assert_eq!(bounds(&s,source),after);
    }

    #[test]
    fn leo_v022_other_objects_extensions_commit_undo_and_keep_target_unchanged() {
        let mut s=Session::new();s.execute("file.new",&json!({"width":1200,"height":1200})).unwrap();
        let source=NodeId(s.execute("shape.line",&json!({"x1":600,"y1":600,"x2":500,"y2":400})).unwrap()["id"].as_u64().unwrap());
        let target=NodeId(s.execute("shape.line",&json!({"x1":300,"y1":100,"x2":300,"y2":200})).unwrap()["id"].as_u64().unwrap());
        let view=ViewInfo {snap_to_grid:false,..ViewInfo::default()};s.select_tool("directSelection",view).unwrap();
        for y in [50.0,350.0] {
            s.execute("select.anchors",&json!({"id":source.0,"anchors":[[0,1]],"mode":"set"})).unwrap();
            for (kind,x,y) in [(PointerKind::Down,500.0,400.0),(PointerKind::Drag,304.0,y),(PointerKind::Up,304.0,y)] {
                s.pointer(&PointerEvent::new(kind,x,y),view).unwrap();
            }
            assert_eq!(endpoint(&s,source,1),Point::new(300.0,y));
            assert_eq!(endpoint(&s,source,0),Point::new(600.0,600.0));
            assert_eq!(endpoint(&s,target,0),Point::new(300.0,100.0));
            assert_eq!(endpoint(&s,target,1),Point::new(300.0,200.0));
            s.execute("edit.undo",&json!({})).unwrap();assert_eq!(endpoint(&s,source,1),Point::new(500.0,400.0));
            s.execute("edit.redo",&json!({})).unwrap();assert_eq!(endpoint(&s,source,1),Point::new(300.0,y));
            s.execute("edit.undo",&json!({})).unwrap();
        }
    }

    #[test]
    fn leo_v022_bidirectional_original_edge_extension_commits_and_undoes() {
        let mut s=Session::new();
        s.execute("file.new",&json!({"width":1000,"height":1000})).unwrap();
        let id=NodeId(s.execute("path.create",&json!({"d":"M300 300 L600 330 L600 500 L460 420 Z"})).unwrap()["id"].as_u64().unwrap());
        let view=ViewInfo {snap_to_grid:false,..ViewInfo::default()};
        s.select_tool("directSelection",view).unwrap();
        for (pointer,expected) in [((541.5,478.0),(540.0,480.0)),((381.5,358.0),(380.0,360.0)),((221.5,238.0),(220.0,240.0))] {
            s.execute("select.anchors",&json!({"id":id.0,"anchors":[[0,3]],"mode":"set"})).unwrap();
            for (kind,p) in [(PointerKind::Down,(460.0,420.0)),(PointerKind::Drag,pointer),(PointerKind::Up,pointer)] {
                s.pointer(&PointerEvent::new(kind,p.0,p.1),view).unwrap();
            }
            assert!(endpoint(&s,id,3).distance(Point::new(expected.0,expected.1))<1e-8);
            assert_eq!(endpoint(&s,id,0),Point::new(300.0,300.0));
            s.execute("edit.undo",&json!({})).unwrap();
            assert_eq!(endpoint(&s,id,3),Point::new(460.0,420.0));
            s.execute("edit.redo",&json!({})).unwrap();
            assert!(endpoint(&s,id,3).distance(Point::new(expected.0,expected.1))<1e-8);
            s.execute("edit.undo",&json!({})).unwrap();
        }
    }

    #[test]
    fn leo_v022_dense_document_node_drag_commits_and_undoes_for_both_shift_states() {
        for shift in [false,true] {
            let mut s=Session::new();
            s.execute("file.new",&json!({"width":500,"height":500})).unwrap();
            {
                let d=std::sync::Arc::make_mut(&mut s.doc_mut().unwrap().doc);
                let mut path=vectorcraft_geom::shapes::rectangle(Rect::new(10000.0,10000.0,10010.0,10010.0));
                path.subpaths=vec![path.subpaths[0].clone();5001];
                let id=d.alloc_id();
                d.insert(Some(d.layers[0].id),0,vectorcraft_doc::Node::path(id,path,
                    vectorcraft_doc::Appearance::default_art())).unwrap();
            }
            let source=NodeId(s.execute("shape.line",&json!({"x1":300,"y1":340,"x2":380,"y2":340})).unwrap()["id"].as_u64().unwrap());
            s.execute("shape.line",&json!({"x1":390,"y1":300,"x2":410,"y2":360})).unwrap();
            let view=ViewInfo {snap_to_grid:false,snap_to_pixel:false,..ViewInfo::default()};
            s.select_tool("directSelection",view).unwrap();
            s.execute("select.anchors",&json!({"id":source.0,"anchors":[[0,1]],"mode":"set"})).unwrap();
            for (kind,x) in [(PointerKind::Down,380.0),(PointerKind::Drag,402.0),(PointerKind::Up,402.0)] {
                s.pointer(&PointerEvent::new(kind,x,340.0).with_mods(Mods {shift,..Mods::default()}),view).unwrap();
            }
            let hit=endpoint(&s,source,1);
            assert!((3.0*hit.x-hit.y-870.0).abs()<1e-6,"shift={shift}: {hit:?}");
            if shift {assert!((hit.y-340.0).abs()<1e-9);}
            assert_eq!(endpoint(&s,source,0),Point::new(300.0,340.0));
            s.execute("edit.undo",&json!({})).unwrap();
            assert_eq!(endpoint(&s,source,1),Point::new(380.0,340.0));
            s.execute("edit.redo",&json!({})).unwrap();
            assert_eq!(endpoint(&s,source,1),hit);
        }
    }

    #[test]
    fn leo_v022_shift_selected_anchor_drag_retains_live_selection_and_snaps_exactly() {
        for whole in [false, true] {
            let mut s = Session::new();
            s.execute("file.new", &json!({"width":500,"height":500})).unwrap();
            let source = NodeId(s.execute("shape.line", &json!({"x1":300,"y1":340,"x2":380,"y2":340})).unwrap()["id"].as_u64().unwrap());
            s.execute("shape.line", &json!({"x1":390,"y1":300,"x2":410,"y2":360})).unwrap();
            let view = ViewInfo { snap_to_grid: false, ..ViewInfo::default() };
            s.select_tool("directSelection", view).unwrap();
            if whole {
                s.execute("select.set", &json!({"ids":[source.0]})).unwrap();
            } else {
                s.execute("select.anchors", &json!({"id":source.0,"anchors":[[0,1]],"mode":"set"})).unwrap();
            }
            let undo = s.doc().unwrap().history.undo.len();
            let ev = |kind, x, y| PointerEvent::new(kind, x, y).with_mods(Mods { shift:true, ..Mods::default() });
            s.pointer(&ev(PointerKind::Down,380.0,340.0),view).unwrap();
            assert!(s.doc().unwrap().selection.contains(source), "Shift Down must not erase the live selection");
            for kind in [PointerKind::Drag, PointerKind::Up] {
                s.pointer(&ev(kind,402.0,340.0),view).unwrap();
            }
            assert!(endpoint(&s,source,1).distance(Point::new(403.333333333333,340.0)) < 1e-6);
            let first_x = if whole {323.333333333333} else {300.0};
            assert!(endpoint(&s,source,0).distance(Point::new(first_x,340.0)) < 1e-6);
            assert_eq!(s.doc().unwrap().history.undo.len(),undo+1);
            s.execute("edit.undo",&json!({})).unwrap();
            assert_eq!(endpoint(&s,source,0),Point::new(300.0,340.0));
            assert_eq!(endpoint(&s,source,1),Point::new(380.0,340.0));
            s.execute("edit.redo",&json!({})).unwrap();
            assert!((endpoint(&s,source,1).x-403.333333333333).abs()<1e-6);
        }
    }

    #[test]
    fn leo_v022_shift_click_without_drag_still_deselects_the_anchor_on_release() {
        let mut s = Session::new();
        s.execute("file.new",&json!({"width":500,"height":500})).unwrap();
        let id = NodeId(s.execute("shape.line",&json!({"x1":300,"y1":340,"x2":380,"y2":340})).unwrap()["id"].as_u64().unwrap());
        let view = ViewInfo::default();
        s.select_tool("directSelection",view).unwrap();
        s.execute("select.anchors",&json!({"id":id.0,"anchors":[[0,1]],"mode":"set"})).unwrap();
        let ev = |kind| PointerEvent::new(kind,380.0,340.0).with_mods(Mods {shift:true,..Mods::default()});
        s.pointer(&ev(PointerKind::Down),view).unwrap();
        assert!(s.doc().unwrap().selection.contains(id));
        s.pointer(&ev(PointerKind::Up),view).unwrap();
        assert!(!s.doc().unwrap().selection.anchors.get(&id).is_some_and(|a| a.contains(&(0,1))));
        assert_eq!(endpoint(&s,id,1),Point::new(380.0,340.0));
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

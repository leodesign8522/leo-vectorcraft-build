#[cfg(test)]
mod leo_text_modes_tests {
    use super::*;
    fn session()->Session {let mut s=Session::new();s.execute("file.new",&json!({"width":1400,"height":1000})).unwrap();s}
    fn make(s:&mut Session,area:bool,x:f64)->NodeId {
        let mut p=json!({"x":x,"y":180,"text":"B5\nEditable text","size":30});
        if area {p["area"]=json!({"width":300,"height":550});}
        NodeId(s.execute("text.create",&p).unwrap()["id"].as_u64().unwrap())
    }
    fn text(s:&Session,id:NodeId)->TextObject {let NodeKind::Text(t)=&s.doc().unwrap().doc.node(id).unwrap().kind else {panic!("text")};(**t).clone()}
    fn set_mode(s:&mut Session,id:NodeId,fit:&str,glyphs:bool) {
        s.execute("select.set",&json!({"ids":[id.0]})).unwrap();
        s.execute("text.areaOptions",&json!({"fit":fit})).unwrap();
        s.execute("type.glyphSelectionBounds",&json!({"on":glyphs})).unwrap();
    }
    #[test]
    fn leo_text_modes_new_text_defaults_on_but_legacy_objects_stay_off() {
        let mut s=session();assert!(s.prefs.auto_size_area_type);
        for area in [false,true] {
            let id=make(&mut s,area,100.0);let t=text(&s,id);assert!(t.leo_glyph_bounds);
            if area {assert_eq!(t.area.fit,vectorcraft_doc::AreaFit::AutoHeight);assert!(t.bounds().unwrap().height()<550.0);}
            let mut old=serde_json::to_value(&t).unwrap();old.as_object_mut().unwrap().remove("leoGlyphBounds");
            let old:TextObject=serde_json::from_value(old).unwrap();assert!(!old.leo_glyph_bounds);
        }
        s.prefs.auto_size_area_type=false;let fixed=make(&mut s,true,600.0);
        assert_eq!(text(&s,fixed).area.fit,vectorcraft_doc::AreaFit::None,"later user preference stays respected");
    }
    #[test]
    fn leo_text_modes_eyedropper_copies_both_directions_and_undo() {
        for (fit,glyphs) in [("autoHeight",true),("none",false)] {
            let mut s=session();let source=make(&mut s,true,100.0);set_mode(&mut s,source,fit,glyphs);
            let target=make(&mut s,true,600.0);set_mode(&mut s,target,if fit=="none" {"autoHeight"}else{"none"},!glyphs);
            if fit=="autoHeight" {s.execute("text.areaOptions",&json!({"height":550})).unwrap();}
            let original=text(&s,target);let source_before=text(&s,source);
            let frame=original.area_frame().unwrap().bounds().unwrap();
            s.execute("appearance.copyFrom",&json!({"source":source.0,"pickUp":{"appearance":false,"character":false,"paragraph":false}})).unwrap();
            let after=text(&s,target);assert_eq!(after.leo_glyph_bounds,glyphs);assert_eq!(after.area.fit,source_before.area.fit);
            assert_eq!(after.runs,original.runs);assert_eq!(after.xf,original.xf);assert_eq!(text(&s,source),source_before);
            let now=after.area_frame().unwrap().bounds().unwrap();assert_eq!(now.width(),frame.width());assert_eq!(now.y0,frame.y0);
            if fit=="none" {assert_eq!(now,frame);}
            else {assert!(now.height()<frame.height(),"auto height must fit the target contents");}
            s.execute("edit.undo",&json!({})).unwrap();assert_eq!(text(&s,target),original);
            s.execute("edit.redo",&json!({})).unwrap();assert_eq!(text(&s,target),after);
        }
    }
    #[test]
    fn leo_text_modes_options_can_disable_sampling_and_nontext_does_not_reset() {
        let mut s=session();let source=make(&mut s,true,100.0);set_mode(&mut s,source,"none",false);
        let target=make(&mut s,true,600.0);let original=text(&s,target);
        s.execute("appearance.copyFrom",&json!({"source":source.0,"pickUp":{"appearance":false,"character":false,"paragraph":false,"textFrameMode":false,"glyphBounds":false}})).unwrap();
        assert_eq!(text(&s,target),original);
        let shape=NodeId(s.execute("shape.rectangle",&json!({"x":800,"y":300,"width":50,"height":50})).unwrap()["id"].as_u64().unwrap());
        s.execute("select.set",&json!({"ids":[target.0]})).unwrap();
        s.execute("appearance.copyFrom",&json!({"source":shape.0,"pickUp":{"appearance":false,"character":false,"paragraph":false}})).unwrap();
        assert_eq!(text(&s,target),original);
    }
    #[test]
    fn leo_text_modes_point_source_only_copies_applicable_flag() {
        let mut s=session();let source=make(&mut s,false,100.0);
        s.execute("type.glyphSelectionBounds",&json!({"on":false})).unwrap();
        let target=make(&mut s,true,600.0);let before=text(&s,target);
        s.execute("appearance.copyFrom",&json!({"source":source.0,"pickUp":{"appearance":false,"character":false,"paragraph":false}})).unwrap();
        let after=text(&s,target);assert!(!after.leo_glyph_bounds);assert_eq!(after.area,before.area);assert_eq!(after.kind,before.kind);
    }

    #[test]
    fn leo_text_modes_grid_node_mid_drag_releases_geometry_and_restores_after_grid_off() {
        use crate::ViewInfo;
        use vectorcraft_geom::Point;
        use vectorcraft_tools::{Mods,Overlay,PointerEvent,PointerKind};
        let mut s=session();s.edit("grid fixture",|d,_|{d.grid.spacing=20.0;d.grid.subdivisions=1;Ok(())}).unwrap();
        let source=NodeId(s.execute("shape.line",&json!({"x1":300,"y1":340,"x2":380,"y2":340})).unwrap()["id"].as_u64().unwrap());
        s.execute("shape.line",&json!({"x1":390,"y1":300,"x2":410,"y2":360})).unwrap();
        s.execute("select.anchors",&json!({"id":source.0,"anchors":[[0,1]],"mode":"set"})).unwrap();
        let view=ViewInfo::default();s.select_tool("directSelection",view).unwrap();
        let ev=|kind|PointerEvent::new(kind,402.0,340.0).with_mods(Mods {shift:true,..Mods::default()});
        s.pointer(&PointerEvent::new(PointerKind::Down,380.0,340.0).with_mods(Mods {shift:true,..Mods::default()}),view).unwrap();
        let endpoint=|s:&Session|s.doc().unwrap().doc.node(source).unwrap().path_data().unwrap().subpaths[0].anchors[1].p;
        s.pointer(&ev(PointerKind::Drag),view).unwrap();assert!(endpoint(&s).distance(Point::new(403.333333333333,340.0))<1e-6);
        let grid=ViewInfo {snap_to_grid:true,snap_to_pixel:true,..view};
        s.pointer(&ev(PointerKind::Drag),grid).unwrap();assert_eq!(endpoint(&s),Point::new(400.0,340.0));
        assert!(!s.overlays(grid).iter().any(|o|matches!(o,Overlay::Label {..})),"no stale smart hit guide");
        s.pointer(&ev(PointerKind::Drag),view).unwrap();assert!(endpoint(&s).distance(Point::new(403.333333333333,340.0))<1e-6);
        s.pointer(&ev(PointerKind::Up),view).unwrap();s.execute("edit.undo",&json!({})).unwrap();assert_eq!(endpoint(&s),Point::new(380.0,340.0));
    }

    #[test]
    fn leo_text_modes_grid_object_resize_and_rotation_ignore_smart_guides() {
        use crate::ViewInfo;
        use vectorcraft_geom::Point;
        use vectorcraft_tools::{Overlay,PointerEvent,PointerKind};
        for tool in ["move","resize","rotate"] {
            let mut result=None;
            for smart in [false,true] {
                let mut s=session();s.edit("grid fixture",|d,_|{d.grid.spacing=20.0;d.grid.subdivisions=1;Ok(())}).unwrap();
                let id=NodeId(s.execute("shape.rectangle",&json!({"x":400,"y":300,"width":300,"height":200})).unwrap()["id"].as_u64().unwrap());
                s.execute("shape.line",&json!({"x1":203.25,"y1":100,"x2":203.25,"y2":800})).unwrap();
                s.execute("shape.line",&json!({"x1":700,"y1":300,"x2":300,"y2":100})).unwrap();
                s.execute("select.set",&json!({"ids":[id.0]})).unwrap();
                let view=ViewInfo {smart_guides:smart,snap_to_point:true,snap_to_grid:true,snap_to_pixel:true,..ViewInfo::default()};
                s.select_tool(if tool=="rotate" {"rotate"}else{"selection"},view).unwrap();
                if tool=="rotate" {for kind in [PointerKind::Down,PointerKind::Up] {s.pointer(&PointerEvent::new(kind,700.0,300.0),view).unwrap();}}
                let (from,to)=match tool {"move"=>(Point::new(550.0,400.0),Point::new(354.0,400.0)),"resize"=>(Point::new(400.0,400.0),Point::new(207.25,400.0)),_=>(Point::new(400.0,300.0),Point::new(427.0,184.0))};
                for (kind,p) in [(PointerKind::Down,from),(PointerKind::Drag,to)] {s.pointer(&PointerEvent::new(kind,p.x,p.y),view).unwrap();}
                assert!(!s.overlays(view).iter().any(|o|matches!(o,Overlay::Label {..})),"{tool}: smart guide leaked into grid mode");
                s.pointer(&PointerEvent::new(PointerKind::Up,to.x,to.y),view).unwrap();
                let n=s.doc().unwrap().doc.node(id).unwrap().clone();
                if tool!="rotate" {assert!((n.geometric_bounds().unwrap().x0-200.0).abs()<1e-7,"{tool}");}
                if let Some(before)=&result {assert_eq!(&n,before,"{tool}: smart setting altered grid result");}else{result=Some(n);}
            }
        }
    }
}


#[cfg(test)]
mod leo_glyph_bounds_tests {
    use super::*;
    use vectorcraft_geom::{Point, Rect, Shape};
    use crate::ViewInfo;
    use vectorcraft_tools::{PointerEvent, PointerKind};

    fn fixture(text: &str, area: bool) -> (Session, NodeId) {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width":1200,"height":1000})).unwrap();
        let mut p = json!({"x":150,"y":180,"text":text,"size":80,"font":"Arial"});
        if area { p["area"] = json!({"width":560,"height":650}); }
        let id = NodeId(s.execute("text.create", &p).unwrap()["id"].as_u64().unwrap());
        s.execute("select.set", &json!({"ids":[id.0]})).unwrap();
        (s,id)
    }
    fn text(s: &Session, id: NodeId) -> &TextObject {
        let NodeKind::Text(t) = &s.doc().unwrap().doc.node(id).unwrap().kind else { panic!("text") };
        t
    }
    fn close(a: Rect, b: Rect) {
        for (a,b) in [a.x0,a.y0,a.x1,a.y1].into_iter().zip([b.x0,b.y0,b.x1,b.y1]) {
            assert!((a-b).abs()<1e-6, "{a} != {b}");
        }
    }
    fn toggle(s: &mut Session, on: bool) {
        s.execute("type.glyphSelectionBounds", &json!({"on":on})).unwrap();
    }

    #[test]
    fn leo_glyph_bounds_point_is_ink_and_undo_restores_metrics() {
        let (mut s,id)=fixture("a",false);
        let before=text(&s,id).clone();
        let metrics=s.transform_box(&[id]).unwrap();
        toggle(&mut s,true);
        let t=text(&s,id);
        let lay=vectorcraft_text::layout(vectorcraft_text::FontDb::global(),t);
        let ink=t.xf.transform_rect_bbox(lay.ink_bounds().unwrap());
        close(s.transform_bounds(&[id]).unwrap(),ink);
        assert!(ink.height()<metrics.rect.height());
        let mut expected=before.clone();expected.leo_glyph_bounds=true;
        assert_eq!(t,&expected,"only the selection preference changes");
        assert_eq!(t.bounds(),before.bounds(),"rendering geometry stays unchanged");
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(text(&s,id),&before);
        s.execute("edit.redo",&json!({})).unwrap();assert_eq!(text(&s,id),&expected);
        toggle(&mut s,false);assert_eq!(s.transform_box(&[id]),Some(metrics));
    }

    #[test]
    fn leo_glyph_bounds_multiline_preserves_frame_glyphs_and_breaks() {
        for content in ["adfadfadfadfasdf\nasdfasdfasdfa\ndfadfasdf\ndfadfa", "中文汉字\n字体贴合\nAgjp"] {
            let (mut s,id)=fixture(content,true);
            let before=text(&s,id).clone();
            let lay=vectorcraft_text::layout(vectorcraft_text::FontDb::global(),&before);
            toggle(&mut s,true);
            let after=text(&s,id);
            assert_eq!(after.kind,before.kind);assert_eq!(after.runs,before.runs);assert_eq!(after.xf,before.xf);
            let now=vectorcraft_text::layout(vectorcraft_text::FontDb::global(),after);
            assert_eq!(lay.to_bezpath(),now.to_bezpath());
            assert_eq!(lay.lines.len(),now.lines.len());
            close(s.transform_bounds(&[id]).unwrap(),after.xf.transform_rect_bbox(lay.ink_bounds().unwrap()));
        }
    }

    #[test]
    fn leo_glyph_bounds_rotation_uses_local_glyph_outline() {
        let (mut s,id)=fixture("Agjp",false);toggle(&mut s,true);
        s.execute("object.transform",&json!({"matrix":[0.8,0.6,-0.6,0.8,50,20]})).unwrap();
        let t=text(&s,id);let b=s.transform_box(&[id]).unwrap();
        let mut path=vectorcraft_text::layout(vectorcraft_text::FontDb::global(),t).to_bezpath();
        path.apply_affine(b.to_doc().inverse()*t.xf);
        close(b.rect,path.bounding_box());
    }

    #[test]
    fn leo_glyph_bounds_serialization_and_old_documents() {
        let (mut s,id)=fixture("可编辑 a",true);let old=serde_json::to_value(text(&s,id)).unwrap();
        assert!(old.get("leoGlyphBounds").is_none());
        let legacy:TextObject=serde_json::from_value(old).unwrap();assert!(!legacy.leo_glyph_bounds);
        toggle(&mut s,true);
        let value=serde_json::to_value(text(&s,id)).unwrap();assert_eq!(value["leoGlyphBounds"],true);
        let reopened:TextObject=serde_json::from_value(value).unwrap();
        assert!(reopened.leo_glyph_bounds);assert_eq!(reopened.kind,text(&s,id).kind);
        assert_eq!(reopened.runs,text(&s,id).runs);
    }

    #[test]
    fn leo_glyph_bounds_empty_and_whitespace_keep_usable_frame() {
        for content in ["", "   \n  "] {
            let (mut s,id)=fixture(content,true);let before=s.transform_box(&[id]);
            toggle(&mut s,true);assert_eq!(s.transform_box(&[id]),before);
        }
    }

    #[test]
    fn leo_glyph_bounds_editing_updates_controls_and_numeric_scale_keeps_frame() {
        let (mut s,id)=fixture("a",true);toggle(&mut s,true);
        let before=s.transform_box(&[id]).unwrap();
        s.execute("text.setText",&json!({"id":id.0,"text":"MMMMMM\nAgjp"})).unwrap();
        let t=text(&s,id).clone();let bx=s.transform_box(&[id]).unwrap();
        assert!(bx.rect.width()>before.rect.width());
        let lay=vectorcraft_text::layout(vectorcraft_text::FontDb::global(),&t);
        close(s.transform_bounds(&[id]).unwrap(),t.xf.transform_rect_bbox(lay.ink_bounds().unwrap()));
        s.execute("object.setBounds",&json!({"width":bx.rect.width()*1.5,"height":bx.rect.height()*1.5,"reference":0})).unwrap();
        assert_eq!(text(&s,id).kind,t.kind);assert_eq!(text(&s,id).runs,t.runs);
        let after=s.transform_box(&[id]).unwrap();
        assert!((after.rect.width()/bx.rect.width()-1.5).abs()<1e-6);
        assert!((after.rect.height()/bx.rect.height()-1.5).abs()<1e-6);
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(text(&s,id),&t);
    }

    #[test]
    fn leo_glyph_bounds_selection_handle_scales_area_without_reflow() {
        let (mut s,id)=fixture("First line\nSecond line",true);toggle(&mut s,true);
        let before=text(&s,id).clone();let bx=s.transform_box(&[id]).unwrap();let r=bx.rect;
        let p=bx.to_doc()*Point::new(r.x1,r.y1);let q=bx.to_doc()*Point::new(r.x1+r.width(),r.y1+r.height());
        let view=ViewInfo::default();s.select_tool("selection",view).unwrap();
        for (kind,p) in [(PointerKind::Down,p),(PointerKind::Drag,q),(PointerKind::Up,q)] {
            s.pointer(&PointerEvent::new(kind,p.x,p.y),view).unwrap();
        }
        let after=text(&s,id);
        assert_eq!(after.kind,before.kind,"the area frame is not resized or reflowed");
        assert_eq!(after.runs,before.runs);assert!(after.leo_glyph_bounds);
        assert!((s.transform_box(&[id]).unwrap().rect.width()/r.width()-2.0).abs()<1e-6);
        s.execute("edit.undo",&json!({})).unwrap();assert_eq!(text(&s,id),&before);
    }
}

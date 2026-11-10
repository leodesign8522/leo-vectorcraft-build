#[cfg(test)]
mod leo_whole_selection_tests {
    use super::*;
    use vectorcraft_tools::{Mods, PointerEvent, PointerKind};
    fn fixture(nested: bool) -> (Session, NodeId, NodeId, NodeId) {
        let mut s=session();
        let circle=id_of(&s.execute("shape.ellipse",&json!({"x":100,"y":100,"width":200,"height":200})).unwrap());
        let inner=id_of(&s.execute("shape.rectangle",&json!({"x":160,"y":160,"width":40,"height":40})).unwrap());
        s.execute("select.set",&json!({"ids":[circle.0,inner.0]})).unwrap();
        let mut group=id_of(&s.execute("object.group",&json!({})).unwrap());
        if nested {
            let other=id_of(&s.execute("shape.rectangle",&json!({"x":350,"y":100,"width":20,"height":20})).unwrap());
            s.execute("select.set",&json!({"ids":[group.0,other.0]})).unwrap();
            group=id_of(&s.execute("object.group",&json!({})).unwrap());
        }
        let view=crate::ViewInfo {smart_guides:false,snap_to_point:false,..crate::ViewInfo::default()};
        s.select_tool("selection",view).unwrap();s.select_tool("directSelection",view).unwrap();
        (s,circle,inner,group)
    }
    fn bounds(s:&Session,id:NodeId)->Rect {s.doc().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap()}
    fn event(s:&mut Session,kind:PointerKind,x:f64,y:f64,mods:Mods) {
        s.pointer(&PointerEvent::new(kind,x,y).with_mods(mods),crate::ViewInfo {smart_guides:false,snap_to_point:false,..crate::ViewInfo::default()}).unwrap();
    }
    #[test]
    fn leo_whole_selection_group_anchor_fill_and_curve_drags_translate_every_member() {
        for nested in [false,true] {for (x,y) in [(200.,300.),(130.,190.),(129.2893218813,129.2893218813)] {
            let (mut s,circle,inner,group)=fixture(nested);
            let before=[bounds(&s,circle),bounds(&s,inner),bounds(&s,group)];
            for (kind,dx,dy) in [(PointerKind::Down,0.,0.),(PointerKind::Drag,35.,45.),(PointerKind::Up,35.,45.)] {
                event(&mut s,kind,x+dx,y+dy,Mods::default());
            }
            for (id,b) in [circle,inner,group].into_iter().zip(before) {
                assert_eq!(bounds(&s,id),b+vectorcraft_geom::Vec2::new(35.,45.),"{nested} {x} {y}");
            }
            assert_eq!(s.doc().unwrap().selection.objects,vec![group]);
            s.execute("edit.undo",&json!({})).unwrap();assert_eq!(bounds(&s,group),before[2]);
            s.execute("edit.redo",&json!({})).unwrap();assert_eq!(bounds(&s,inner),before[1]+vectorcraft_geom::Vec2::new(35.,45.));
        }}
    }
    #[test]
    fn leo_whole_selection_click_still_drills_down_and_single_anchor_can_reshape() {
        let (mut s,circle,inner,_)=fixture(true);let original=bounds(&s,circle);let inside=bounds(&s,inner);
        for kind in [PointerKind::Down,PointerKind::Up] {event(&mut s,kind,200.,300.,Mods::default());}
        assert_eq!(s.doc().unwrap().selection.objects,vec![circle]);
        assert!(s.doc().unwrap().selection.partial(circle).is_some());
        for (kind,y) in [(PointerKind::Down,300.),(PointerKind::Drag,340.),(PointerKind::Up,340.)] {event(&mut s,kind,200.,y,Mods::default());}
        assert!(bounds(&s,circle).height()>original.height()+39.);assert_eq!(bounds(&s,inner),inside);
        let (mut s,circle,_,_)=fixture(false);
        for kind in [PointerKind::Down,PointerKind::Up] {event(&mut s,kind,130.,190.,Mods::default());}
        assert_eq!(s.doc().unwrap().selection.objects,vec![circle]);
        assert!(s.doc().unwrap().selection.partial(circle).is_none());
    }
    #[test]
    fn leo_whole_selection_alt_copy_keeps_original_group_and_contents() {
        for (x,y) in [(200.,300.),(130.,190.)] {
            let (mut s,circle,inner,group)=fixture(true);let old=bounds(&s,group);let c=bounds(&s,circle);let i=bounds(&s,inner);
            let mods=Mods {alt:true,..Mods::default()};
            for (kind,dx,dy) in [(PointerKind::Down,0.,0.),(PointerKind::Drag,35.,45.),(PointerKind::Up,35.,45.)] {event(&mut s,kind,x+dx,y+dy,mods);}
            assert_eq!(bounds(&s,group),old);assert_eq!(bounds(&s,circle),c);assert_eq!(bounds(&s,inner),i);
            let copy=s.doc().unwrap().selection.objects[0];assert_ne!(copy,group);
            assert_eq!(bounds(&s,copy),old+vectorcraft_geom::Vec2::new(35.,45.));
        }
    }
}

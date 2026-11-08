#[cfg(test)]
mod leo_rotation_tool_tests {
    use super::*;
    use crate::testutil::{cx,paint};
    use vectorcraft_doc::{Document,Node,Selection,Appearance};
    use vectorcraft_geom::shapes;

    #[test]
    fn leo_snapping_engine_r_rotation_pivot_shift_and_alt_preview() {
        let mut d=Document::new(1200.0,1200.0);let id=d.alloc_id();
        d.insert(Some(d.layers[0].id),0,Node::path(id,shapes::rectangle(Rect::new(200.0,600.0,600.0,700.0)),Appearance::default_art())).unwrap();
        let target=d.alloc_id();d.insert(Some(d.layers[0].id),1,Node::path(target,shapes::line(Point::new(600.0,600.0),Point::new(200.0,400.0)),Appearance::default_art())).unwrap();
        let mut s=Selection::default();s.set([id]);let paint=paint();let cx=cx(&d,&s,&paint);let origin=Point::new(600.0,600.0);
        for mods in [Mods::default(),Mods {alt:true,..Mods::default()},Mods {shift:true,..Mods::default()}] {
            let mut t=TransformTool::new(TransformKind::Rotate);
            for kind in [PointerKind::Down,PointerKind::Up] {t.pointer(&cx,&PointerEvent::new(kind,600.0,600.0));}
            t.pointer(&cx,&PointerEvent::new(PointerKind::Down,200.0,600.0).with_mods(mods));
            let raw=0.5f64.atan()+0.01;let p=origin+(Affine::rotate(raw)*Point::new(-400.0,0.0)).to_vec2();
            let out=t.pointer(&cx,&PointerEvent::new(PointerKind::Drag,p.x,p.y).with_mods(mods));
            let Some(Action::Preview(_,v))=out.last() else {panic!("{out:?}")};
            assert_eq!(v["copy"],mods.alt);
            let c:Vec<f64>=v["matrix"].as_array().unwrap().iter().map(|n| n.as_f64().unwrap()).collect();let m=Affine::new(c.try_into().unwrap());
            let want=if mods.shift {std::f64::consts::FRAC_PI_4} else {0.5f64.atan()};
            assert!((m.as_coeffs()[0]-want.cos()).abs()<1e-8);assert!((m*origin).distance(origin)<1e-8);
            assert_eq!(t.overlays(&cx).iter().any(|o| matches!(o,Overlay::Label {text,..} if text.starts_with("旋转吸附"))),!mods.shift);
            let out=t.pointer(&cx,&PointerEvent::new(PointerKind::Up,p.x,p.y).with_mods(mods));assert!(out.iter().any(|a| matches!(a,Action::Commit)));
        }
    }
}

#[cfg(test)]
mod leo_rotation_tests {
    use super::*;
    fn scene(offset: f64) -> (Document,Selection) {
        use vectorcraft_geom::shapes;
        let mut d=Document::new(1200.0,1200.0);
        let source=d.alloc_id();d.insert(Some(d.layers[0].id),0,Node::path(source,shapes::rectangle(Rect::new(200.0,600.0,600.0,700.0)),vectorcraft_doc::Appearance::default_art())).unwrap();
        let target=d.alloc_id();d.insert(Some(d.layers[0].id),1,Node::path(target,shapes::line(Point::new(600.0,600.0+offset),Point::new(200.0,400.0+offset)),vectorcraft_doc::Appearance::default_art())).unwrap();
        let mut selected=Selection::default();selected.set([source]);(d,selected)
    }
    fn pointer(origin: Point,angle: f64) -> Point {origin+(Affine::rotate(angle)*Point::new(-400.0,0.0)).to_vec2()}

    #[test]
    fn leo_snapping_engine_rotation_custom_pivot_is_rigid_and_collinear() {
        let (d,s)=scene(0.0);let paint=crate::testutil::paint();let cx=crate::testutil::cx(&d,&s,&paint);
        let origin=Point::new(600.0,600.0);let snap=LeoRotationSnap::new(&cx,origin).unwrap();let want=0.5f64.atan();
        let raw=want+0.01;let (m,angle,g)=snap.snap(&cx,raw,pointer(origin,raw),false).unwrap();
        assert!((angle-want).abs()<1e-10);assert!((m*origin).distance(origin)<1e-8);
        let (a,b)=(m*Point::new(200.0,600.0),m*origin);
        assert!(((a-b).cross(Vec2::new(-400.0,-200.0))).abs()<1e-6);
        assert!((a.distance(b)-400.0).abs()<1e-8);
        assert!(g.iter().any(|o| matches!(o,Overlay::Label {text,..} if text=="旋转吸附 · 共线")));
    }

    #[test]
    fn leo_snapping_engine_rotation_parallel_is_labelled_without_translation() {
        let (d,s)=scene(-100.0);let paint=crate::testutil::paint();let cx=crate::testutil::cx(&d,&s,&paint);
        let origin=Point::new(600.0,600.0);let snap=LeoRotationSnap::new(&cx,origin).unwrap();let raw=0.5f64.atan()+0.01;
        let (m,_,g)=snap.snap(&cx,raw,pointer(origin,raw),false).unwrap();
        assert!((m*origin).distance(origin)<1e-8);
        assert!(g.iter().any(|o| matches!(o,Overlay::Label {text,..} if text=="旋转吸附 · 平行")));
    }

    #[test]
    fn leo_snapping_engine_rotation_pixels_release_and_shift_at_three_zooms() {
        let (d,s)=scene(0.0);let paint=crate::testutil::paint();let origin=Point::new(600.0,600.0);let want=0.5f64.atan();
        for zoom in [0.25,1.0,4.0] {
            let cx=ToolContext {zoom,..crate::testutil::cx(&d,&s,&paint)};let snap=LeoRotationSnap::new(&cx,origin).unwrap();
            let delta=|px: f64| 2.0*(px/zoom/(2.0*snap.radius)).asin();
            for (px,hit) in [(7.9,true),(12.7,true),(12.9,false),(8.1,false),(7.9,true)] {
                let raw=want+delta(px);assert_eq!(snap.snap(&cx,raw,pointer(origin,raw),false).is_some(),hit,"zoom={zoom},px={px}");
            }
            assert!(snap.snap(&cx,want,pointer(origin,want),true).is_none());assert!(snap.held.get().is_none());
        }
    }

    #[test]
    fn leo_snapping_engine_rotation_viewport_and_frozen_target() {
        let (mut d,s)=scene(0.0);let target=d.layers[0].children().unwrap()[1].id;
        d.node_mut(target).unwrap().visible=false;
        let paint=crate::testutil::paint();let origin=Point::new(600.0,600.0);
        assert!(LeoRotationSnap::new(&crate::testutil::cx(&d,&s,&paint),origin).is_none());
        d.node_mut(target).unwrap().visible=true;
        let view=crate::ScreenFrame {origin:Point::new(0.0,0.0),right:Vec2::new(1.0,0.0),down:Vec2::new(0.0,1.0),size:(1000.0,1000.0)};
        let cx=ToolContext {screen:Some(view),..crate::testutil::cx(&d,&s,&paint)};let snap=LeoRotationSnap::new(&cx,origin).unwrap();let want=0.5f64.atan();
        assert!(snap.snap(&cx,want+0.01,pointer(origin,want+0.01),false).is_some());
        let off=ToolContext {screen:Some(crate::ScreenFrame {origin:Point::new(2000.0,2000.0),..view}),..cx};
        assert!(snap.snap(&off,want+0.02,pointer(origin,want+0.02),false).is_some());
        assert!(snap.snap(&off,want+0.1,pointer(origin,want+0.1),false).is_none());
        assert!(snap.snap(&off,want,pointer(origin,want),false).is_none());
    }

    #[test]
    fn leo_snapping_engine_rotation_dense_tail_full_anchor_selection_and_angle_wrap() {
        let (mut d,mut s)=scene(0.0);
        let mut filler=vectorcraft_geom::shapes::rectangle(Rect::new(10000.0,10000.0,10010.0,10010.0));filler.subpaths=vec![filler.subpaths[0].clone();5001];
        let id=d.alloc_id();d.insert(Some(d.layers[0].id),0,Node::path(id,filler,vectorcraft_doc::Appearance::default_art())).unwrap();
        let source=s.objects[0];s.anchors.insert(source,[(0,0),(0,1),(0,2),(0,3)].into());
        let origin=Point::new(600.0,600.0);let want=(-4.0f64).to_radians();
        let target=d.layers[0].children().unwrap()[2].id;
        let end=origin+(Affine::rotate(want)*Point::new(-400.0,0.0)).to_vec2();
        *d.node_mut(target).unwrap()=Node::path(target,vectorcraft_geom::shapes::line(origin,end),vectorcraft_doc::Appearance::default_art());
        let paint=crate::testutil::paint();let cx=crate::testutil::cx(&d,&s,&paint);let snap=LeoRotationSnap::new(&cx,origin).unwrap();
        let raw=want-0.01;let (m,angle,_)=snap.snap(&cx,raw,pointer(origin,raw),false).unwrap();
        assert!((angle-want).abs()<1e-8);assert!((m*Point::new(200.0,600.0)).distance(end)<1e-7);
    }
}

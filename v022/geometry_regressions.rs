#[cfg(test)]
mod leo_v022_geometry_tests {
    use super::*;

    #[test]
    fn leo_snapping_engine_bidirectional_extension_capture_release_and_shift() {
        for zoom in [0.25,1.0,4.0] {
            let mut t=Targets::default();
            let at=Point::new(300.0,300.0);
            let dir=Vec2::new(0.8,0.6);
            let normal=Vec2::new(-0.6,0.8);
            t.extensions.push((at,dir));
            for length in [300.0,100.0,-100.0] {
                t.clear_leo_snap();
                let on=at+dir*length;
                let (q,_)=t.leo_snap_extension(on+normal*(7.9/zoom),8.0/zoom,12.8/zoom,None).unwrap();
                assert!(q.distance(on)<1e-9);
                assert!(t.leo_snap_extension(on+normal*(12.7/zoom),8.0/zoom,12.8/zoom,None).is_some());
                assert!(t.leo_snap_extension(on+normal*(12.9/zoom),8.0/zoom,12.8/zoom,None).is_none());
                assert!(t.leo_extension.get().is_none());
                assert!(t.leo_snap_extension(on+normal*(8.1/zoom),8.0/zoom,12.8/zoom,None).is_none());
                assert!(t.leo_snap_extension(on,8.0/zoom,12.8/zoom,Some(Vec2::new(1.0,0.0))).is_none());
                assert!(t.leo_snap_extension(on,8.0/zoom,12.8/zoom,Some(-dir)).is_none());
            }
        }
    }

    #[test]
    fn leo_snapping_engine_extension_feedback_is_distinct_and_only_on_capture() {
        let mut t=Targets::default();
        t.extensions.push((Point::new(300.0,300.0),Vec2::new(1.0,0.0)));
        t.style.lines=true; t.style.labels=true;
        let (q,guides)=t.leo_snap_extension(Point::new(450.0,304.0),8.0,12.8,None).unwrap();
        assert_eq!(q,Point::new(450.0,300.0));
        assert!(guides.iter().any(|o| matches!(o,Overlay::Line {dashed:true,..})));
        assert!(guides.iter().any(|o| matches!(o,Overlay::Label {text,..} if text=="extension")));
        assert!(!guides.iter().any(|o| matches!(o,Overlay::Label {text,..} if text=="path")));
        assert!(t.leo_snap_extension(Point::new(450.0,320.0),8.0,12.8,None).is_none());
        t.style.lines=false; t.style.labels=false;
        assert!(t.leo_snap_extension(Point::new(450.0,304.0),8.0,12.8,None).unwrap().1.is_empty());
        assert!(t.leo_snap_extension(Point::new(f64::NAN,0.0),8.0,12.8,None).is_none());
        assert!(t.leo_extension.get().is_none());
    }

    #[test]
    fn leo_snapping_engine_dense_tail_bezier_uses_actual_curve_at_multiple_zooms() {
        use vectorcraft_doc::{Document, Node, Appearance};
        use vectorcraft_geom::{shapes, PathData};
        let mut d=Document::new(1000.0,1000.0);
        let mut filler=shapes::rectangle(Rect::new(10000.0,10000.0,10010.0,10010.0));
        filler.subpaths=vec![filler.subpaths[0].clone();5001];
        let id=d.alloc_id();
        d.insert(Some(d.layers[0].id),0,Node::path(id,filler,Appearance::default_art())).unwrap();
        let id=d.alloc_id();
        let curve=vectorcraft_geom::kurbo::CubicBez::new((400.0,300.0),(450.0,320.0),(350.0,360.0),(400.0,380.0));
        let mut bez=BezPath::new();
        bez.move_to(curve.p0); bez.curve_to(curve.p1,curve.p2,curve.p3);
        d.insert(Some(d.layers[0].id),1,Node::path(id,PathData::from_bezpath(&bez),Appearance::default_art())).unwrap();
        let selection=Selection::default();
        let paint=crate::testutil::paint();
        let cx=crate::testutil::cx(&d,&selection,&paint);
        // t=1/4 is on the curved bulge, well away from its endpoint chord.
        let expected=Point::new(414.0625,318.125);
        for zoom in [0.25,1.0,4.0] {
            let t=Targets::leo_for_anchor_drag(&cx);
            let p=expected+Vec2::new(3.0/zoom,0.0);
            let (q,_) = t.leo_snap_anchor(p,8.0/zoom,12.8/zoom).unwrap();
            assert!(curve.nearest(q,1e-8).distance_sq<1e-12);
            t.clear_leo_snap();
            let (q,_) = t.leo_snap_anchor_along(p,Point::new(350.0,expected.y),8.0/zoom,12.8/zoom).unwrap();
            assert!(q.distance(expected)<1e-7,"zoom={zoom}: {q:?}");
        }
    }

    #[test]
    fn leo_snapping_engine_local_query_retains_segment_identity_when_candidates_change() {
        use vectorcraft_doc::{Document, Node, Appearance};
        use vectorcraft_geom::shapes;
        let mut d=Document::new(1000.0,1000.0);
        for x in [450.0,430.0] {
            let id=d.alloc_id();
            d.insert(Some(d.layers[0].id),usize::MAX,Node::path(id,
                shapes::line(Point::new(x,300.0),Point::new(x,400.0)),Appearance::default_art())).unwrap();
        }
        let selection=Selection::default();
        let paint=crate::testutil::paint();
        let cx=crate::testutil::cx(&d,&selection,&paint);
        let t=Targets::leo_for_anchor_drag(&cx);
        for (x,want) in [(434.0,430.0),(438.0,430.0),(444.0,450.0)] {
            let (q,_)=t.leo_snap_anchor(Point::new(x,350.0),8.0,12.8).unwrap();
            assert_eq!(q,Point::new(want,350.0));
        }
    }

    #[test]
    fn leo_object_move_dense_document_tail_target_is_not_truncated() {
        use vectorcraft_doc::{Document, Node, Appearance};
        use vectorcraft_geom::shapes;
        let mut d=Document::new(1000.0,1000.0);
        let mut filler=shapes::rectangle(Rect::new(10000.0,10000.0,10010.0,10010.0));
        filler.subpaths=vec![filler.subpaths[0].clone();5001];
        let id=d.alloc_id();
        d.insert(Some(d.layers[0].id),0,Node::path(id,filler,Appearance::default_art())).unwrap();
        let source=d.alloc_id();
        d.insert(Some(d.layers[0].id),1,Node::path(source,
            shapes::rectangle(Rect::new(100.0,100.0,200.0,200.0)),Appearance::default_art())).unwrap();
        let target=d.alloc_id();
        d.insert(Some(d.layers[0].id),2,Node::path(target,
            shapes::line(Point::new(300.0,100.0),Point::new(300.0,200.0)),Appearance::default_art())).unwrap();
        let mut selection=Selection::default(); selection.set([source]);
        let paint=crate::testutil::paint();
        let cx=crate::testutil::cx(&d,&selection,&paint);
        let snapper=ObjectMoveSnap::new(&cx,Some(Rect::new(100.0,100.0,200.0,200.0)));
        let (delta,guides)=snapper.snap_with_tolerance(Point::new(150.0,150.0),Vec2::new(99.0,0.0),false,8.0,12.8,None);
        assert_eq!(delta,Vec2::new(100.0,0.0));
        assert!(!guides.is_empty());
    }

    #[test]
    fn leo_object_move_shift_alignment_keeps_twelve_point_eight_pixel_release() {
        let mut t = Targets::default();
        t.xs.push((300.0, Point::new(300.0, 600.0), Kind::Edge));
        let (x, y) = (Cell::new(None), Cell::new(None));
        let sources = [Point::new(100.0, 100.0)];
        let snap = |dx| snap_move_alignment(&t, &sources, Some(Rect::new(100.0, 100.0, 200.0, 200.0)),
            Vec2::new(dx, 0.0), 8.0, 12.8, &x, &y, Some(Vec2::new(1.0, 0.0)), None);
        assert_eq!(snap(199.0).0.x, 200.0);
        assert_eq!(snap(210.0).0.x, 200.0);
        let (d, guides) = snap(213.0);
        assert_eq!(d.x, 213.0);
        assert!(guides.is_empty());
    }

    #[test]
    fn leo_snapping_engine_shift_path_is_exact_and_has_hysteresis() {
        let mut t = Targets::default();
        let line = Line::new((100.0, 0.0), (120.0, 100.0));
        t.segments.push((line.bounding_box(), PathSeg::Line(line)));
        let at = Point::new(0.0, 50.0);
        let hit = t.leo_snap_anchor_along(Point::new(105.0, 50.0), at, 8.0, 12.8).unwrap();
        assert!(hit.0.distance(Point::new(110.0, 50.0)) < 1e-7);
        assert!(!hit.1.is_empty());
        assert!(t.leo_snap_anchor_along(Point::new(121.0, 50.0), at, 8.0, 12.8).is_some());
        assert!(t.leo_snap_anchor_along(Point::new(123.0, 50.0), at, 8.0, 12.8).is_none());
    }

    #[test]
    fn leo_snapping_engine_shift_rejects_nearby_off_line_anchor() {
        let mut t = Targets::default();
        t.points.push((Point::new(100.0, 53.0), Kind::Anchor));
        assert!(t.leo_snap_anchor_along(Point::new(99.0, 50.0), Point::new(0.0, 50.0), 8.0, 12.8).is_none());
        assert!(t.leo_snap.get().is_none());
    }

    #[test]
    fn leo_snapping_engine_shift_cubic_hits_actual_curve() {
        use vectorcraft_geom::kurbo::CubicBez;
        let curve = PathSeg::Cubic(CubicBez::new((100.0, 0.0), (150.0, 30.0), (50.0, 70.0), (100.0, 100.0)));
        let mut t = Targets::default();
        t.segments.push((vectorcraft_geom::kurbo::ParamCurveExtrema::bounding_box(&curve), curve));
        let (q, _) = t.leo_snap_anchor_along(Point::new(96.0, 50.0), Point::new(0.0, 50.0), 8.0, 12.8).unwrap();
        assert!(q.distance(Point::new(100.0, 50.0)) < 1e-6);
        assert!(curve.nearest(q, 1e-8).distance_sq < 1e-12);
    }

    #[test]
    fn leo_object_move_continuous_edge_captures_node_between_box_features() {
        let edge = PathSeg::Line(Line::new((200.0, 100.0), (200.0, 200.0)));
        let mut t = Targets::default();
        t.points.push((Point::new(300.0, 137.0), Kind::Anchor));
        let mut sources = vec![];
        edge_sources(&[edge], &t, Vec2::new(96.0, 0.0), 8.0, &mut sources);
        assert!(sources.iter().any(|p| p.distance(Point::new(200.0, 137.0)) < 1e-7));
        let lock = find_move_geometry(&t, &sources, Some(Rect::new(100.0, 100.0, 200.0, 200.0)), Vec2::new(96.0, 0.0), 8.0, None).unwrap();
        let (d, q, _, _) = resolve_move_lock(&t, lock, Vec2::new(96.0, 0.0), 12.8, None).unwrap();
        assert_eq!(d, Vec2::new(100.0, 0.0));
        assert!(edge.nearest(q - d, 1e-8).distance_sq < 1e-12);
    }
}

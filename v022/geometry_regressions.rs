#[cfg(test)]
mod leo_v022_geometry_tests {
    use super::*;

    #[test]
    fn leo_snapping_engine_extensions_use_actual_visible_reference_segments() {
        let view=crate::ScreenFrame {origin:Point::ORIGIN,right:Vec2::new(1.0,0.0),down:Vec2::new(0.0,1.0),size:(100.0,100.0)};
        assert!(LeoExtension::new(Point::new(-50.0,50.0),Point::new(150.0,50.0)).unwrap().reference_in_view(Some(&view)));
        // Its bounding box overlaps, but this diagonal stays beyond the corner.
        assert!(!LeoExtension::new(Point::new(-100.0,50.0),Point::new(50.0,-100.0)).unwrap().reference_in_view(Some(&view)));
        let outside=LeoExtension::new(Point::new(50.0,1000.0),Point::new(50.0,1100.0)).unwrap();
        assert!(!outside.reference_in_view(Some(&view)));
        let scene=LeoScene {paths:vec![],rectangles:vec![],centers:false};
        let mut index=LeoExtensionIndex::new(&scene,Point::ORIGIN);
        index.insert(outside);
        assert!(index.nearest_in_view(Point::new(54.0,20.0),8.0,Some(&view)).is_none());
        // Same supporting line, distinct segment now visible. Keep both references.
        index.insert(LeoExtension::new(Point::new(50.0,60.0),Point::new(50.0,80.0)).unwrap());
        assert_eq!(index.nearest_in_view(Point::new(54.0,20.0),8.0,Some(&view)).unwrap().1,Point::new(50.0,20.0));
        let rotated=crate::ScreenFrame {origin:Point::new(100.0,0.0),right:Vec2::new(0.0,1.0),down:Vec2::new(-1.0,0.0),size:(100.0,100.0)};
        assert_eq!(index.nearest_in_view(Point::new(54.0,20.0),8.0,Some(&rotated)).unwrap().1,Point::new(50.0,20.0));
    }

    #[test]
    fn leo_snapping_engine_extension_view_updates_and_held_target_survives_pan() {
        let scene=LeoScene {paths:vec![],rectangles:vec![],centers:false};
        let mut index=LeoExtensionIndex::new(&scene,Point::ORIGIN);
        index.insert(LeoExtension::new(Point::new(50.0,60.0),Point::new(50.0,80.0)).unwrap());
        let mut t=Targets::default();t.leo_extension_index=Some(index);
        let visible=crate::ScreenFrame {origin:Point::ORIGIN,right:Vec2::new(1.0,0.0),down:Vec2::new(0.0,1.0),size:(100.0,100.0)};
        let offscreen=crate::ScreenFrame {origin:Point::new(1000.0,1000.0),..visible};
        let p=Point::new(54.0,20.0);
        assert!(t.leo_snap_extension_in_view(p,8.0,12.8,None,Some(&offscreen)).is_none());
        assert_eq!(t.leo_snap_extension_in_view(p,8.0,12.8,None,Some(&visible)).unwrap().0,Point::new(50.0,20.0));
        assert_eq!(t.leo_snap_extension_in_view(Point::new(62.7,20.0),8.0,12.8,None,Some(&offscreen)).unwrap().0,Point::new(50.0,20.0));
        assert!(t.leo_snap_extension_in_view(Point::new(62.9,20.0),8.0,12.8,None,Some(&offscreen)).is_none());
        assert!(t.leo_snap_extension_in_view(p,8.0,12.8,None,Some(&offscreen)).is_none());
        assert!(t.leo_snap_extension_in_view(p,8.0,12.8,None,Some(&visible)).is_some());
    }

    #[test]
    fn leo_snapping_engine_extension_index_finds_both_rays_at_bucket_boundaries() {
        let scene=LeoScene {paths:vec![],rectangles:vec![],centers:false};
        for i in 0..256 {
            let angle=i as f64*std::f64::consts::PI/128.0;
            let dir=Vec2::new(angle.cos(),angle.sin());
            let normal=Vec2::new(-dir.y,dir.x);
            let at=Point::new(12500.0+i as f64*7.0,7500.0);
            let line=LeoExtension::new(at,at+dir*50.0).unwrap();
            let mut index=LeoExtensionIndex::new(&scene,Point::new(400.0,500.0));
            index.insert(line);
            for along in [-500.0,1000.0] {
                let expected=at+dir*along;
                let p=expected+normal*7.9;
                let (_,q)=index.nearest(p,8.0).unwrap();
                assert!(q.distance(expected)<1e-7,"angle={angle}: {q:?}");
                assert!(index.nearest(expected+normal*8.1,8.0).is_none());
            }
        }
    }

    #[test]
    fn leo_snapping_engine_other_object_extension_after_twenty_thousand_unique_lines() {
        use vectorcraft_doc::{Document,Node,Appearance};
        use vectorcraft_geom::{PathData,shapes};
        let mut d=Document::new(1200.0,1200.0);
        let mut filler=PathData::new(vec![]);
        for i in 0..5001 {
            let (x,y)=(10000.0+i as f64*20.0,10000.0+i as f64*15.0);
            filler.subpaths.extend(shapes::rectangle(Rect::new(x,y,x+10.0,y+10.0)).subpaths);
        }
        let background=d.alloc_id();
        d.insert(Some(d.layers[0].id),0,Node::path(background,filler,Appearance::default_art())).unwrap();
        let source=d.alloc_id();
        d.insert(Some(d.layers[0].id),1,Node::path(source,shapes::line(Point::new(600.0,600.0),Point::new(500.0,400.0)),Appearance::default_art())).unwrap();
        let target=d.alloc_id();
        d.insert(Some(d.layers[0].id),2,Node::path(target,shapes::line(Point::new(300.0,100.0),Point::new(300.0,200.0)),Appearance::default_art())).unwrap();
        let mut selection=Selection::default();selection.set([source]);
        selection.anchors.insert(source,[(0,1)].into());
        let paint=crate::testutil::paint();let cx=crate::testutil::cx(&d,&selection,&paint);
        let started=std::time::Instant::now();
        let targets=Targets::leo_for_anchor_drag(&cx);
        let build_time=started.elapsed();
        assert!(targets.leo_extension_index.as_ref().unwrap().buckets.iter().map(|b| b.entries.len()).sum::<usize>()>20000);
        for y in [50.0,350.0] {
            targets.clear_leo_snap();
            let (q,guides)=targets.leo_snap_extension(Point::new(304.0,y),8.0,12.8,None).unwrap();
            assert_eq!(q,Point::new(300.0,y));
            assert!(guides.iter().any(|g| matches!(g,Overlay::Label {text,color,..} if text=="延长线吸附" && *color==[255,214,0])));
        }
        let started=std::time::Instant::now();
        for i in 0..1000 {
            targets.clear_leo_snap();
            assert!(targets.leo_snap_extension(Point::new(304.0,350.0+(i%50) as f64),8.0,12.8,None).is_some());
        }
        println!("extension index: 20005 lines, build={build_time:?}, 1000 queries={:?}",started.elapsed());
    }

    #[test]
    fn leo_snapping_engine_distant_extension_has_bounded_yellow_feedback() {
        let mut targets=Targets::default();targets.style.lines=true;targets.style.labels=true;
        targets.extensions.push((Point::new(1_000_000.0,350.0),Vec2::new(1.0,0.0)));
        let (q,guides)=targets.leo_snap_extension(Point::new(300.0,354.0),8.0,12.8,None).unwrap();
        assert_eq!(q,Point::new(300.0,350.0));
        for guide in guides {
            if let Overlay::Line {a,b,color,dashed}=guide {
                assert!(a.distance(b)<=512.0+1e-7);assert_eq!(b,q);assert!(dashed);assert_eq!(color,[255,214,0]);
            }
        }
    }

    #[test]
    fn leo_snapping_engine_indexed_extension_lock_keeps_identity_and_release() {
        let scene=LeoScene {paths:vec![],rectangles:vec![],centers:false};
        let mut index=LeoExtensionIndex::new(&scene,Point::new(500.0,400.0));
        for x in [300.0,320.0] {index.insert(LeoExtension::new(Point::new(x,100.0),Point::new(x,200.0)).unwrap());}
        for bucket in &mut index.buckets {bucket.entries.sort_by(|a,b| a.0.total_cmp(&b.0));}
        let mut t=Targets::default();t.leo_extension_index=Some(index);
        for (x,want) in [(304.0,300.0),(310.0,300.0),(314.0,320.0)] {
            assert_eq!(t.leo_snap_extension(Point::new(x,350.0),8.0,12.8,None).unwrap().0,Point::new(want,350.0));
        }
    }

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
        assert!(guides.iter().any(|o| matches!(o,Overlay::Label {text,..} if text=="延长线吸附")));
        assert!(guides.iter().any(|o| matches!(o,Overlay::Line {color,..} if *color==[255,214,0])));
        assert!(guides.iter().any(|o| matches!(o,Overlay::Path {width,..} if *width>=3.0)));
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

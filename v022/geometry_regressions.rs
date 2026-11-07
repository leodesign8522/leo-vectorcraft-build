#[cfg(test)]
mod leo_v022_geometry_tests {
    use super::*;

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

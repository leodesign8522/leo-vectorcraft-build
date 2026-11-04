#[cfg(test)]
mod leo_snapping_rotation_tool_tests {
    use super::*;
    use crate::testutil::*;
    use vectorcraft_doc::{Appearance, Node, Selection};
    use vectorcraft_geom::shapes;

    fn matrix(actions: &[Action]) -> Affine {
        let Some(Action::Preview(cmd, v)) = actions
            .iter()
            .rev()
            .find(|a| matches!(a, Action::Preview(..)))
        else {
            panic!("{actions:?}")
        };
        assert_eq!(cmd, "object.transform");
        let v = v["matrix"].as_array().unwrap();
        Affine::new(std::array::from_fn(|i| v[i].as_f64().unwrap()))
    }

    #[test]
    fn custom_pivot_rotate_lands_grabbed_endpoint_on_a_real_edge() {
        let (mut d, _) = doc_with_rect();
        let layer = d.layers[0].id;
        let (source, target) = (d.alloc_id(), d.alloc_id());
        let origin = Point::new(300.0, 340.0);
        let grab = Point::new(400.0, 340.0);
        d.insert(
            Some(layer),
            1,
            Node::path(
                source,
                shapes::line(origin, grab),
                Appearance::default_art(),
            ),
        )
        .unwrap();
        d.insert(
            Some(layer),
            2,
            Node::path(
                target,
                shapes::line(Point::new(380.0, 360.0), Point::new(380.0, 420.0)),
                Appearance::default_art(),
            ),
        )
        .unwrap();
        let mut selected = Selection::default();
        selected.set([source]);
        let paint = paint();
        let c = cx(&d, &selected, &paint);
        let mut tool = TransformTool::new(TransformKind::Rotate);
        for kind in [PointerKind::Down, PointerKind::Up] {
            tool.pointer(&c, &PointerEvent::new(kind, origin.x, origin.y));
        }
        tool.pointer(&c, &PointerEvent::new(PointerKind::Down, grab.x, grab.y));
        let drag = tool.pointer(&c, &PointerEvent::new(PointerKind::Drag, 382.0, 398.0));
        let m = matrix(&drag);
        assert!(
            tool.measure
                .as_ref()
                .unwrap()
                .0
                .distance(Point::new(380.0, 400.0))
                < 1e-7,
            "readout follows the actual snapped point"
        );
        assert!((m * origin).distance(origin) < 1e-8);
        assert!(
            (m * grab).distance(Point::new(380.0, 400.0)) < 1e-7,
            "{drag:?}"
        );
        assert!(
            (m.determinant() - 1.0).abs() < 1e-9,
            "rotation never scales the object"
        );
        let ov = tool.overlays(&c);
        assert!(
            ov.iter()
                .any(|o| matches!(o, Overlay::Label { text, .. } if text == "旋转贴边"))
        );
        assert!(
            !ov.iter()
                .any(|o| matches!(o, Overlay::Label { text, .. } if text == "align"))
        );
        let release = tool.pointer(&c, &PointerEvent::new(PointerKind::Up, 382.0, 398.0));
        assert_eq!(release.last(), Some(&Action::Commit));
        assert_eq!(matrix(&release), m);
    }

    #[test]
    fn rotation_excludes_source_edges_and_shift_keeps_45_degrees() {
        let (d, source) = doc_with_rect();
        let mut s = Selection::default();
        s.set([source]);
        let paint = paint();
        let c = cx(&d, &s, &paint);
        let snap = PointSnap::for_rotation(&c);
        let origin = Point::new(150.0, 150.0);
        let grab = Point::new(200.0, 200.0);
        let pointer = Affine::translate(origin.to_vec2())
            * Affine::rotate(0.02)
            * Affine::translate(-origin.to_vec2())
            * grab;
        let (m, _, ov) = snap.snap_rotation(&c, origin, grab, pointer, grab, false);
        assert!((m * grab).distance(pointer) < 1e-8);
        assert!(
            ov.is_empty(),
            "the selection must not snap to its own original edges"
        );
        let pointer = Point::new(210.0, 170.0);
        let (m, degrees, ov) = snap.snap_rotation(&c, origin, grab, pointer, grab, true);
        let (expected, angle) = rotate_for_drag(origin, grab, pointer, true);
        assert_eq!(m, expected);
        assert_eq!(degrees, angle);
        assert!(ov.is_empty());
    }

    #[test]
    fn corner_rotation_uses_the_grabbed_corner_not_the_offset_pointer() {
        let (mut d, source) = doc_with_rect();
        let layer = d.layers[0].id;
        let target = d.alloc_id();
        d.insert(
            Some(layer),
            1,
            Node::path(
                target,
                shapes::line(Point::new(220.0, 145.0), Point::new(220.0, 165.0)),
                Appearance::default_art(),
            ),
        )
        .unwrap();
        let mut s = Selection::default();
        s.set([source]);
        let paint = paint();
        let c = cx(&d, &s, &paint);
        // The top-right corner travels on radius sqrt(5000), crossing x=220 at y=160.
        let origin = Point::new(150.0, 150.0);
        let from = Point::new(207.0, 93.0);
        let grab = Point::new(200.0, 100.0);
        let on = Point::new(220.0, 160.0);
        let theta = (on - origin).atan2() - (grab - origin).atan2() + 0.01;
        let raw = Affine::translate(origin.to_vec2())
            * Affine::rotate(theta)
            * Affine::translate(-origin.to_vec2());
        let pointer = raw * from;
        let mut selection = crate::select::SelectionTool::default();
        selection.pointer(&c, &PointerEvent::new(PointerKind::Down, from.x, from.y));
        let result = selection.pointer(
            &c,
            &PointerEvent::new(PointerKind::Drag, pointer.x, pointer.y),
        );
        assert!((matrix(&result) * grab).distance(on) < 1e-7, "{result:?}");
        let mut free = crate::xform::FreeTransformTool::default();
        free.pointer(&c, &PointerEvent::new(PointerKind::Down, from.x, from.y));
        let result = free.pointer(
            &c,
            &PointerEvent::new(PointerKind::Drag, pointer.x, pointer.y),
        );
        assert!((matrix(&result) * grab).distance(on) < 1e-7, "{result:?}");
    }
}

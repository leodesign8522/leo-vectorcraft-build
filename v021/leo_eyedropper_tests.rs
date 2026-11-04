fn leo_eyedropper_text(s: &Session, id: NodeId) -> vectorcraft_doc::TextObject {
    let NodeKind::Text(t) = node(s, id).kind else {
        panic!("not text")
    };
    *t
}

fn leo_eyedropper_effective(s: &Session, id: NodeId) -> vectorcraft_doc::CharStyle {
    let t = leo_eyedropper_text(s, id);
    t.effective_char_style(&t.first_style()).unwrap()
}

#[test]
fn leo_eyedropper_scaled_text_matches_displayed_attributes_and_undo() {
    for (source_matrix, target_matrix) in [
        (
            [2.0, 0.0, 0.0, 3.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        ),
        (
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.5, 0.0, 0.0, 4.0, 0.0, 0.0],
        ),
        (
            [0.0, -2.0, 3.0, 0.0, 0.0, 0.0],
            [0.0, 1.5, -0.75, 0.0, 0.0, 0.0],
        ),
        (
            [-2.0, 0.0, 0.4, 1.5, 0.0, 0.0],
            [1.0, 0.0, 0.2, 2.0, 0.0, 0.0],
        ),
    ] {
        let mut s = session();
        let src = id_of(
            s.execute("text.create", &json!({"x": 20, "y": 100, "text": "Source"}))
                .unwrap(),
        );
        s.execute(
            "text.setStyle",
            &json!({"font": "Inter", "size": 17, "leading": 23, "tracking": 42, "fill": "#00ff00"}),
        )
        .unwrap();
        s.execute(
            "text.setFormat",
            &json!({"hScale": 125, "vScale": 90, "baselineShift": 3}),
        )
        .unwrap();
        s.execute("object.transform", &json!({"matrix": source_matrix}))
            .unwrap();
        let expected = leo_eyedropper_effective(&s, src);
        let source_before = leo_eyedropper_text(&s, src);
        let dst = id_of(
            s.execute(
                "text.create",
                &json!({"x": 20, "y": 300, "text": "Destination"}),
            )
            .unwrap(),
        );
        s.execute("object.transform", &json!({"matrix": target_matrix}))
            .unwrap();
        let before = leo_eyedropper_text(&s, dst);
        s.execute("appearance.copyFrom", &json!({"source": src.0}))
            .unwrap();
        let copied = leo_eyedropper_text(&s, dst);
        let actual = leo_eyedropper_effective(&s, dst);
        for (a, b) in [
            (actual.size, expected.size),
            (actual.leading.unwrap(), expected.leading.unwrap()),
            (actual.baseline_shift, expected.baseline_shift),
            (actual.h_scale, expected.h_scale),
            (actual.v_scale, expected.v_scale),
            (actual.tracking, expected.tracking),
        ] {
            assert!((a - b).abs() < 1e-8, "{a} != {b}");
        }
        assert_eq!(
            (actual.font_family, actual.font_style),
            (expected.font_family, expected.font_style)
        );
        assert_eq!(actual.fill.color().unwrap().to_hex(), "#00ff00");
        assert_eq!(
            copied.xf, before.xf,
            "sampling never moves or rotates the target"
        );
        assert_eq!(copied.plain_text(), before.plain_text());
        assert_eq!(leo_eyedropper_text(&s, src), source_before);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(leo_eyedropper_text(&s, dst), before);
        s.execute("edit.redo", &json!({})).unwrap();
        assert!((leo_eyedropper_effective(&s, dst).size - expected.size).abs() < 1e-8);
    }
}

#[test]
fn leo_eyedropper_scaled_text_respects_either_disabled_character_option() {
    for key in ["pickUp", "apply"] {
        let mut s = session();
        let src = id_of(
            s.execute(
                "text.create",
                &json!({"x": 20, "y": 100, "text": "Source", "size": 30}),
            )
            .unwrap(),
        );
        s.execute("paint.setFill", &json!({"color": "#00ff00"}))
            .unwrap();
        s.execute("object.scale", &json!({"sx": 200})).unwrap();
        let dst = id_of(
            s.execute(
                "text.create",
                &json!({"x": 20, "y": 300, "text": "Target", "size": 12}),
            )
            .unwrap(),
        );
        s.execute("object.scale", &json!({"sx": 50, "sy": 150}))
            .unwrap();
        let before = leo_eyedropper_effective(&s, dst);
        let mut options = json!({});
        options[key] = json!({"character": false});
        s.execute("eyedropper.setOptions", &options).unwrap();
        s.execute("appearance.copyFrom", &json!({"source": src.0}))
            .unwrap();
        let after = leo_eyedropper_effective(&s, dst);
        assert_eq!(
            (after.size, after.h_scale, after.leading, after.tracking),
            (before.size, before.h_scale, before.leading, before.tracking)
        );
        assert_eq!(after.fill.color().unwrap().to_hex(), "#00ff00");
    }
}

#[test]
fn leo_eyedropper_collapsed_destination_rejects_before_partial_changes() {
    let mut s = session();
    let src = id_of(
        s.execute(
            "text.create",
            &json!({"x": 20, "y": 100, "text": "Source", "size": 30}),
        )
        .unwrap(),
    );
    s.execute("paint.setFill", &json!({"color": "#00ff00"}))
        .unwrap();
    let a = id_of(
        s.execute("text.create", &json!({"x": 20, "y": 300, "text": "Valid"}))
            .unwrap(),
    );
    let b = id_of(
        s.execute(
            "text.create",
            &json!({"x": 20, "y": 400, "text": "Collapsed"}),
        )
        .unwrap(),
    );
    s.execute("object.transform", &json!({"matrix": [0, 0, 0, 0, 0, 0]}))
        .unwrap();
    s.execute("select.set", &json!({"ids": [a.0, b.0]}))
        .unwrap();
    let before = (
        leo_eyedropper_text(&s, a),
        leo_eyedropper_text(&s, b),
        s.paint.fill.clone(),
        undo_len(&s),
    );
    assert!(
        s.execute("appearance.copyFrom", &json!({"source": src.0}))
            .is_err()
    );
    assert_eq!(
        (
            leo_eyedropper_text(&s, a),
            leo_eyedropper_text(&s, b),
            s.paint.fill.clone(),
            undo_len(&s)
        ),
        before
    );
    assert!(s.eyedropper_sample.is_none());
}

#[test]
fn leo_eyedropper_scaled_range_only_changes_highlighted_characters() {
    let mut s = session();
    s.prefs.placeholder_text = false;
    let src = id_of(
        s.execute(
            "text.create",
            &json!({"x": 300, "y": 100, "text": "Source", "size": 20}),
        )
        .unwrap(),
    );
    s.execute("object.scale", &json!({"sx": 150})).unwrap();
    let expected = leo_eyedropper_effective(&s, src);
    let view = ViewInfo::default();
    s.select_tool("type", view).unwrap();
    for kind in [PointerKind::Down, PointerKind::Up] {
        s.pointer(&PointerEvent::new(kind, 20.0, 300.0), view)
            .unwrap();
    }
    let dst = s.doc().unwrap().selection.objects[0];
    s.tool_text("MAMA NA KAZI", view).unwrap();
    if let NodeKind::Text(t) = &mut s.doc_mut().unwrap().doc.node_mut(dst).unwrap().kind {
        t.xf *= vectorcraft_geom::Affine::scale_non_uniform(0.5, 2.0);
    }
    let before = leo_eyedropper_text(&s, dst);
    s.set_tool_option("select", &json!({"start": 0, "end": 4}));
    s.select_tool("eyedropper", view).unwrap();
    s.execute("appearance.copyFrom", &json!({"source": src.0}))
        .unwrap();
    let t = leo_eyedropper_text(&s, dst);
    assert_eq!(t.runs.len(), 2);
    assert_eq!(t.runs[0].text, "MAMA");
    assert!((t.effective_char_style(&t.runs[0].style).unwrap().size - expected.size).abs() < 1e-8);
    assert_eq!(t.runs[1].style, before.first_style());
    assert_eq!(t.xf, before.xf);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(leo_eyedropper_text(&s, dst), before);
}

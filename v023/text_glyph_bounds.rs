/// Whether selected editable text requests ink controls. Rendering, export, wrapping,
/// and the document's geometric bounds deliberately continue to use the real frame.
pub fn leo_has_glyph_bounds(doc: &Document, ids: &[NodeId]) -> bool {
    let mut found = false;
    for n in ids.iter().filter_map(|id| doc.node(*id)) {
        n.walk(&mut |c| {
            found |= matches!(&c.kind, NodeKind::Text(t) if t.leo_glyph_bounds);
        });
    }
    found
}

fn leo_ink_bounds(n: &Node, symbols: &SymbolBoxes, a: Affine, preview: bool) -> Option<Rect> {
    // Effects/perspective/clips retain native controls: their drawn result need not
    // coincide with the original glyph outlines. Never change their rendering model.
    if !reshapes(n) && n.extras().perspective.is_none() {
        if let NodeKind::Text(t) = &n.kind {
            if t.leo_glyph_bounds {
                let lay = vectorcraft_text::layout(vectorcraft_text::FontDb::global(), t);
                let xf = a * t.xf;
                let mut path = lay.to_bezpath();
                path.apply_affine(xf);
                let mut ink = (!path.elements().is_empty()).then(|| path.bounding_box());
                for g in lay.glyphs.iter().chain(&lay.ruby) {
                    if let Some((image, placed)) = g.placed_image() {
                        ink = union_opt(ink, Some((xf * placed).transform_rect_bbox(image.rect())));
                    }
                    if let Some(c) = &g.colour {
                        ink = union_opt(ink, c.bounds_in(symbols, xf * g.xf, false));
                    }
                }
                for i in &lay.inlines {
                    ink = union_opt(ink, Some(xf.transform_rect_bbox(i.bounds)));
                }
                if let Some(b) = ink {
                    let stroke = if preview { t.char_stroke_outset() } else { 0.0 };
                    return Some(b.inflate(stroke, stroke));
                }
                // Empty/whitespace-only text keeps usable controls instead of a zero box.
            }
        }
        if matches!(n.kind, NodeKind::Group { .. } | NodeKind::Layer { .. }) && !n.clips() && !has_own_paint(n) {
            return n.editing_children().iter().fold(None, |b, c| union_opt(b, leo_ink_bounds(c, symbols, a, preview)));
        }
    }
    if preview { bounds_in(symbols, n, a) } else { n.bounds_in(symbols, a, false) }
}

/// Selection-only measurement. Existing document geometry and text layout stay intact.
pub fn leo_selection_bounds(doc: &Document, ids: &[NodeId], preview: bool) -> Option<Rect> {
    if !leo_has_glyph_bounds(doc, ids) { return bounds_of(doc, ids, preview); }
    let symbols = doc.symbol_boxes();
    ids.iter().filter_map(|id| doc.node(*id)).fold(None, |b, n| union_opt(b, leo_ink_bounds(n, &symbols, Affine::IDENTITY, preview)))
}

pub fn leo_selection_box(doc: &Document, ids: &[NodeId], preview: bool) -> Option<OrientedBox> {
    if !leo_has_glyph_bounds(doc, ids) { return oriented_bounds(doc, ids, preview); }
    let angle = doc.bbox_angle(ids);
    let a = OrientedBox { angle, rect: Rect::ZERO }.to_doc().inverse();
    let symbols = doc.symbol_boxes();
    let rect = ids.iter().filter_map(|id| doc.node(*id)).fold(None, |b, n| union_opt(b, leo_ink_bounds(n, &symbols, a, preview)))?;
    Some(OrientedBox { angle, rect })
}

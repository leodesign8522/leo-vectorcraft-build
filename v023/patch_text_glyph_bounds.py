#!/usr/bin/env python3
"""Selection-only glyph bounds: never modify the text's layout frame or content."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
here = Path(__file__).parent

def edit(rel, old, new, count=1):
    p = root / rel
    s = p.read_text(encoding='utf-8')
    if s.count(old) != count:
        raise SystemExit(f'FAIL glyph selection bounds: {rel}: expected {count} anchors, got {s.count(old)}')
    p.write_text(s.replace(old, new), encoding='utf-8', newline='\n')

edit('crates/doc/src/text.rs', '    pub optical_margin: bool,', '''    pub optical_margin: bool,
    /// Leo: selection/transform controls use visible ink; the layout frame stays intact.
    #[serde(default, rename = "leoGlyphBounds", skip_serializing_if = "crate::skip::is_default")]
    pub leo_glyph_bounds: bool,''')
edit('crates/doc/src/text/object.rs', '            optical_margin: false,', '            optical_margin: false,\n            leo_glyph_bounds: false,')
edit('crates/engine/src/cmd/textedit.rs', '        optical_margin: false,', '        optical_margin: false,\n        leo_glyph_bounds: false,', 2)

p = root / 'crates/effects/src/preview.rs'
with p.open('a', encoding='utf-8') as f:
    f.write('\n' + (here / 'text_glyph_bounds.rs').read_text(encoding='utf-8'))

for rel in ['crates/tools/src/select.rs', 'crates/engine/src/cmd/object/align.rs']:
    p = root / rel
    s = p.read_text(encoding='utf-8')
    # These two files measure selection controls, not rendering/layout geometry.
    if rel.endswith('align.rs'):
        s = s.replace('preview::bounds_of(&self.doc()', 'preview::leo_selection_bounds(&self.doc()')
        s = s.replace('preview::oriented_bounds(&self.doc()', 'preview::leo_selection_box(&self.doc()')
    else:
        s = s.replace('preview::bounds_of(', 'preview::leo_selection_bounds(')
        s = s.replace('preview::oriented_bounds(', 'preview::leo_selection_box(')
    p.write_text(s, encoding='utf-8', newline='\n')

edit('crates/engine/src/cmd/object/align.rs',
     'return effects::preview::bounds_of(d, ids, self.preview);',
     'return effects::preview::leo_selection_bounds(d, ids, self.preview);')
edit('crates/engine/src/cmd/object/align.rs',
     '    fn of(self, d: &Document, id: NodeId) -> Option<Rect> {\n',
     '''    fn of(self, d: &Document, id: NodeId) -> Option<Rect> {
        // Per-object ink controls must also define Align/Distribute measurements,
        // including text nested in ordinary groups and key-object references.
        if effects::preview::leo_has_glyph_bounds(d, &[id]) {
            return effects::preview::leo_selection_bounds(d, &[id], self.preview);
        }
''')

edit('crates/tools/src/select.rs',
     'let is_area = |id: &NodeId| cx.doc.node(*id).is_some_and(is_area_type);',
     'let is_area = |id: &NodeId| cx.doc.node(*id).is_some_and(|n| is_area_type(n) && !matches!(&n.kind, vectorcraft_doc::NodeKind::Text(t) if t.leo_glyph_bounds));')
edit('crates/engine/src/cmd/object/bounds.rs',
     'let frame = d.oriented_bounds(&ids, false).ok_or_else',
     'let glyphs = effects::preview::leo_has_glyph_bounds(d, &ids);\n    let frame = effects::preview::leo_selection_box(d, &ids, false).ok_or_else')
edit('crates/engine/src/cmd/object/bounds.rs',
     '(effects::preview::oriented_bounds(d, &ids, true).map_or(g, |b| b.rect), scaling_box(d, &ids, &frame).unwrap_or(g))',
     '(effects::preview::leo_selection_box(d, &ids, true).map_or(g, |b| b.rect), if glyphs { g } else { scaling_box(d, &ids, &frame).unwrap_or(g) })')
edit('crates/engine/src/cmd/object/bounds.rs',
     'let solved = if !(s.prefs.use_preview_bounds && w >= 0.0 && h >= 0.0) {',
     'let solved = if glyphs || !(s.prefs.use_preview_bounds && w >= 0.0 && h >= 0.0) {')

edit('crates/engine/src/cmd/typecmd.rs', '    vec![\n', '''    vec![
        cmd!("type.glyphSelectionBounds", "贴合字形选框", ["Type"], None,
            "{ids?|id?,on?} toggle tight glyph selection controls without changing layout → {on,ids}",
            has_selection, leo_glyph_selection_bounds),
''')
p = root / 'crates/engine/src/cmd/typecmd.rs'
with p.open('a', encoding='utf-8') as f:
    f.write('''
fn leo_glyph_selection_bounds(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "type.glyphSelectionBounds";
    let ids = text_targets(s, p, C)?;
    let first = ids.first().and_then(|id| match &s.doc().ok()?.doc.node(*id)?.kind {
        NodeKind::Text(t) => Some(t.leo_glyph_bounds), _ => None,
    });
    let on = switch_or(p, "on", !first.unwrap_or(false), C)?;
    s.edit("贴合字形选框", |d, _| {
        for id in &ids {
            if let Some(NodeKind::Text(t)) = d.node_mut(*id).map(|n| &mut n.kind) {
                t.leo_glyph_bounds = on;
            }
        }
        Ok(())
    })?;
    Ok(json!({"on":on,"ids":ids.iter().map(|id|id.0).collect::<Vec<_>>()}))
}
''')
    f.write((here / 'text_glyph_regressions.rs').read_text(encoding='utf-8'))

edit('crates/ui-egui/src/panels/character.rs',
     '    metrics_grid(app, ui, "ch-grid", &s, w, !hidden);',
     '''    metrics_grid(app, ui, "ch-grid", &s, w, !hidden);
    ui.add_space(4.0);
    let tight = shared_text_flag(app, |t| t.leo_glyph_bounds) == Some(true);
    if ui.selectable_label(tight, "贴合字形选框")
        .on_hover_text("选框和对象对齐按实际字形计算；保留字号和换行。再次点击恢复排版框。文字仍可编辑。")
        .clicked()
    {
        let p = if let Some((id, _, _)) = text_editing(app) {
            json!({"id":id.0,"on":!tight})
        } else { json!({"on":!tight}) };
        app.run("type.glyphSelectionBounds", p).ok();
    }''')

# The Type tool still draws/edits its true frame. Selection must not show a second,
# misleading metric frame or baseline on top of the ink controls.
edit('crates/ui-egui/src/canvas/selection_marks.rs',
     'if !matches!(n.kind, NodeKind::Text(ref tx) if matches!(tx.kind, vectorcraft_doc::TextKind::Point)) {',
     'if !matches!(n.kind, NodeKind::Text(ref tx) if matches!(tx.kind, vectorcraft_doc::TextKind::Point) || tx.leo_glyph_bounds) {')
edit('crates/ui-egui/src/canvas/selection_marks.rs',
     '&& matches!(tx.kind, vectorcraft_doc::TextKind::Point)\n',
     '&& matches!(tx.kind, vectorcraft_doc::TextKind::Point) && !tx.leo_glyph_bounds\n')
edit('crates/ui-egui/src/canvas/outline.rs',
     '        NodeKind::Text(t) => {\n',
     '''        NodeKind::Text(t) => {
            // Ink controls replace the metric outline even when this text is nested
            // in a selected group. The Type tool draws its real editing frame itself.
            if t.leo_glyph_bounds { return; }
''')

# Selection's ports remain available from the Type tool, where the real frame is visible.
edit('crates/tools/src/text/ports.rs',
     'let Some(t) = ported(cx.doc, *id) else { continue };',
     'let Some(t) = ported(cx.doc, *id).filter(|t| !t.leo_glyph_bounds) else { continue };')
edit('crates/tools/src/text/ports.rs',
     '    port_near(cx, p).map(|(id, port, _)| (id, port))',
     '    port_near(cx, p).filter(|(id, _, _)| ported(cx.doc, *id).is_some_and(|t| !t.leo_glyph_bounds)).map(|(id, port, _)| (id, port))')
print('Applied non-destructive text glyph selection bounds')

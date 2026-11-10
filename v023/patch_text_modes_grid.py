#!/usr/bin/env python3
"""Explicit text modes, creation defaults, Eyedropper object options and grid priority."""
from pathlib import Path
import sys
root=Path(sys.argv[1]);here=Path(__file__).parent
def edit(rel,old,new,count=1):
    p=root/rel;s=p.read_text(encoding='utf-8')
    if s.count(old)!=count:raise SystemExit(f'FAIL text modes/grid: {rel}: expected {count}, found {s.count(old)}: {old[:70]}')
    p.write_text(s.replace(old,new),encoding='utf-8',newline='\n')

# Defaults apply to new editable text, not to old files/imported objects. Explicit
# saved per-object modes continue to deserialize exactly as before.
edit('crates/engine/src/cmd/create.rs',
     'let mut t = TextObject::point(Point::new(x, y), text, new_type_style(s, p));',
     'let mut t = TextObject::point(Point::new(x, y), text, new_type_style(s, p));\n    t.leo_glyph_bounds = true;')
edit('crates/engine/src/cmd/textedit.rs',
     '        leo_glyph_bounds: false,', '        leo_glyph_bounds: true,',2)
edit('crates/engine/src/prefs.rs','    pub auto_size_area_type: bool,', '''    pub auto_size_area_type: bool,
    /// One-time migration of the new-area default; later user choices stay saved.
    #[serde(default)]
    pub leo_text_defaults_v1: bool,''')
edit('crates/engine/src/prefs.rs','            auto_size_area_type: false,',
     '            auto_size_area_type: true,\n            leo_text_defaults_v1: true,')
# This import regression deliberately needs overflow from a fixed first frame.
# Specify that fixture precondition now that NEW area text defaults to Auto Height.
# The assertion that the second frame receives/outlines text remains unchanged.
edit('crates/engine/src/cmd/fileio/tests_aiimport.rs',
     'fn outlined_threaded_type_of_the_editing_data_shows_the_text_of_every_frame() {\n    let mut s = Session::new();',
     'fn outlined_threaded_type_of_the_editing_data_shows_the_text_of_every_frame() {\n    let mut s = Session::new();\n    s.prefs.auto_size_area_type = false; // Fixed-frame threading fixture, independent of creation defaults.')
edit('crates/ui-egui/src/prefs_dialog.rs',
     '    app.session.apply_prefs(p);', '''    if !p.leo_text_defaults_v1 {
        p.auto_size_area_type = true;
        p.leo_text_defaults_v1 = true;
        // Preserve opaque preferences owned by optional modules.
        if let Some(o) = app.ui.engine_prefs.as_object_mut() {
            o.insert("autoSizeAreaType".into(), serde_json::json!(true));
            o.insert("leoTextDefaultsV1".into(), serde_json::json!(true));
        }
    }
    app.session.apply_prefs(p);''')

# Object-level text settings are separate from character/paragraph attributes.
edit('crates/engine/src/cmd/xform.rs','    pub paragraph: bool,\n}', '''    pub paragraph: bool,
    #[serde(rename = "textFrameMode")]
    pub text_frame_mode: bool,
    #[serde(rename = "glyphBounds")]
    pub glyph_bounds: bool,
}''')
edit('crates/engine/src/cmd/xform.rs',
     'Self { appearance: Default::default(), character: true, paragraph: true }',
     'Self { appearance: Default::default(), character: true, paragraph: true, text_frame_mode: true, glyph_bounds: true }')
edit('crates/ui-egui/src/dialogs/eyedropper.rs',
     '    ("/paragraph", "Paragraph Style", 0),',
     '    ("/paragraph", "Paragraph Style", 0),\n    ("/textFrameMode", "文字框高度模式", 0),\n    ("/glyphBounds", "贴合字形选框", 0),')
edit('crates/engine/src/cmd/appearance.rs',
     '    text: Option<(CharStyle, ParaStyle)>,',
     '    text: Option<(CharStyle, ParaStyle)>,\n    text_modes: Option<(Option<vectorcraft_doc::AreaFit>, bool)>,')
edit('crates/engine/src/cmd/appearance.rs',
     '        Self { appearance, bounds, opacity: n.opacity, blend: n.blend, text }',
     '''        let text_modes = match &n.kind {
            NodeKind::Text(t) => Some((matches!(t.kind, vectorcraft_doc::TextKind::Area { .. }).then_some(t.area.fit), t.leo_glyph_bounds)),
            _ => None,
        };
        Self { appearance, bounds, opacity: n.opacity, blend: n.blend, text, text_modes }''')
edit('crates/engine/src/cmd/appearance.rs',
     '            if text.is_some() && (a.character || a.paragraph) {',
     '''            if let Some((fit, glyphs)) = self.text_modes {
                if a.glyph_bounds { t.leo_glyph_bounds = glyphs; }
                if a.text_frame_mode && matches!(t.kind, vectorcraft_doc::TextKind::Area { .. }) {
                    if let Some(fit) = fit { t.area.fit = fit; }
                }
            }
            if (text.is_some() && (a.character || a.paragraph)) || (self.text_modes.is_some() && (a.glyph_bounds || a.text_frame_mode)) {''')

# Real state, including selected group descendants and text being edited.
p=root/'crates/ui-egui/src/panels/paragraph.rs';s=p.read_text(encoding='utf-8')
start=s.index('    let area_selected = app.session.active().is_some_and(|st| {')
end=s.index('    ui.add_space(4.0);',start)
s=s[:start]+'''    let (area_selected, mode) = leo_area_height_state(app);
    ui.horizontal_wrapped(|ui| {
        for (fit, label, key) in [
            (vectorcraft_doc::AreaFit::AutoHeight, "自动贴合高度", "autoHeight"),
            (vectorcraft_doc::AreaFit::None, "固定框高", "none"),
        ] {
            let on = mode == Some(fit);
            let text = if on { format!("✓ {label}") } else { label.to_owned() };
            if ui.add_enabled(area_selected, egui::Button::new(text).selected(on)).clicked() {
                if let Err(e) = app.run("text.areaOptions", json!({"fit":key})) { app.ui.status=e; }
            }
        }
    });
    if area_selected {
        let label = match mode {
            Some(vectorcraft_doc::AreaFit::AutoHeight) => "当前：自动贴合高度",
            Some(vectorcraft_doc::AreaFit::None) => "当前：固定框高",
            Some(_) => "当前：缩小文字适应框",
            None => "混合状态",
        };
        ui.label(label);
    }
'''+s[end:]
s+='''
pub(crate) fn leo_area_height_state(app: &VectorcraftApp) -> (bool, Option<vectorcraft_doc::AreaFit>) {
    let Some(st) = app.session.active() else { return (false, None) };
    let roots = super::character::text_editing(app).map(|(id,_,_)|vec![id]).unwrap_or_else(|| st.selection.objects.clone());
    let (mut seen, mut mixed, mut mode) = (false, false, None);
    for n in roots.iter().filter_map(|id|st.doc.node(*id)) {
        n.walk(&mut |n| {
            if let NodeKind::Text(t) = &n.kind {
                if matches!(t.kind, vectorcraft_doc::TextKind::Area { .. }) {
                    if seen && mode != Some(t.area.fit) { mixed = true; }
                    if !seen { mode = Some(t.area.fit); }
                    seen = true;
                }
            }
        });
    }
    (seen, if mixed { None } else { mode })
}
'''
p.write_text(s,encoding='utf-8',newline='\n')

# Grid temporarily suspends object geometry; it does not overwrite the user's
# Smart Guides preference. Guards also drop pre-grid capture locks mid-drag.
edit('crates/engine/src/tooling.rs','            smart_guides: view.smart_guides,','            smart_guides: view.smart_guides && !view.snap_to_grid,')
edit('crates/engine/src/tooling.rs','            snap_to_point: view.snap_to_point,','            snap_to_point: view.snap_to_point && !view.snap_to_grid,')
edit('crates/tools/src/guides/snap.rs',
     '(cx.smart_guides || cx.snap_to_point).then(||',
     '(!cx.snap_to_grid && (cx.smart_guides || cx.snap_to_point)).then(||',3)
edit('crates/tools/src/guides/snap.rs',
     '        let constrained = if shift { from.and_then(|f| f.constrained(p)) } else { None };',
     '''        if cx.snap_to_grid {
            if let Some((targets, _)) = &self.targets { targets.leo_snap.set(None); targets.leo_extension.set(None); }
            return self.snap_from(cx, p, if shift { from } else { None });
        }
        let constrained = if shift { from.and_then(|f| f.constrained(p)) } else { None };''')
edit('crates/tools/src/guides/snap.rs','if cx.snap_to_pixel {','if cx.snap_to_pixel && !cx.snap_to_grid {',2)
edit('crates/tools/src/guides/leo_snapping_engine.rs','        self.leo_scene.as_ref()?;',
     '        if cx.snap_to_grid { if let Ok(mut lock)=self.leo_scale.try_borrow_mut() { *lock=None; } return None; }\n        self.leo_scene.as_ref()?;')
for name in ['snap','snap_along']:
    old=f'    pub(crate) fn {name}(&self, cx: &ToolContext, start: Point, d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {{'
    edit('crates/tools/src/guides/leo_snapping_engine.rs',old,old+'\n        if cx.snap_to_grid { self.reset_locks(); return (d, vec![]); }')
    old=f'    pub(crate) fn {name}(&self, cx: &ToolContext, start: Point, mut d: Vec2, copy: bool) -> (Vec2, Vec<Overlay>) {{'
    edit('crates/tools/src/select.rs',old,old+f'''
        if cx.snap_to_grid && (self.leo.is_some() || self.targets.is_some() || self.points.is_some() || self.copy_targets.is_some() || self.copy_points.is_some() || self.rulers.is_some()) {{
            if let Some(leo) = &self.leo {{ let _ = leo.{name}(cx, start, d, copy); }}
            return Self::grid_only(self.bounds).{name}(cx, start, d, copy);
        }}''')
edit('crates/tools/src/guides/leo_snapping_engine.rs',
     'if !(cx.smart_guides || cx.snap_to_point) || !origin.is_finite()',
     'if cx.snap_to_grid || !(cx.smart_guides || cx.snap_to_point) || !origin.is_finite()')
edit('crates/tools/src/guides/leo_snapping_engine.rs',
     'if shift || !raw.is_finite() || !pointer.is_finite()',
     'if cx.snap_to_grid || shift || !raw.is_finite() || !pointer.is_finite()')
edit('crates/tools/src/xform/transform.rs','if self.rotation.is_some() {(p,vec![])}',
     'if self.rotation.is_some() && !cx.snap_to_grid {(p,vec![])}')

for rel,source in [('crates/engine/src/cmd/typecmd.rs','text_mode_regressions.rs'),
                   ('crates/ui-egui/src/panels/paragraph.rs','text_mode_ui_regressions.rs')]:
    with (root/rel).open('a',encoding='utf-8') as f:f.write('\n'+(here/source).read_text(encoding='utf-8'))
print('Applied text mode sampling/defaults/state and grid priority')

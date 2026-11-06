#!/usr/bin/env python3
"""Add independent geometric target and feedback controls to the snapping module."""
from pathlib import Path
import sys

root = Path(sys.argv[1]) / "crates"


def edit(path, old, new, count=1):
    file = root / path
    src = file.read_text(encoding="utf-8")
    if src.count(old) != count:
        raise SystemExit(f"{path}: expected {count} occurrence(s) of {old[:100]!r}, found {src.count(old)}")
    file.write_text(src.replace(old, new), encoding="utf-8", newline="\n")


prefs = "engine/src/prefs.rs"
spec = "engine/src/cmd/prefscmds.rs"
ctx = "tools/src/context.rs"
tools = "tools/src/lib.rs"
engine = "engine/src/tooling.rs"
guides = "tools/src/guides.rs"
targets = "tools/src/guides/targets.rs"
snap = "tools/src/guides/snap.rs"
resolver = "tools/src/guides/leo_snapping.rs"
move = "tools/src/guides/leo_move_contact.rs"
rotation = "tools/src/guides/leo_rotation.rs"
dialog = "ui-egui/src/prefs_dialog.rs"

# Key, label, default attraction, existing swatch (or a new RGB default).
rows = [
    ("Anchor", "路径锚点", True, "Anchor"),
    ("Midpoint", "线段与曲线中点", False, "#f5a9ff"),
    ("Path", "线段与曲线", True, "Path"),
    ("Text", "可编辑文字轮廓", True, "#7ae582"),
    ("Intersection", "路径与参考线交点", True, "#ffcc66"),
    ("Center", "对象中心", True, "Center"),
    ("Bounds", "对象边界框", True, "#96d7fa"),
    ("Artboard", "画板边缘", True, "#bba8ff"),
    ("Bleed", "出血边缘", True, "#f49bbb"),
    ("Ruler", "标尺参考线", True, "#35d7ff"),
    ("Horizontal", "水平对齐", True, "Horizontal"),
    ("Vertical", "垂直对齐", True, "Vertical"),
    ("Spacing", "等间距", None, "Spacing"),
    ("Construction", "构造角度", None, "Construction"),
    ("Extension", "原线延长／缩短", True, "#ffb33d"),
    ("Tangent", "曲线切点", None, "#48d9b5"),
    ("Perpendicular", "线段垂足", None, "#9fb7ff"),
    ("Rotation", "旋转抓点贴边", True, "#ffb33d"),
]

prefs_fields, prefs_defaults, specs, style_fields, style_defaults, propagation, ui_rows = ([] for _ in range(7))
for name, label, default, color in rows:
    field = name.lower()
    feedback = f"leo_{field}_feedback"
    enabled = f"leo_{field}_snap"
    ui_feedback = f"leo{name}Feedback"
    ui_enabled = f"leo{name}Snap"
    prefs_fields.append(f"    pub {feedback}: bool,")
    prefs_defaults.append(f"            {feedback}: true,")
    specs.append(f'    p!("{ui_feedback}", "Leo Core Bridge 吸附", "吸附目标", "{label}", bool),')
    style_fields.append(f"    pub(super) {feedback}: bool,")
    style_defaults.append(f"            {feedback}: true,")
    propagation.append(f"                {feedback}: self.prefs.{feedback},")
    if default is not None:
        value = str(default).lower()
        prefs_fields.append(f"    pub {enabled}: bool,")
        prefs_defaults.append(f"            {enabled}: {value},")
        specs.append(f'    p!("{ui_enabled}", "Leo Core Bridge 吸附", "吸附目标", "{label}", bool),')
        style_fields.append(f"    pub(super) {enabled}: bool,")
        style_defaults.append(f"            {enabled}: {value},")
        propagation.append(f"                {enabled}: self.prefs.{enabled},")
    else:
        ui_enabled = {
            "Spacing": "spacingGuides", "Construction": "constructionGuides",
            "Tangent": "snapTangent", "Perpendicular": "snapPerpendicular",
        }[name]
    if color.startswith("#"):
        color_field = f"leo_{field}_color"
        color_key = f"leo{name}Color"
        rgb = ", ".join("0x" + color[i:i+2] for i in (1, 3, 5))
        prefs_fields.append(f"    pub {color_field}: String,")
        prefs_defaults.append(f'            {color_field}: s("{color}"),')
        specs.append(f'    p!("{color_key}", "Leo Core Bridge 吸附", "吸附目标", "{label}", color),')
        style_fields.append(f"    pub(super) {color_field}: [u8; 3],")
        style_defaults.append(f"            {color_field}: [{rgb}],")
        propagation.append(f"                {color_field}: rgb(&self.prefs.{color_field}, [{rgb}]),")
    else:
        color_key = f"smartGuide{color}Color"
    ui_rows.append(f'    ("{ui_enabled}", "{label}", "{color_key}", "{ui_feedback}"),')

def block(lines):
    return "\n".join(lines) + "\n"


edit(prefs, "    pub alignment_guides: bool,", block(prefs_fields) + "    pub alignment_guides: bool,")
edit(prefs, "            alignment_guides: true,", block(prefs_defaults) + "            alignment_guides: true,")
edit(spec, '    p!("alignmentGuides", "Leo Core Bridge 吸附"', block(specs) + '    p!("alignmentGuides", "Leo Core Bridge 吸附"')
edit(ctx, "    pub smart_guide_color: [u8; 3],", "    pub leo_snap_options: crate::guides::LeoSnapOptions,\n    pub smart_guide_color: [u8; 3],")
edit(tools, "            smart_guide_color: guides::MAGENTA,", "            leo_snap_options: guides::LeoSnapOptions::default(),\n            smart_guide_color: guides::MAGENTA,")
edit(engine, "            smart_guide_color: rgb(", "            leo_snap_options: vectorcraft_tools::guides::LeoSnapOptions {\n" + block(propagation) + "            },\n            smart_guide_color: rgb(")
# The engine crate constructs this structure, while rendering and attraction live in tools.
fields = block([field.replace("pub(super)", "pub") for field in style_fields])
options = "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct LeoSnapOptions {\n" + fields + "}\n\n"
options += "impl Default for LeoSnapOptions {\n    fn default() -> Self {\n        Self {\n" + block(style_defaults) + "        }\n    }\n}\n\n"
edit(guides, "pub const MAGENTA:", options + "pub const MAGENTA:")
edit(guides, "    pub color: [u8; 3],", "    pub(super) leo: LeoSnapOptions,\n    pub color: [u8; 3],")
edit(guides, "            color: MAGENTA,", "            leo: LeoSnapOptions::default(),\n            color: MAGENTA,")
edit(guides, "            color: cx.smart_guide_color,", "            leo: cx.leo_snap_options,\n            color: cx.smart_guide_color,")
edit(guides, '            "path" => self.path_color,', '''            "path" => self.path_color,
            "midpoint" => self.leo.leo_midpoint_color,
            "text outline" => self.leo.leo_text_color,
            "intersect" => self.leo.leo_intersection_color,
            "bounds" => self.leo.leo_bounds_color,
            "artboard" => self.leo.leo_artboard_color,
            "bleed" => self.leo.leo_bleed_color,
            "guide" => self.leo.leo_ruler_color,
            "tangent" => self.leo.leo_tangent_color,
            "perpendicular" => self.leo.leo_perpendicular_color,''')
edit(guides, "    pub fn color_for_label", '''    pub(super) fn feedback_for(&self, kind: &str) -> bool {
        match kind {
            "anchor" => self.leo.leo_anchor_feedback,
            "midpoint" => self.leo.leo_midpoint_feedback,
            "text outline" => self.leo.leo_text_feedback,
            "center" => self.leo.leo_center_feedback,
            "path" => self.leo.leo_path_feedback,
            "intersect" => self.leo.leo_intersection_feedback,
            "bounds" => self.leo.leo_bounds_feedback,
            "artboard" => self.leo.leo_artboard_feedback,
            "bleed" => self.leo.leo_bleed_feedback,
            "guide" => self.leo.leo_ruler_feedback,
            "tangent" => self.leo.leo_tangent_feedback,
            "perpendicular" => self.leo.leo_perpendicular_feedback,
            _ => true,
        }
    }

    pub fn color_for_label''')

edit(targets, "    Anchor,\n", "    Anchor,\n    Midpoint,\n    TextAnchor,\n")
edit(targets, '            Kind::Anchor => "anchor",', '            Kind::Anchor => "anchor",\n            Kind::Midpoint => "midpoint",\n            Kind::TextAnchor => "text outline",')
edit(targets, "    pub(super) segments: Vec<(Rect, PathSeg)>,", "    pub(super) segments: Vec<(Rect, PathSeg)>,\n    pub(super) text_segments: Vec<(Rect, PathSeg)>,")
edit(targets, "t.points.push((seg.start(), Kind::Anchor));\n                    t.segments.push", "t.points.push((seg.start(), Kind::TextAnchor));\n                    t.text_segments.push")
edit(targets, "    pub fn styled(mut self,", """    fn contact_segments(&self) -> impl Iterator<Item = &(Rect, PathSeg)> {
        self.segments.iter().chain(self.text_segments.iter())
    }

    fn path_hint_kind(&self, point: Point) -> &'static str {
        if self.text_segments.iter().any(|(bounds, seg)| {
            bounds.inflate(1e-4, 1e-4).contains(point) && seg.nearest(point, 1e-6).distance_sq < 1e-8
        }) { "text outline" } else { Kind::Edge.label() }
    }

    pub fn styled(mut self,""")
edit(targets, "        self.style = GuideStyle::of(cx);\n        self", '''        self.style = GuideStyle::of(cx);
        let o = self.style.leo;
        let enabled = |kind| match kind {
            Kind::Anchor => o.leo_anchor_snap,
            Kind::Midpoint => o.leo_midpoint_snap,
            Kind::TextAnchor => o.leo_text_snap && o.leo_anchor_snap,
            Kind::Center => o.leo_center_snap,
            Kind::Edge => o.leo_bounds_snap,
            Kind::Artboard => o.leo_artboard_snap,
            Kind::Bleed => o.leo_bleed_snap,
            Kind::Guide => o.leo_ruler_snap,
            Kind::Glyph => true,
        };
        if !o.leo_text_snap { self.text_segments.clear(); }
        self.points.retain(|(_, k)| *k != Kind::Midpoint);
        if o.leo_midpoint_snap {
            let midpoints = self.contact_segments().map(|(_, seg)| (seg.eval(0.5), Kind::Midpoint)).collect::<Vec<_>>();
            self.points.extend(midpoints);
            self.points.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
        }
        self.points.retain(|(_, k)| enabled(*k));
        self.xs.retain(|(_, _, k)| o.leo_vertical_snap && enabled(*k));
        self.ys.retain(|(_, _, k)| o.leo_horizontal_snap && enabled(*k));
        if !o.leo_extension_snap { self.extensions.clear(); }
        self''')
# Preserve the style-only call sites while supplying target-specific colors for known hits.
edit(targets, "        let a = match self.compact_hint", '''        self.target_line(a, b, None)
    }

    fn target_line(&self, a: Point, b: Point, kind: Option<Kind>) -> Option<Overlay> {
        let a = match self.compact_hint''')
edit(targets, """        self.style.lines.then_some(Overlay::Line { a, b, color, dashed: false })""", """        let axis_feedback = if (a.y - b.y).abs() <= 1e-6 {
            self.style.leo.leo_horizontal_feedback
        } else if (a.x - b.x).abs() <= 1e-6 {
            self.style.leo.leo_vertical_feedback
        } else { true };
        let label = kind.map(|k| if k == Kind::Edge { "bounds" } else { k.label() });
        let defaults = super::LeoSnapOptions::default();
        let color = match kind {
            Some(Kind::Edge) if self.style.leo.leo_bounds_color != defaults.leo_bounds_color => self.style.leo.leo_bounds_color,
            Some(Kind::Artboard) => self.style.leo.leo_artboard_color,
            Some(Kind::Bleed) => self.style.leo.leo_bleed_color,
            Some(Kind::Guide) => self.style.leo.leo_ruler_color,
            _ => color,
        };
        (self.style.lines && axis_feedback && label.is_none_or(|k| self.style.feedback_for(k)))
            .then_some(Overlay::Line { a, b, color, dashed: false })""")
edit(targets, "self.style.labels.then(|| Overlay::Label", "(self.style.labels && self.style.feedback_for(text)).then(|| Overlay::Label")
edit(targets, "        if shown {\n            self.rulers", "        if shown && cx.leo_snap_options.leo_ruler_snap {\n            self.rulers")
edit(targets, "        self.anchors_only().with_rulers(cx)", "        self.anchors_only().styled(cx).with_rulers(cx)")
edit(targets, "        // A ruler guide running past,", "        if !self.style.leo.leo_intersection_snap { return None; }\n        // A ruler guide running past,")
edit(targets, "        self.segments\n            .iter()\n            .filter(|(b, _)| b.inflate(tol, tol).contains(p))", "        if !self.style.leo.leo_path_snap { return None; }\n        self.contact_segments()\n            .filter(|(b, _)| b.inflate(tol, tol).contains(p))")
edit(targets, "self.segments.iter().filter(|(b, _)| b.inflate(tol, tol).contains(p))", "self.contact_segments().filter(|(b, _)| b.inflate(tol, tol).contains(p))")
edit(targets, "self.segments.iter().filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q))", "self.contact_segments().filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q))")
edit(targets, "let near: Vec<PathSeg> = self\n            .segments\n            .iter()", "let near: Vec<PathSeg> = self.contact_segments()")
edit(targets, "        self.segments.clear();", "        self.segments.clear();\n        self.text_segments.clear();")
edit(targets, "filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q))", "filter(|(bounds, _)| self.style.leo.leo_intersection_snap && bounds.inflate(reach, reach).contains(q))")
edit(targets, "if self.style.lines && let Some((end, projected, dir))", "if self.style.lines && self.style.leo.leo_extension_feedback && let Some((end, projected, dir))")
# Keep the original preview cyan as the default. A custom extension swatch colors both states.
edit(targets, "color: EXTENSION_PREVIEW_COLOR,", "color: if self.style.leo.leo_extension_color == EXTENSION_SNAP_COLOR { EXTENSION_PREVIEW_COLOR } else { self.style.leo.leo_extension_color },")
edit(targets, "        let mut out: Vec<Overlay> = self.line(at, q).into_iter().collect();", """        let kind = if perpendicular { "perpendicular" } else { "tangent" };
        if !self.style.feedback_for(kind) { return vec![]; }
        let color = self.style.color_for_label(kind);
        let mut out: Vec<Overlay> = self.style.lines.then_some(Overlay::Line { a: at, b: q, color, dashed: false }).into_iter().collect();""")
edit(targets, "out.push(Overlay::Path { path: mark, color: self.style.construction_color,", "out.push(Overlay::Path { path: mark, color,")
edit(targets, 'out.extend(self.label(q, if perpendicular { "perpendicular" } else { "tangent" }));', "out.extend(self.label(q, kind));")
edit(targets, "gaps.into_iter().flat_map(|g| self.gap_marks(g, x, tol)).collect()", "gaps.into_iter().filter(|_| self.style.leo.leo_spacing_feedback).flat_map(|g| self.gap_marks(g, x, tol)).collect()")
edit(targets, "            if spaced {", "            if spaced && self.style.leo.leo_spacing_feedback {")
edit(resolver, "if self.style.lines {\n                ov.push", "if self.style.lines && self.style.leo.leo_extension_feedback {\n                ov.push")
edit(resolver, "            ov.push(Overlay::CurvePoint { p: hit, color: EXTENSION_SNAP_COLOR, filled: true, size: 8.0 });", """            if self.style.leo.leo_extension_feedback {
                ov.push(Overlay::CurvePoint { p: hit, color: self.style.leo.leo_extension_color, filled: true, size: 8.0 });
            }""")
edit(resolver, "color: EXTENSION_SNAP_COLOR", "color: self.style.leo.leo_extension_color", 2)
edit(resolver, "            if self.style.labels {", "            if self.style.labels && self.style.leo.leo_extension_feedback {")
edit(resolver, "                    let mut ov: Vec<Overlay> = self.line(at, hit).into_iter().collect();", "                    let mut ov: Vec<Overlay> = self.construction_hint(at, hit).into_iter().collect();")
edit(resolver, "                None => (q, self.line(at, q).into_iter().collect()),", "                None => (q, self.construction_hint(at, q).into_iter().collect()),")
edit(resolver, "if let Some((x, start, _))", "if let Some((x, start, kind))")
edit(resolver, "if let Some((y, start, _))", "if let Some((y, start, kind))")
edit(resolver, "self.line(start, Point::new(x, p.y))", "self.target_line(start, Point::new(x, p.y), Some(kind))")
edit(resolver, "self.line(start, Point::new(p.x, y))", "self.target_line(start, Point::new(p.x, y), Some(kind))")
edit(resolver, "self.contact_hint(q, Kind::Edge.label())", "self.contact_hint(q, self.path_hint_kind(q))")
edit(targets, "    fn label(&self,", """    fn construction_hint(&self, a: Point, b: Point) -> Option<Overlay> {
        (self.style.lines && self.style.leo.leo_construction_feedback)
            .then_some(Overlay::Line { a, b, color: self.style.construction_color, dashed: false })
    }

    fn label(&self,""")
edit(move, "self.compact_hint.is_some() && self.style.lines", "self.compact_hint.is_some() && self.style.lines && self.style.feedback_for(kind)")
edit(move, "if kind == Kind::Anchor && self.style.lines", 'if matches!(kind, Kind::Anchor | Kind::Midpoint | Kind::TextAnchor) && self.style.lines && self.style.feedback_for(kind.label())')
edit(move, "        let hit = if let Some(dir) = along {", "        let hit = if !self.style.leo.leo_path_snap {\n            None\n        } else if let Some(dir) = along {")
edit(move, "self.segments.iter()", "self.contact_segments()")
edit(move, "let kind = Kind::Edge.label();", "let kind = self.path_hint_kind(q);")
edit(move, "let mark = self.style.lines.then(|| self.contact_mark(q, kind));", "let mark = (self.style.lines && self.style.feedback_for(kind)).then(|| self.contact_mark(q, kind));")
edit(rotation, "const ROTATION_COLOR:", "#[cfg(test)]\nconst ROTATION_COLOR:")
edit(rotation, "        if !raw.is_finite()", "        if !self.style.leo.leo_rotation_snap\n            || !raw.is_finite()")
edit(rotation, "if kind != Kind::Anchor", "if !matches!(kind, Kind::Anchor | Kind::TextAnchor | Kind::Midpoint)")
edit(rotation, "self\n                    .segments\n                    .iter()", "self.contact_segments()")
edit(rotation, "        let mut overlays = vec![];", "        let mut overlays = vec![];\n        if !self.style.leo.leo_rotation_feedback { return Some((hit, overlays)); }")
edit(rotation, "color: ROTATION_COLOR,", "color: self.style.leo.leo_rotation_color,", 3)
edit(snap, "cx.snap_to_glyph && cx.smart_guides && cx.alignment_guides", "cx.snap_to_glyph && cx.smart_guides")
edit("tools/src/select.rs", "let text_ink = if guides &&", "let text_ink = if guides && cx.leo_snap_options.leo_text_snap &&")
edit("tools/src/select.rs", "            return Targets::hold_text_outline", "            if !cx.leo_snap_options.leo_text_snap { return None; }\n            return Targets::hold_text_outline")

# Keep the target type through move and resize, so their guides use the same row settings.
edit(targets, "map(move |(t, from, _)| (t - v, *from, v, r))", "map(move |(t, from, kind)| (t - v, *from, v, r, *kind))")
edit(targets, "Some((dx, from, x, r))", "Some((dx, from, x, r, kind))")
edit(targets, "Some((dy, from, y, r))", "Some((dy, from, y, r, kind))")
edit(targets, "self.line(Point::new(x, y0), Point::new(x, y1))", "self.target_line(Point::new(x, y0), Point::new(x, y1), Some(kind))")
edit(targets, "self.line(Point::new(x0, y), Point::new(x1, y))", "self.target_line(Point::new(x0, y), Point::new(x1, y), Some(kind))")
edit(targets, "Option<(f64, f64, Point, bool)>", "Option<(f64, f64, Point, bool, Kind)>")
edit(targets, "for (t, from, _) in lines_near(lines, v, tol)", "for (t, from, kind) in lines_near(lines, v, tol)")
edit(targets, "Some((s, *t, *from, x))", "Some((s, *t, *from, x, *kind))")
edit(targets, "Some((s, t, from, x))", "Some((s, t, from, x, kind))")
edit(targets, "self.line(Point::new(t, from.y.min(m.y0)), Point::new(t, from.y.max(m.y1)))", "self.target_line(Point::new(t, from.y.min(m.y0)), Point::new(t, from.y.max(m.y1)), Some(kind))")
edit(targets, "self.line(Point::new(from.x.min(m.x0), t), Point::new(from.x.max(m.x1), t))", "self.target_line(Point::new(from.x.min(m.x0), t), Point::new(from.x.max(m.x1), t), Some(kind))")
edit(targets, "type Hit = Option<(f64, Point)>;", "type Hit = Option<(f64, Point, Kind)>;")
edit(targets, "                .map(|(t, from, _)| (t, from))\n                .filter(|(t, _)|", "                .filter(|(t, _, _)|")
edit(targets, "(Some((q.x, q)), Some((q.y, q)), Some((q, k)))", "(Some((q.x, q, k)), Some((q.y, q, k)), Some((q, k)))")
edit(targets, "Some((x, from)) = hit_x", "Some((x, from, kind)) = hit_x")
edit(targets, "Some((y, from)) = hit_y", "Some((y, from, kind)) = hit_y")
edit(targets, "self.line(Point::new(x, from.y.min(nr.y0)), Point::new(x, from.y.max(nr.y1)))", "self.target_line(Point::new(x, from.y.min(nr.y0)), Point::new(x, from.y.max(nr.y1)), Some(kind))")
edit(targets, "self.line(Point::new(from.x.min(nr.x0), y), Point::new(from.x.max(nr.x1), y))", "self.target_line(Point::new(from.x.min(nr.x0), y), Point::new(from.x.max(nr.x1), y), Some(kind))")
edit(targets, "ov.extend(gaps.into_iter().flat_map(|g| self.gap_marks(g, is_x, tol)));", "ov.extend(gaps.into_iter().filter(|_| self.style.leo.leo_spacing_feedback).flat_map(|g| self.gap_marks(g, is_x, tol)));")
edit(targets, """    pub fn for_scale(cx: &ToolContext) -> Self {
        Self::for_move(cx, &cx.selection.objects)
    }""", """    pub fn for_scale(cx: &ToolContext) -> Self {
        let gather = if cx.leo_snap_options.leo_midpoint_snap { Gather::MoveContacts } else { Gather::Bounds };
        Self::move_targets(cx, &cx.selection.objects, gather)
    }""")
edit(targets, 'Some((t, from, k)) => (t, self.label(from, k.label()).into_iter().collect()),', 'Some((t, from, k)) => (t, self.label(from, if k == Kind::Edge { "bounds" } else { k.label() }).into_iter().collect()),')

with (root / resolver).open("a", encoding="utf-8", newline="\n") as file:
    file.write('''
#[cfg(test)]
mod leo_snapping_target_controls_tests {
    use super::*;
    use crate::testutil::{cx, doc_with_rect, paint};

    #[test]
    fn target_visibility_and_geometry_can_be_set_independently() {
        let (doc, _) = doc_with_rect();
        let selection = Selection::default();
        let paint = paint();
        let mut cx = cx(&doc, &selection, &paint);
        cx.leo_snap_options.leo_anchor_feedback = false;
        let anchor = Point::new(100.0, 100.0);
        let pointer = anchor + Vec2::new(1.0, 1.0);
        let targets = Targets::collect(&doc, &[], None).styled(&cx);
        assert_eq!(targets.snap_point(pointer, 4.0), (anchor, vec![]));
        cx.leo_snap_options.leo_anchor_snap = false;
        cx.leo_snap_options.leo_vertical_snap = false;
        cx.leo_snap_options.leo_horizontal_snap = false;
        let targets = Targets::collect(&doc, &[], None).styled(&cx);
        assert_eq!(targets.snap_point(pointer, 4.0).0, pointer);
    }

    #[test]
    fn optional_midpoint_uses_its_swatch_even_when_path_attraction_is_off() {
        let (doc, _) = doc_with_rect();
        let selection = Selection::default();
        let paint = paint();
        let mut cx = cx(&doc, &selection, &paint);
        cx.leo_snap_options.leo_midpoint_snap = true;
        cx.leo_snap_options.leo_path_snap = false;
        let midpoint = Point::new(150.0, 100.0);
        let targets = Targets::for_anchor_drag(&doc, &selection).styled(&cx);
        let (point, hints) = targets.snap_point(midpoint + Vec2::new(0.5, 0.5), 4.0);
        assert_eq!(point, midpoint);
        assert!(hints.iter().any(|hint| matches!(hint, Overlay::Label { text, color, .. }
            if text == "midpoint" && *color == cx.leo_snap_options.leo_midpoint_color)));
    }
}
''')

ui = 'const LEO_SNAP_ROWS: &[(&str, &str, &str, &str)] = &[\n' + block(ui_rows) + '];\n\n'
ui += '''fn leo_target_rows(ui: &mut egui::Ui, d: &mut Dialog) {
    ui.label("每行依次设置吸附、辅助线颜色和提示显示");
    for &(key, label, color, feedback) in LEO_SNAP_ROWS {
        ui.horizontal(|ui| {
            bool_row(ui, d, key, label);
            let hex = d.fields.get(color).and_then(Value::as_str).unwrap_or("#ff3dfc");
            let packed = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0xff3dfc);
            let mut rgb = [(packed >> 16) as u8, (packed >> 8) as u8, packed as u8];
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                d.fields.insert(color.into(), json!(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])));
            }
            bool_row(ui, d, feedback, "显示提示");
        });
    }
}

'''
edit(dialog, "fn category_fields(", ui + "fn category_fields(")
edit(dialog, '        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);', '        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);\n        leo_target_rows(ui, d);')
edit(dialog, "    for sp in specs {\n", """    for sp in specs {
        if cat == "Leo Core Bridge 吸附"
            && LEO_SNAP_ROWS.iter().any(|&(key, _, color, feedback)| [key, color, feedback].contains(&sp.key))
        { continue; }
""")
print("PASS per-target geometry, colors and independent feedback integrated")

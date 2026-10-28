#!/usr/bin/env python3
"""Apply Leo Design Smart Select, Smart Guides and Group Anchor Move patches to VectorCraft v0.17.0.

Usage: python scripts/patch_smart_select.py vectorcraft/crates/engine/src/tooling.rs
The script fails closed if upstream source no longer matches the expected anchors.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python scripts/patch_smart_select.py PATH_TO_vectorcraft/crates/engine/src/tooling.rs")

tooling_path = Path(sys.argv[1])
tooling = tooling_path.read_text(encoding="utf-8")
tooling_marker = '        // Leo Design: Ctrl-click in Selection borrows Group Selection.'

if tooling_marker in tooling:
    raise SystemExit("Leo Design tooling patch is already applied; refusing to apply twice.")

tooling_anchor = '        if id == "directSelection" && mods.cmd && mods.alt {'
if tooling.count(tooling_anchor) != 1:
    raise SystemExit("Expected VectorCraft v0.17.0 tooling.rs layout not found; refusing to patch.")

tooling_change = (
    tooling_marker + '\n'
    '        // Ctrl+Alt retains the upstream Select Behind behaviour.\n'
    '        if id == "selection" && mods.cmd && !mods.alt {\n'
    '            return Some("groupSelection");\n'
    '        }\n'
)
tooling = tooling.replace(tooling_anchor, tooling_change + tooling_anchor, 1)
tooling_path.write_text(tooling, encoding="utf-8", newline="\n")

# After the temporary Group Selection click, keep that child as the black-arrow target.
# Otherwise SelectionTool resolves every nested hit to the group's top object on the next drag.
root = tooling_path.parents[3]
select_path = root / "crates/tools/src/select.rs"
select = select_path.read_text(encoding="utf-8")
select_marker = '                        // Leo Design: keep a selected group child as the black-arrow target.'
if select_marker in select:
    raise SystemExit("Leo Design select.rs patch is already applied; refusing to apply twice.")

select_anchor = '''                    Some(h) => {
                        let top = h.top_object(cx.isolation);
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
if select.count(select_anchor) != 1:
    raise SystemExit("Expected VectorCraft v0.17.0 select.rs hit-test block not found; refusing to patch.")

select_change = '''                    Some(h) => {
                        let top = h.top_object(cx.isolation);
                        // Leo Design: keep a selected group child as the black-arrow target.
                        // A normal click on another child still selects the whole group as usual.
                        let target = if h.leaf != top && cx.selection.contains(h.leaf) { h.leaf } else { top };
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
select = select.replace(select_anchor, select_change, 1)

# Scope replacements to the object-selection block only.
start = select.index(select_change)
end_anchor = '''                    }
                    None => {
                        self.state = State::Marquee'''
end = select.index(end_anchor, start)
block = select[start:end]
block = block.replace('cx.selection.contains(top)', 'cx.selection.contains(target)')
block = block.replace('deselect = Some(top)', 'deselect = Some(target)')
block = block.replace('"id": top.0', '"id": target.0')
block = block.replace('"ids": [top.0]', '"ids": [target.0]')
block = block.replace('key = Some(top)', 'key = Some(target)')
select = select[:start] + block + select[end:]
select_path.write_text(select, encoding="utf-8", newline="\n")

# --- Leo Design Smart Guides multi-colour preferences ---
# The upstream Preferences dialog already renders and saves every PREF_SPECS colour field.
def replace_once(path, old, new, label):
    content = path.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"Expected exactly one {label} anchor in {path}, found {count}; refusing to patch.")
    path.write_text(content.replace(old, new, 1), encoding="utf-8", newline="\n")

engine_lib = root / "crates/engine/src/lib.rs"
replace_once(
    engine_lib,
    '    pub smart_guide_color: String,\n    pub alignment_guides: bool,',
    '''    pub smart_guide_color: String,
    pub smart_guide_horizontal_color: String,
    pub smart_guide_vertical_color: String,
    pub smart_guide_spacing_color: String,
    pub smart_guide_construction_color: String,
    pub smart_guide_distance_color: String,
    pub smart_guide_distance_label_color: String,
    pub smart_guide_label_color: String,
    pub smart_guide_anchor_color: String,
    pub smart_guide_center_color: String,
    pub smart_guide_path_color: String,
    pub alignment_guides: bool,''',
    "Prefs Smart Guides colour fields",
)
replace_once(
    engine_lib,
    '            smart_guide_color: s("#ff3dfc"),\n            alignment_guides: true,',
    '''            smart_guide_color: s("#ff3dfc"),
            smart_guide_horizontal_color: s("#ff3dfc"),
            smart_guide_vertical_color: s("#ff3dfc"),
            smart_guide_spacing_color: s("#ff3dfc"),
            smart_guide_construction_color: s("#ff3dfc"),
            smart_guide_distance_color: s("#ff3dfc"),
            smart_guide_distance_label_color: s("#ff3dfc"),
            smart_guide_label_color: s("#ff3dfc"),
            smart_guide_anchor_color: s("#ff3dfc"),
            smart_guide_center_color: s("#ff3dfc"),
            smart_guide_path_color: s("#ff3dfc"),
            alignment_guides: true,''',
    "Prefs Smart Guides default colours",
)

prefs_cmds = root / "crates/engine/src/cmd/prefscmds.rs"
replace_once(
    prefs_cmds,
    '    p!("smartGuideColor", "Smart Guides", "Display Options", "Color", color),',
    '''    p!("smartGuideColor", "Smart Guides", "Display Options", "Default / Other Guides Color", color),
    p!("smartGuideHorizontalColor", "Smart Guides", "Display Options", "Horizontal Alignment Line", color),
    p!("smartGuideVerticalColor", "Smart Guides", "Display Options", "Vertical Alignment Line", color),
    p!("smartGuideSpacingColor", "Smart Guides", "Display Options", "Spacing Guide Lines", color),
    p!("smartGuideConstructionColor", "Smart Guides", "Display Options", "Construction / Angle Guides", color),
    p!("smartGuideDistanceColor", "Smart Guides", "Display Options", "Distance Measurement Line", color),
    p!("smartGuideDistanceLabelColor", "Smart Guides", "Display Options", "Distance Measurement Label", color),
    p!("smartGuideLabelColor", "Smart Guides", "Display Options", "General Hint Labels", color),
    p!("smartGuideAnchorColor", "Smart Guides", "Display Options", "Anchor Hint", color),
    p!("smartGuideCenterColor", "Smart Guides", "Display Options", "Center Hint", color),
    p!("smartGuidePathColor", "Smart Guides", "Display Options", "Path Hint", color),''',
    "PREF_SPECS Smart Guides colour controls",
)

tools_lib = root / "crates/tools/src/lib.rs"
replace_once(
    tools_lib,
    '    pub smart_guide_color: [u8; 3],\n    /// Smart Guides → Alignment Guides:',
    '''    pub smart_guide_color: [u8; 3],
    pub smart_guide_horizontal_color: [u8; 3],
    pub smart_guide_vertical_color: [u8; 3],
    pub smart_guide_spacing_color: [u8; 3],
    pub smart_guide_construction_color: [u8; 3],
    pub smart_guide_distance_color: [u8; 3],
    pub smart_guide_distance_label_color: [u8; 3],
    pub smart_guide_label_color: [u8; 3],
    pub smart_guide_anchor_color: [u8; 3],
    pub smart_guide_center_color: [u8; 3],
    pub smart_guide_path_color: [u8; 3],
    /// Smart Guides → Alignment Guides:''',
    "ToolContext Smart Guides colour fields",
)
replace_once(
    tools_lib,
    '            smart_guide_color: guides::MAGENTA,\n            alignment_guides: true,',
    '''            smart_guide_color: guides::MAGENTA,
            smart_guide_horizontal_color: guides::MAGENTA,
            smart_guide_vertical_color: guides::MAGENTA,
            smart_guide_spacing_color: guides::MAGENTA,
            smart_guide_construction_color: guides::MAGENTA,
            smart_guide_distance_color: guides::MAGENTA,
            smart_guide_distance_label_color: guides::MAGENTA,
            smart_guide_label_color: guides::MAGENTA,
            smart_guide_anchor_color: guides::MAGENTA,
            smart_guide_center_color: guides::MAGENTA,
            smart_guide_path_color: guides::MAGENTA,
            alignment_guides: true,''',
    "ToolContext test-context default colours",
)

replace_once(
    tooling_path,
    '            smart_guide_color: rgb(&self.prefs.smart_guide_color, vectorcraft_tools::guides::MAGENTA),\n            alignment_guides: self.prefs.alignment_guides,',
    '''            smart_guide_color: rgb(&self.prefs.smart_guide_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_horizontal_color: rgb(&self.prefs.smart_guide_horizontal_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_vertical_color: rgb(&self.prefs.smart_guide_vertical_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_spacing_color: rgb(&self.prefs.smart_guide_spacing_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_construction_color: rgb(&self.prefs.smart_guide_construction_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_distance_color: rgb(&self.prefs.smart_guide_distance_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_distance_label_color: rgb(&self.prefs.smart_guide_distance_label_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_label_color: rgb(&self.prefs.smart_guide_label_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_anchor_color: rgb(&self.prefs.smart_guide_anchor_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_center_color: rgb(&self.prefs.smart_guide_center_color, vectorcraft_tools::guides::MAGENTA),
            smart_guide_path_color: rgb(&self.prefs.smart_guide_path_color, vectorcraft_tools::guides::MAGENTA),
            alignment_guides: self.prefs.alignment_guides,''',
    "engine ToolContext Smart Guides colour wiring",
)

guides = root / "crates/tools/src/guides.rs"
replace_once(
    guides,
    '    pub color: [u8; 3],\n    /// Alignment Guides:',
    '''    pub color: [u8; 3],
    pub horizontal_color: [u8; 3],
    pub vertical_color: [u8; 3],
    pub spacing_color: [u8; 3],
    pub construction_color: [u8; 3],
    pub label_color: [u8; 3],
    pub anchor_color: [u8; 3],
    pub center_color: [u8; 3],
    pub path_color: [u8; 3],
    /// Alignment Guides:''',
    "GuideStyle colour fields",
)
replace_once(
    guides,
    '        Self { color: MAGENTA, lines: true, labels: true }',
    '''        Self {
            color: MAGENTA,
            horizontal_color: MAGENTA,
            vertical_color: MAGENTA,
            spacing_color: MAGENTA,
            construction_color: MAGENTA,
            label_color: MAGENTA,
            anchor_color: MAGENTA,
            center_color: MAGENTA,
            path_color: MAGENTA,
            lines: true,
            labels: true,
        }''',
    "GuideStyle default colours",
)
replace_once(
    guides,
    '        Self { color: cx.smart_guide_color, lines: cx.alignment_guides, labels: cx.anchor_path_labels }',
    '''        Self {
            color: cx.smart_guide_color,
            horizontal_color: cx.smart_guide_horizontal_color,
            vertical_color: cx.smart_guide_vertical_color,
            spacing_color: cx.smart_guide_spacing_color,
            construction_color: cx.smart_guide_construction_color,
            label_color: cx.smart_guide_label_color,
            anchor_color: cx.smart_guide_anchor_color,
            center_color: cx.smart_guide_center_color,
            path_color: cx.smart_guide_path_color,
            lines: cx.alignment_guides,
            labels: cx.anchor_path_labels,
        }''',
    "GuideStyle preference wiring",
)
replace_once(
    guides,
    '''    fn line(&self, a: Point, b: Point) -> Option<Overlay> {
        self.style.lines.then_some(Overlay::Line { a, b, color: self.style.color, dashed: false })
    }''',
    '''    fn line(&self, a: Point, b: Point) -> Option<Overlay> {
        let color = if (a.y - b.y).abs() <= 1e-6 {
            self.style.horizontal_color
        } else if (a.x - b.x).abs() <= 1e-6 {
            self.style.vertical_color
        } else {
            self.style.color
        };
        self.style.lines.then_some(Overlay::Line { a, b, color, dashed: false })
    }''',
    "axis-specific alignment line colour",
)
replace_once(
    guides,
    '''    fn label(&self, p: Point, text: &str) -> Option<Overlay> {
        self.style.labels.then(|| Overlay::Label { p, text: text.into(), color: self.style.color })
    }''',
    '''    fn label(&self, p: Point, text: &str) -> Option<Overlay> {
        self.style.labels.then(|| Overlay::Label { p, text: text.into(), color: self.style.color_for_label(text) })
    }''',
    "Smart Guides label colour selection",
)
replace_once(
    guides,
    'impl GuideStyle {\n',
    '''impl GuideStyle {
    /// Colour for the point/path label shown by a Smart Guide.
    pub fn color_for_label(&self, text: &str) -> [u8; 3] {
        match text {
            "anchor" => self.anchor_color,
            "center" => self.center_color,
            "path" => self.path_color,
            _ => self.label_color,
        }
    }

''',
    "GuideStyle label colour helper",
)
replace_once(
    guides,
    'let construction = |q: Point| Overlay::Line { a: at, b: q, color: self.style.color, dashed: false };',
    'let construction = |q: Point| Overlay::Line { a: at, b: q, color: self.style.construction_color, dashed: false };',
    "construction guide colour",
)
replace_once(
    guides,
    'out.push(Overlay::Path { path: mark, color: self.style.color, width: 1.0, dashed: false });',
    'out.push(Overlay::Path { path: mark, color: self.style.construction_color, width: 1.0, dashed: false });',
    "perpendicular construction mark colour",
)
replace_once(
    guides,
    'let line = |a: Point, b: Point| Overlay::Line { a, b, color: self.style.color, dashed: false };',
    'let line = |a: Point, b: Point| Overlay::Line { a, b, color: self.style.spacing_color, dashed: false };',
    "spacing guide colour",
)
replace_once(
    guides,
    '    let color = cx.smart_guide_color;\n    let mut out = Vec::new();',
    '    let color = cx.smart_guide_distance_color;\n    let label_color = cx.smart_guide_distance_label_color;\n    let mut out = Vec::new();',
    "distance guide line and label colours",
)
replace_once(
    guides,
    'out.push(Overlay::Distance { a: pt(fu, across), b: pt(tu, across), text: cx.len(length), color });',
    'out.push(Overlay::Distance { a: pt(fu, across), b: pt(tu, across), text: cx.len(length), color, label_color });',
    "distance overlay label colour",
)
replace_once(
    guides,
    'let c = ToolContext { smart_guide_color: green, ..cx(&d, &s, &p) };',
    '''let c = ToolContext {
            smart_guide_color: green,
            smart_guide_horizontal_color: green,
            smart_guide_vertical_color: green,
            smart_guide_spacing_color: green,
            smart_guide_construction_color: green,
            smart_guide_distance_color: green,
            smart_guide_distance_label_color: green,
            smart_guide_label_color: green,
            smart_guide_anchor_color: green,
            smart_guide_center_color: green,
            smart_guide_path_color: green,
            ..cx(&d, &s, &p)
        };''',
    "Smart Guides colour unit test setup",
)

tools_lib = root / "crates/tools/src/lib.rs"
replace_once(
    tools_lib,
    '    Distance { a: Point, b: Point, text: String, color: [u8; 3] },',
    '    Distance { a: Point, b: Point, text: String, color: [u8; 3], label_color: [u8; 3] },',
    "Distance overlay separate label colour",
)


# Update the distance overlay comment without relying on literal backticks in this patch script.
distance_doc = '    /// ' + chr(96) + 'text' + chr(96) + ' (its length) in a pill of ' + chr(96) + 'color' + chr(96) + ' on its middle.\n    Distance { a: Point, b: Point, text: String, color: [u8; 3], label_color: [u8; 3] },'
distance_doc_new = '    /// ' + chr(96) + 'text' + chr(96) + ' (its length) in a pill of ' + chr(96) + 'label_color' + chr(96) + ' on its middle.\n    Distance { a: Point, b: Point, text: String, color: [u8; 3], label_color: [u8; 3] },'
replace_once(tools_lib, distance_doc, distance_doc_new, "Distance overlay documentation")

canvas = root / "crates/ui-egui/src/canvas.rs"
replace_once(
    canvas,
    '            Overlay::Distance { a, b, text, color } => distance_guide(p, xf.to_screen(*a), xf.to_screen(*b), text, c32(*color)),',
    '            Overlay::Distance { a, b, text, color, label_color } => distance_guide(p, xf.to_screen(*a), xf.to_screen(*b), text, c32(*color), c32(*label_color)),',
    "Distance overlay rendering wiring",
)
replace_once(
    canvas,
    '/// on its middle, in white or black, whichever reads on it.',
    '/// on its middle. The label pill uses its own colour and contrast-aware text.',
    "Distance guide documentation",
)
replace_once(
    canvas,
    'fn distance_guide(p: &egui::Painter, a: Pos2, b: Pos2, text: &str, color: Color32) {',
    'fn distance_guide(p: &egui::Painter, a: Pos2, b: Pos2, text: &str, color: Color32, label_color: Color32) {',
    "Distance guide renderer signature",
)
replace_once(
    canvas,
    '''    let luma = 0.299 * f32::from(color.r()) + 0.587 * f32::from(color.g()) + 0.114 * f32::from(color.b());
    let ink = if luma > 150.0 { Color32::BLACK } else { Color32::WHITE };
    let galley = p.layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.0), ink);
    let r = egui::Rect::from_center_size(a + along / 2.0, galley.size() + vec2(8.0, 4.0));
    p.rect_filled(r, CornerRadius::same(3), color);''',
    '''    let luma = 0.299 * f32::from(label_color.r()) + 0.587 * f32::from(label_color.g()) + 0.114 * f32::from(label_color.b());
    let ink = if luma > 150.0 { Color32::BLACK } else { Color32::WHITE };
    let galley = p.layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.0), ink);
    let r = egui::Rect::from_center_size(a + along / 2.0, galley.size() + vec2(8.0, 4.0));
    p.rect_filled(r, CornerRadius::same(3), label_color);''',
    "Distance guide label pill colour",
)

select_path = root / "crates/tools/src/select.rs"
replace_once(
    select_path,
    'o.push(Overlay::Label { p: q, text: label.into(), color: style.color });',
    'o.push(Overlay::Label { p: q, text: label.into(), color: style.color_for_label(label) });',
    "selection hover label colour",
)

prefs_dialog = root / "crates/ui-egui/src/prefs_dialog.rs"
replace_once(
    prefs_dialog,
    'fn category_fields(ui: &mut egui::Ui, d: &mut Dialog, cat: &str) {',
    '''fn apply_smart_guide_palette(d: &mut Dialog, colors: &[(&str, &str)]) {
    for (key, value) in colors {
        d.fields.insert((*key).into(), json!(*value));
    }
}

fn category_fields(ui: &mut egui::Ui, d: &mut Dialog, cat: &str) {''',
    "Smart Guides palette helper",
)
replace_once(
    prefs_dialog,
    '''    if cat == "Smart Guides" {
        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);
        ui.add_space(4.0);
    }''',
    '''    if cat == "Smart Guides" {
        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Colour Presets").color(t.text_strong).size(11.5));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Classic Magenta").clicked() {
                apply_smart_guide_palette(d, &[
                    ("smartGuideColor", "#ff3dfc"),
                    ("smartGuideHorizontalColor", "#ff3dfc"),
                    ("smartGuideVerticalColor", "#ff3dfc"),
                    ("smartGuideSpacingColor", "#ff3dfc"),
                    ("smartGuideConstructionColor", "#ff3dfc"),
                    ("smartGuideDistanceColor", "#ff3dfc"),
                    ("smartGuideDistanceLabelColor", "#ff3dfc"),
                    ("smartGuideLabelColor", "#ff3dfc"),
                    ("smartGuideAnchorColor", "#ff3dfc"),
                    ("smartGuideCenterColor", "#ff3dfc"),
                    ("smartGuidePathColor", "#ff3dfc"),
                ]);
            }
            if ui.button("High Contrast").clicked() {
                apply_smart_guide_palette(d, &[
                    ("smartGuideColor", "#ff3dfc"),
                    ("smartGuideHorizontalColor", "#ff4d4f"),
                    ("smartGuideVerticalColor", "#00d4ff"),
                    ("smartGuideSpacingColor", "#ffd166"),
                    ("smartGuideConstructionColor", "#49e59a"),
                    ("smartGuideDistanceColor", "#a78bfa"),
                    ("smartGuideDistanceLabelColor", "#ffffff"),
                    ("smartGuideLabelColor", "#ffffff"),
                    ("smartGuideAnchorColor", "#ff4d4f"),
                    ("smartGuideCenterColor", "#00d4ff"),
                    ("smartGuidePathColor", "#49e59a"),
                ]);
            }
            if ui.button("Cool Studio").clicked() {
                apply_smart_guide_palette(d, &[
                    ("smartGuideColor", "#ff3dfc"),
                    ("smartGuideHorizontalColor", "#ff5bd6"),
                    ("smartGuideVerticalColor", "#38d9ff"),
                    ("smartGuideSpacingColor", "#ffd166"),
                    ("smartGuideConstructionColor", "#7ae582"),
                    ("smartGuideDistanceColor", "#b197fc"),
                    ("smartGuideDistanceLabelColor", "#38d9ff"),
                    ("smartGuideLabelColor", "#38d9ff"),
                    ("smartGuideAnchorColor", "#ff5bd6"),
                    ("smartGuideCenterColor", "#38d9ff"),
                    ("smartGuidePathColor", "#7ae582"),
                ]);
            }
        });
        ui.label(egui::RichText::new("Choose a preset, then fine-tune each colour below. Confirm with OK to save.").color(t.text_dim).size(11.0));
        ui.add_space(4.0);
    }''',
    "Smart Guides preset controls",
)

# --- Leo Design Group Anchor Move: rigidly translate a selected group by a child anchor ---
direct_path = root / "crates/tools/src/direct.rs"
replace_once(
    direct_path,
    '    /// Dragging a curved segment: it follows the pointer',
    '''    /// Ctrl-dragging a descendant anchor moves its selected group as a rigid whole.
    MoveGroupAnchor {
        start: Point,
        grab: Point,
        began: bool,
    },
    /// Dragging a curved segment: it follows the pointer''',
    "Group Anchor Move state",
)

replace_once(
    direct_path,
    '''                if let Some((id, si, ai, grab)) = hit_anchor(cx, p, point) {
                    if cx.doc.node(id).is_some_and(is_area_type) {''',
    '''                if let Some((id, si, ai, grab)) = hit_anchor(cx, p, point) {
                    // Leo Design: Ctrl-drag a child anchor to move the selected group rigidly.
                    // The grabbed child anchor is the snap reference; only the selected parent group
                    // is transformed, so no path or internal spacing is reshaped.
                    if ev.mods.cmd
                        && !ev.mods.shift
                        && !ev.mods.alt
                        && cx.selection.objects.len() == 1
                        && let Some(selected) = cx.selection.objects.first().copied()
                        && selected != id
                        && cx.doc.node(selected).is_some_and(|n| n.children().is_some())
                        && cx.doc.ancestry(id).is_some_and(|anc| anc.contains(&selected))
                    {
                        self.state = State::MoveGroupAnchor { start: p, grab, began: false };
                        return vec![];
                    }
                    if cx.doc.node(id).is_some_and(is_area_type) {''',
    "Group Anchor Move gesture",
)

replace_once(
    direct_path,
    '''            (PointerKind::Drag, State::MoveObject { start, grab, hold, began, .. }) => {
                let mut out = vec![];''',
    '''            (PointerKind::Drag, State::MoveGroupAnchor { start, grab, began }) => {
                let mut out = vec![];
                if !began {
                    if p.distance(start) < cx.tol(3.0) {
                        return out;
                    }
                    out.push(Action::Begin("Move Group by Anchor".into()));
                    self.state = State::MoveGroupAnchor { start, grab, began: true };
                    self.anchor_snap = Some(PointSnap::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection)));
                }
                // Ctrl activates this gesture. Normal Shift-constrained editing remains unchanged.
                let mut d = move_delta(start, p, false, cx.constrain_angle);
                let unsnapped = d;
                self.guides.clear();
                if let Some(snap) = &self.anchor_snap {
                    let (q, guides) = snap.snap_from(cx, grab + (p - start), None);
                    (d, self.guides) = (q - grab, guides);
                }
                self.snapped = snapped(d, unsnapped, false, &self.guides);
                out.push(Action::Preview(
                    "object.transform".into(),
                    json!({"matrix": matrix_json(vectorcraft_geom::Affine::translate(d)), "copy": false}),
                ));
                out
            }
            (PointerKind::Drag, State::MoveObject { start, grab, hold, began, .. }) => {
                let mut out = vec![];''',
    "Group Anchor Move drag",
)

replace_once(
    direct_path,
    '''                | State::MoveObject { began, .. }
                | State::Segment { began, .. }''',
    '''                | State::MoveObject { began, .. }
                | State::MoveGroupAnchor { began, .. }
                | State::Segment { began, .. }''',
    "Group Anchor Move release",
)

replace_once(
    direct_path,
    '''                if matches!(self.state, State::Handle { .. } | State::MoveAnchors { .. } | State::MoveObject { .. }) {''',
    '''                if matches!(self.state, State::Handle { .. } | State::MoveAnchors { .. } | State::MoveObject { .. } | State::MoveGroupAnchor { .. }) {''',
    "Group Anchor Move smart guides",
)

replace_once(
    direct_path,
    '''                | State::MoveAnchors { began: true, .. }
                | State::MoveObject { began: true, .. } => {''',
    '''                | State::MoveAnchors { began: true, .. }
                | State::MoveObject { began: true, .. }
                | State::MoveGroupAnchor { began: true, .. } => {''',
    "Group Anchor Move cursor",
)

print("SUCCESS: Smart Select, customizable Smart Guides, and rigid Group Anchor Move patches applied.")


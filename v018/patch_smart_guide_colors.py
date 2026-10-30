#!/usr/bin/env python3
"""Restore Leo Design v0.5.7 multicolour Smart Guides on official v0.18.0.

The native v0.18 snapping engine remains responsible for target finding.
This adapts its UI, preference fields, and per-guide overlay colors to the
split source modules (prefs/context/targets/canvas/overlays).
"""
from pathlib import Path
import sys
if len(sys.argv)!=2: raise SystemExit("Usage: patch_smart_guide_colors.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])
def rep(rel,old,new,label):
    f=root/rel
    s=f.read_text(encoding="utf-8")
    if s.count(old)!=1:raise SystemExit(f"FAIL {label}: {rel}, anchors={s.count(old)}")
    f.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
    print("OK",label)

# Complete compatibility with the v0.5.7 preference model. 10 separate
# channels, plus the original smartGuideColor fallback.
channels=[
 ("Horizontal","horizontal","Horizontal Alignment Line"),
 ("Vertical","vertical","Vertical Alignment Line"),
 ("Spacing","spacing","Spacing Guide Lines"),
 ("Construction","construction","Construction / Angle Guides"),
 ("Distance","distance","Distance Measurement Line"),
 ("DistanceLabel","distance_label","Distance Measurement Label"),
 ("Label","label","General Hint Labels"),
 ("Anchor","anchor","Anchor Hint"),
 ("Center","center","Center Hint"),
 ("Path","path","Path Hint"),
]
fields="\n".join(f"    pub smart_guide_{n}_color: String," for _,n,_ in channels)
defaults="\n".join(f'            smart_guide_{n}_color: s("#ff3dfc"),' for _,n,_ in channels)
specs="\n".join(f'    p!("smartGuide{camel}Color", "Smart Guides", "Display Options", "{label}", color),' for camel,_,label in channels)
context="\n".join(f"    pub smart_guide_{n}_color: [u8; 3]," for _,n,_ in channels)
test_ctx="\n".join(f"            smart_guide_{n}_color: guides::MAGENTA," for _,n,_ in channels)
tool_ctx="\n".join(f"            smart_guide_{n}_color: rgb(&self.prefs.smart_guide_{n}_color, vectorcraft_tools::guides::MAGENTA)," for _,n,_ in channels)
style_channels=[("Horizontal","horizontal"),("Vertical","vertical"),("Spacing","spacing"),("Construction","construction"),("Label","label"),("Anchor","anchor"),("Center","center"),("Path","path")]
style_fields="\n".join(f"    pub {n}_color: [u8; 3]," for _,n in style_channels)
style_defaults="\n".join(f"            {n}_color: MAGENTA," for _,n in style_channels)
style_of="\n".join(f"            {n}_color: cx.smart_guide_{n}_color," for _,n in style_channels)

rep("crates/engine/src/prefs.rs",
 '    pub smart_guide_color: String,\n    pub alignment_guides: bool,',
 '    pub smart_guide_color: String,\n'+fields+'\n    pub alignment_guides: bool,',
 "preferences remember guide colours"
)
rep("crates/engine/src/prefs.rs",
 '            smart_guide_color: s("#ff3dfc"),\n            alignment_guides: true,',
 '            smart_guide_color: s("#ff3dfc"),\n'+defaults+'\n            alignment_guides: true,',
 "Smart Guide fallback defaults"
)
rep("crates/engine/src/cmd/prefscmds.rs",
 '    p!("smartGuideColor", "Smart Guides", "Display Options", "Color", color),',
 '    p!("smartGuideColor", "Smart Guides", "Display Options", "Default / Other Guides Color", color),\n'+specs,
 "register all colour pickers"
)
rep("crates/tools/src/context.rs",
 '    pub smart_guide_color: [u8; 3],\n    /// Smart Guides → Alignment Guides:',
 '    pub smart_guide_color: [u8; 3],\n'+context+'\n    /// Smart Guides → Alignment Guides:',
 "tool context colour fields"
)
rep("crates/tools/src/lib.rs",
 '            smart_guide_color: guides::MAGENTA,\n            alignment_guides: true,',
 '            smart_guide_color: guides::MAGENTA,\n'+test_ctx+'\n            alignment_guides: true,',
 "tool test-context colour defaults"
)
rep("crates/engine/src/tooling.rs",
 '            smart_guide_color: rgb(&self.prefs.smart_guide_color, vectorcraft_tools::guides::MAGENTA),\n            alignment_guides: self.prefs.alignment_guides,',
 '            smart_guide_color: rgb(&self.prefs.smart_guide_color, vectorcraft_tools::guides::MAGENTA),\n'+tool_ctx+'\n            alignment_guides: self.prefs.alignment_guides,',
 "engine propagates guide colour preferences"
)
rep("crates/tools/src/guides.rs",
 '    pub color: [u8; 3],\n    /// Alignment Guides:',
 '    pub color: [u8; 3],\n'+style_fields+'\n    /// Alignment Guides:',
 "per-guide display style fields"
)
rep("crates/tools/src/guides.rs",
 '        Self { color: MAGENTA, lines: true, labels: true }',
 '        Self {\n            color: MAGENTA,\n'+style_defaults+'\n            lines: true,\n            labels: true,\n        }',
 "guide style defaults"
)
rep("crates/tools/src/guides.rs",
 '        Self { color: cx.smart_guide_color, lines: cx.alignment_guides, labels: cx.anchor_path_labels }',
 '        Self {\n            color: cx.smart_guide_color,\n'+style_of+'\n            lines: cx.alignment_guides,\n            labels: cx.anchor_path_labels,\n        }',
 "style picks configured colours"
)
rep("crates/tools/src/guides.rs",
 "impl GuideStyle {\n",
 """impl GuideStyle {
    pub fn color_for_label(&self, kind: &str) -> [u8; 3] {
        match kind {
            "anchor" => self.anchor_color,
            "center" => self.center_color,
            "path" => self.path_color,
            _ => self.label_color,
        }
    }
""",
 "anchor, path, center hover labels"
)
target="crates/tools/src/guides/targets.rs"
rep(target,
 '''    fn line(&self, a: Point, b: Point) -> Option<Overlay> {
        self.style.lines.then_some(Overlay::Line { a, b, color: self.style.color, dashed: false })
    }''',
 '''    fn line(&self, a: Point, b: Point) -> Option<Overlay> {
        let color = if (a.y-b.y).abs() <= 1e-6 {
            self.style.horizontal_color
        } else if (a.x-b.x).abs() <= 1e-6 {
            self.style.vertical_color
        } else {
            self.style.color
        };
        self.style.lines.then_some(Overlay::Line { a, b, color, dashed: false })
    }''',
 "independent red/blue horizontal/vertical snap guides"
)
rep(target,
 'self.style.labels.then(|| Overlay::Label { p, text: text.into(), color: self.style.color })',
 'self.style.labels.then(|| Overlay::Label { p, text: text.into(), color: self.style.color_for_label(text) })',
 "hover guide label colours"
)
rep(target,
 'let construction = |q: Point| Overlay::Line { a: at, b: q, color: self.style.color, dashed: false };',
 'let construction = |q: Point| Overlay::Line { a: at, b: q, color: self.style.construction_color, dashed: false };',
 "construction-angle guides"
)
rep(target,
 'out.push(Overlay::Path { path: mark, color: self.style.color, width: 1.0, dashed: false });',
 'out.push(Overlay::Path { path: mark, color: self.style.construction_color, width: 1.0, dashed: false });',
 "angle perpendicular mark guides"
)
rep(target,
 'let line = |a: Point, b: Point| Overlay::Line { a, b, color: self.style.color, dashed: false };',
 'let line = |a: Point, b: Point| Overlay::Line { a, b, color: self.style.spacing_color, dashed: false };',
 "even-spacing hint lines"
)
rep("crates/tools/src/select.rs",
 'o.push(Overlay::Label { p: q, text: label.into(), color: style.color });',
 'o.push(Overlay::Label { p: q, text: label.into(), color: style.color_for_label(label) });',
 "hovered path anchor label hues"
)
rep("crates/tools/src/lib.rs",
 '    Distance { a: Point, b: Point, text: String, color: [u8; 3] },',
 '    Distance { a: Point, b: Point, text: String, color: [u8; 3], label_color: [u8; 3] },',
 "distance overlay line+label independent colour"
)
rep("crates/tools/src/guides/snap.rs",
 '    let color = cx.smart_guide_color;\n    let mut out = Vec::new();',
 '    let color = cx.smart_guide_distance_color;\n    let label_color = cx.smart_guide_distance_label_color;\n    let mut out = Vec::new();',
 "distance guide line palette"
)
rep("crates/tools/src/guides/snap.rs",
 'out.push(Overlay::Distance { a: pt(fu, across), b: pt(tu, across), text: cx.len(length), color });',
 'out.push(Overlay::Distance { a: pt(fu, across), b: pt(tu, across), text: cx.len(length), color, label_color });',
 "distance text pill palette"
)
canvas="crates/ui-egui/src/canvas/overlays.rs"
rep(canvas,
 'Overlay::Distance { a, b, text, color } => distance_guide(p, xf.to_screen(*a), xf.to_screen(*b), text, c32(*color)),',
 'Overlay::Distance { a, b, text, color, label_color } => distance_guide(p, xf.to_screen(*a), xf.to_screen(*b), text, c32(*color), c32(*label_color)),',
 "render separate distance label colour"
)
rep(canvas,
 'fn distance_guide(p: &egui::Painter, a: Pos2, b: Pos2, text: &str, color: Color32) {',
 'fn distance_guide(p: &egui::Painter, a: Pos2, b: Pos2, text: &str, color: Color32, label_color: Color32) {',
 "distance-guide renderer signature"
)
rep(canvas,
 'let luma = 0.299 * f32::from(color.r()) + 0.587 * f32::from(color.g()) + 0.114 * f32::from(color.b());',
 'let luma = 0.299 * f32::from(label_color.r()) + 0.587 * f32::from(label_color.g()) + 0.114 * f32::from(label_color.b());',
 "contrast-aware measurement label"
)
rep(canvas,
 'p.rect_filled(r, CornerRadius::same(3), color);',
 'p.rect_filled(r, CornerRadius::same(3), label_color);',
 "distance pill colour"
)

# Three familiar named presets + manual per-guide colour pickers in Preferences.
dialog="crates/ui-egui/src/prefs_dialog.rs"
palette={
 "经典洋红":["#ff3dfc"]*10,
 "高对比":["#ff4d4f","#00d4ff","#ffd166","#49e59a","#a78bfa","#ffffff","#ffffff","#ff4d4f","#00d4ff","#49e59a"],
 "冷色工作室":["#ff5bd6","#38d9ff","#ffd166","#7ae582","#b197fc","#38d9ff","#38d9ff","#ff5bd6","#38d9ff","#7ae582"],
}
text = 'fn leo_apply_guide_palette(d: &mut Dialog, colors: &[(&str,&str)]) {\n    for (name, color) in colors {\n        d.fields.insert((*name).into(), json!(*color));\n    }\n}\n\n'
rep(dialog,'fn category_fields(ui: &mut egui::Ui, d: &mut Dialog, cat: &str) {',text+'fn category_fields(ui: &mut egui::Ui, d: &mut Dialog, cat: &str) {',"smart guide palette helper")
buttons=""
for label, values in palette.items():
    pairs=[("smartGuideColor","#ff3dfc")]+[(f"smartGuide{camel}Color",value) for (camel,_,_),value in zip(channels,values)]
    kv=",\n                    ".join(f'("{k}", "{v}")' for k,v in pairs)
    buttons+=f'''            if ui.button("{label}").clicked() {{
                leo_apply_guide_palette(d, &[
                    {kv}
                ]);
            }}
'''
rep(dialog,
 '''    if cat == "Smart Guides" {
        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);
        ui.add_space(4.0);
    }''',
 '''    if cat == "Smart Guides" {
        bool_row(ui, d, "__smartGuides", UI_FIELDS[0].2);
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Leo Design · 智能参考线颜色预设").color(t.text_strong));
        ui.horizontal_wrapped(|ui| {
'''+buttons+'''        });
        ui.label(egui::RichText::new("也可以继续在下方分别调整各类参考线颜色。").color(t.text_dim).small());
        ui.add_space(4.0);
    }''',
 "3 Smart Guide palettes + individual colour controls"
)

print("SUCCESS Leo Design v0.18 multicolour Smart Guides restored")

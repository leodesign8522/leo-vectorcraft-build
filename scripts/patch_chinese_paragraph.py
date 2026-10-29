#!/usr/bin/env python3
"""Leo Design v0.5.2: paragraph text-frame auto-height and visible Chinese typesetting.

Targets VectorCraft v0.17.0. Uses existing tested text.areaOptions and
text.setFormat commands; does not change underlying text serialization or renderer.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_chinese_paragraph.py VECTORCRAFT_ROOT")
path = Path(sys.argv[1]) / "crates/ui-egui/src/panels/paragraph.rs"
source = path.read_text(encoding="utf-8")

def replace_once(old, new, label):
    global source
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"FAIL {label}: expected 1 upstream anchor, found {count}")
    source = source.replace(old, new, 1)

replace_once(
    '''    ui.add_space(4.0);
    let fw = ((ui.available_width() - 66.0) / 2.0).clamp(60.0, 100.0);''',
    '''    ui.add_space(6.0);
    // Leo Design: a click fits the area-type frame to the actual number of
    // composed text lines, preserving its width and top-left corner. This calls
    // VectorCraft's original undoable auto-height engine, not an approximation
    // based on font size or the bounding-box drag handles.
    let area_selected = app.session.active().is_some_and(|st| {
        st.selection.objects.iter().any(|id| {
            st.doc.node(*id).is_some_and(|node| {
                matches!(&node.kind, NodeKind::Text(t)
                    if matches!(&t.kind, vectorcraft_doc::TextKind::Area { .. }))
            })
        })
    });
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(area_selected, egui::Button::new("文字框贴合内容高度"))
            .on_hover_text("保持文字框宽度和顶部位置，自动收缩或扩展高度；后续修改文字也自动调整")
            .clicked()
        {
            if let Err(e) = app.run("text.areaOptions", json!({"fit": "autoHeight"})) {
                app.ui.status = e;
            }
        }
        if ui.add_enabled(area_selected, egui::Button::new("固定框高"))
            .on_hover_text("关闭自动高度，保留当前文字框尺寸，以后可以手动拖动边框")
            .clicked()
        {
            if let Err(e) = app.run("text.areaOptions", json!({"fit": "none"})) {
                app.ui.status = e;
            }
        }
    });
    ui.add_space(4.0);
    let fw = ((ui.available_width() - 66.0) / 2.0).clamp(60.0, 100.0);''',
    "Add one-click real auto-height and fixed-height controls",
)

replace_once(
    '''    // Japanese composition: how punctuation is spaced (JLREQ 3.1), with the East Asian options.
    if app.session.prefs.show_east_asian_options {
        ui.horizontal(|ui| {
            widgets::dim_label(ui, tl!("Mojikumi Set"));
            let sets = [tl!("None"), tl!("Line-end Punctuation Half Width")];''',
    '''    // Leo Design: always expose CJK typesetting options to Chinese graphic
    // designers; upstream hides these behind the Show East Asian Options pref.
    // These options already change real paragraph layout and remain serialized.
    // The existing implementation supports None and line-end half-width only;
    // we do not claim to implement Illustrator's full custom Mojikumi sets.
    {
        ui.separator();
        ui.label(egui::RichText::new("Leo Design · 中文排版").strong());
        ui.horizontal(|ui| {
            widgets::dim_label(ui, "标点挤压集");
            let sets = ["无", "行末标点半角"];''',
    "Always expose translated CJK Mojikumi controls",
)

replace_once(
    '''            widgets::dim_label(ui, tl!("Kinsoku Set"));
            let sets = [(tl!("None"), Kinsoku::None, "none"), (tl!("Hard"), Kinsoku::Hard, "hard"), (tl!("Soft"), Kinsoku::Soft, "soft")];''',
    '''            widgets::dim_label(ui, "避头尾集");
            let sets = [("无", Kinsoku::None, "none"), ("严格", Kinsoku::Hard, "hard"), ("宽松", Kinsoku::Soft, "soft")];''',
    "Translate Kinsoku None Hard Soft labels",
)

path.write_text(source, encoding="utf-8", newline="\n")
print("SUCCESS: Leo Design text frame Auto Size and Chinese Kinsoku/Mojikumi paragraph controls.")

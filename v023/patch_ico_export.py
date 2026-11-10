#!/usr/bin/env python3
"""Leo Design Image Pro Export: native Windows ICO multi-size output for VectorCraft v0.21.

Appends an 'ico' writable format and uses the native artboard render pipeline
to write real ICONDIR/ICONDIRENTRY files. The image export sidebar and dialog
share the same encoder, preserve transparency, naming and batch export.
Run after patch_image_export.py and patch_panel_usability.py.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2: raise SystemExit("Usage: patch_ico_export.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1]); source=Path(__file__).resolve().parent
def rep(path,old,new,label):
    f=root/path
    s=f.read_text(encoding="utf-8")
    n=s.count(old)
    if n!=1:raise SystemExit(f"FAIL {label}: {path} anchor count={n}")
    f.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
    print("OK",label)

rep("crates/engine/src/cmd/fileio.rs",
 'mod legacy_ai;\n',
 'mod legacy_ai;\nmod leo_ico;\n',
 "register multi-resolution ICO encoder")
target=root/"crates/engine/src/cmd/fileio/leo_ico.rs"
if target.exists(): raise SystemExit("FAIL duplicate ICO encoder module")
target.write_text((source/"leo_ico.rs").read_text(encoding="utf-8"),encoding="utf-8",newline="\n")
print("OK native ICO encoder module")

rep("crates/engine/src/cmd/fileio/formats.rs",
 '''    reader("jxl", "JPEG XL", &["jxl"], "image/jxl", true),
];''',
 '''    reader("jxl", "JPEG XL", &["jxl"], "image/jxl", true),
    // Leo Design: each .ico contains 1�C7 square PNG layers at Windows icon
    // sizes 16,24,32,48,64,128,256 with RGBA transparency.
    Format {
        id: "ico",
        label: "Windows Icon",
        extensions: &["ico"],
        mime: "image/vnd.microsoft.icon",
        read: false,
        write: true,
        raster: true,
        options: &[ARTBOARD, ARTBOARDS, RANGE, USE_ARTBOARDS, RECT, ANTI_ALIAS],
    },
];''',
 "register ICO extension in native file format registry")

rep("crates/engine/src/cmd/fileio/encode.rs",
 '''        "pdf" => {
            let (bytes, warnings) = super::pdf::encode(C, doc, p)?;
            return Ok(Encoded { warnings, ..Encoded::one(bytes) });
        }
        "png" | "jpg"''',
 '''        "pdf" => {
            let (bytes, warnings) = super::pdf::encode(C, doc, p)?;
            return Ok(Encoded { warnings, ..Encoded::one(bytes) });
        }
        "ico" => return super::leo_ico::encode(doc, p, use_artboards),
        "png" | "jpg"''',
 "route ICO to native renderer and multi-resolution encoder")

panel="crates/ui-egui/src/dialogs/leo_image_export.rs"
# The native sidebar/dialog have independent UI bodies but share params().
# Both dialog and sidebar defaults share the same JSON.
p = root / panel
raw = p.read_text(encoding="utf-8")
old = '"format": "jpg",\n        "colorModel":'
new = '"format": "jpg",\n        "icoSizes": [16,24,32,48,64,128,256],\n        "colorModel":'
if raw.count(old) != 2: raise SystemExit(f"FAIL ICO defaults: expected two, got {raw.count(old)}")
p.write_text(raw.replace(old, new), encoding="utf-8", newline="\n")
print("OK dialog and sidebar default icon sizes")

rep(panel,
 '''"��ȷ������� �� JPEG / ͸�� PNG �� ICC ɫ�ʹ���"''',
 '''"��ȷ������� �� JPEG / ͸�� PNG / ��ߴ� ICO �� ICC ɫ�ʹ���"''',
 "export dialog header")
rep(panel,
 '''select(ui, d, "format", &["jpg", "png"], &["JPEG (.jpg)", "PNG ͸��ͼ (.png)"], 244.0);''',
 '''select(ui, d, "format", &["jpg", "png", "ico"],
                &["JPEG (.jpg)", "PNG ͸��ͼ (.png)", "Windows ͼ�� (.ico)"], 244.0);''',
 "select ICO in full export dialog")

rep(panel,
 '''                ui.label("RGB��PNG ֧��͸����");''',
 '''                ui.label("RGB��PNG/ICO ֧��͸����");''',
 "full dialog RGB/alpha hint")

rep(panel,
 '''            ui.label("�ֱ��ʣ�DPI��");
            ui.horizontal(|ui| {''',
 '''            if d.str("format") == "ico" {
                ico_size_picker(ui, d);
                ui.end_row();
            } else {
            ui.label("�ֱ��ʣ�DPI��");
            ui.horizontal(|ui| {''',
 "show ICO size controls rather than DPI")
rep(panel,
 '''            ui.end_row();

            if d.str("format") == "jpg" {
                ui.label("ѹ������");''',
 '''            ui.end_row();
            }

            if d.str("format") == "jpg" {
                ui.label("ѹ������");''',
 "close optional DPI controls")

rep(panel,
 '''        if let Some(rect) = bounds {
            let scale = d.f64("ppi", 300.0) / 72.0;''',
 '''        if d.str("format") == "ico" {
            ui.label(egui::RichText::new("ICO: 16�C256 px �� ��ߴ�͸��ͼ��").size(19.0).strong());
            ui.label("���ߴ��ʸ�����������Ⱦ���Զ��ȱȾ��е������λ�����");
        } else if let Some(rect) = bounds {
            let scale = d.f64("ppi", 300.0) / 72.0;''',
 "display icon previews by resolution set")

rep(panel,
 '''"ʹ�� VectorCraft ԭ����Ⱦ�� ICC ��������ICO ��δ��������һԤ���档"''',
 '''"JPEG/PNG ʹ��ԭ��ɫ�����棻ICO ʹ�� RGB ͸�� PNG ͼ���װ��Windows ��׼��ʽ��"''',
 "update full dialog help")

rep(panel,
 '''    if format != "jpg" && format != "png" { return Err("��ѡ�� JPEG �� PNG".into()); }''',
 '''    if !["jpg", "png", "ico"].contains(&format.as_str()) {
        return Err("��ѡ�� JPEG��PNG �� ICO".into());
    }''',
 "accept ICO for export params")

rep(panel,
 '''    if format == "jpg" {
        let color = d.str("colorModel");''',
 '''    if format == "ico" {
        let sizes = d.fields.get("icoSizes").and_then(Value::as_array)
            .filter(|v| !v.is_empty()).ok_or("��ѡ������һ�� ICO �ߴ�")?;
        row["icoSizes"] = json!(sizes);
        row["antiAlias"] = json!("art");
        // Windows ICO is RGB with straight alpha; DPI, ICC and JPEG settings
        // have no meaning in .ico files and are intentionally ignored.
        if let Some(fields) = row.as_object_mut() { fields.remove("ppi"); }
    }
    if format == "jpg" {
        let color = d.str("colorModel");''',
 "route icon sizes to native screens export pipeline")

# ico_size_picker relies only on existing dialog store, and can be used in both
# standalone dialog and sidebar.
rep(panel,
 '''pub fn open(app: &mut VectorcraftApp) {''',
 '''fn ico_size_picker(ui: &mut egui::Ui, d: &mut Dialog) {
    ui.label("ICO �ߴ�");
    let mut sizes: Vec<u32> = d.fields.get("icoSizes")
        .and_then(Value::as_array)
        .map(|v| v.iter().filter_map(Value::as_u64).filter_map(|n| u32::try_from(n).ok()).collect())
        .unwrap_or_else(|| vec![16,24,32,48,64,128,256]);
    ui.horizontal_wrapped(|ui| {
        for side in [16,24,32,48,64,128,256] {
            let mut checked = sizes.contains(&side);
            if ui.checkbox(&mut checked, format!("{side}")).changed() {
                if checked && !sizes.contains(&side) {
                    sizes.push(side);
                } else if !checked {
                    sizes.retain(|n| *n != side);
                }
            }
        }
    });
    sizes.sort_unstable();
    set(d, "icoSizes", sizes);
    ui.label(egui::RichText::new("һ�� .ico �ļ��ڰ�����ѡ�����гߴ磬�Զ�����͸��������").small().weak());
}

pub fn open(app: &mut VectorcraftApp) {''',
 "reusable multi-size ICO checkboxes")

rep(panel,
 '''ui.label(egui::RichText::new("JPEG �� PNG �� ICC �� ��������").small().weak());''',
 '''ui.label(egui::RichText::new("JPEG �� PNG �� ICO �� ICC �� ��������").small().weak());''',
 "sidebar header")
rep(panel,
 '''select(ui, &mut d, "format", &["jpg", "png"],
            &["JPEG (.jpg)", "͸�� PNG (.png)"], ui.available_width().clamp(180.0, 280.0));''',
 '''select(ui, &mut d, "format", &["jpg", "png", "ico"],
            &["JPEG (.jpg)", "͸�� PNG (.png)", "Windows ICO (.ico)"], ui.available_width().clamp(180.0, 280.0));''',
 "sidebar ICO format option")

rep(panel,
 '''            ui.label(egui::RichText::new("PNG �� RGB �����֧��͸������").small().weak());''',
 '''            ui.label(egui::RichText::new("PNG / ICO �� RGB �����֧��͸������").small().weak());''',
 "sidebar transparent export hint")

rep(panel,
 '''        ui.label("�ֱ��ʣ�DPI��");
        ui.horizontal_wrapped(|ui| {''',
 '''        if d.str("format") == "ico" {
            ico_size_picker(ui, &mut d);
        } else {
        ui.label("�ֱ��ʣ�DPI��");
        ui.horizontal_wrapped(|ui| {''',
 "sidebar displays ICO sizes, hides irrelevant DPI")

rep(panel,
 '''        if d.str("format") == "jpg" {
            ui.label("ѹ����ʽ");''',
 '''        }
        if d.str("format") == "jpg" {
            ui.label("ѹ����ʽ");''',
 "close sidebar DPI branch")

rep(panel,
 '''        if let Some(rect) = app.session.active()
            .and_then(|s| s.doc.artboards.get(board))''',
 '''        if d.str("format") == "ico" {
            ui.label(egui::RichText::new("16 / 24 / 32 / 48 / 64 / 128 / 256 px").strong());
        } else if let Some(rect) = app.session.active()
            .and_then(|s| s.doc.artboards.get(board))''',
 "sidebar ICO size preview")

rep(panel,
 '''egui::Button::new("���� JPEG / PNG")''',
 '''egui::Button::new(if d.str("format") == "ico" { "���� Windows ICO" } else { "���� JPEG / PNG" })''',
 "sidebar ICO export button")

print("SUCCESS Leo Design ICO export for VectorCraft 0.19; native .ico output, transparency, multiple sizes and batch artboards")


//! Leo Design Image Pro Export v1.0.0 - a dedicated Chinese raster export dialog.
//! Uses the existing, tested Export for Screens renderer and output pipeline.
use serde_json::{Value, json};
use vectorcraft_engine::cmd::fileio;
use vectorcraft_doc::ColorMode;

use super::DialogSpec;
use crate::state::Dialog;
use crate::{VectorcraftApp, io, widgets};

pub const KIND: &str = "leoImageProExport";
pub const SPEC: DialogSpec = DialogSpec {
    heading: |_| "Leo Design · 图像专业导出".into(),
    body,
    confirm,
    ok: Some("Export"),
    min_width: 475.0,
    ..DialogSpec::FORM
};

fn set(d: &mut Dialog, k: &str, value: impl serde::Serialize) {
    d.fields.insert(k.into(), json!(value));
}

fn select(ui: &mut egui::Ui, d: &mut Dialog, key: &str, ids: &[&str], labels: &[&str], width: f32) {
    let current = d.str(key);
    let at = ids.iter().position(|v| *v == current).unwrap_or(0);
    if let Some(index) = widgets::dropdown(ui, ("leo-image", key), labels[at], labels, width) {
        if let Some(value) = ids.get(index) { set(d, key, *value); }
    }
}

pub fn open(app: &mut VectorcraftApp) {
    let n = app.session.active().map_or(0, |s| s.doc.artboards.len());
    let board = crate::panels::artboards::selected(app, n);
    let cmyk = app.session.active().is_some_and(|s| s.doc.color_mode == ColorMode::Cmyk);
    app.ui.dialog = Some(Dialog::new(KIND, json!({
        "format": "jpg",
        "colorModel": if cmyk { "cmyk" } else { "rgb" },
        "embedIcc": true,
        "quality": 100,
        "ppi": 300.0,
        "method": "baseline",
        "antiAlias": "art",
        "scope": "current",
        "currentBoard": board,
        "range": format!("1-{}", n.max(1)),
        "nameMode": "number",
        "folder": fileio::export_folder().unwrap_or_default(),
        "openLocation": false
    })));
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    ui.label(egui::RichText::new("精确画板裁切 · JPEG / 透明 PNG · ICC 色彩管理").small().weak());
    ui.add_space(8.0);

    ui.group(|ui| {
        ui.set_min_width(455.0);
        ui.heading("图像设置");
        ui.add_space(4.0);
        egui::Grid::new("leo-img-setting").num_columns(2).spacing([18.0, 9.0]).show(ui, |ui| {
            ui.label("导出格式");
            select(ui, d, "format", &["jpg", "png"], &["JPEG (.jpg)", "PNG 透明图 (.png)"], 244.0);
            ui.end_row();

            if d.str("format") == "jpg" {
                ui.label("颜色模式");
                select(ui, d, "colorModel", &["rgb", "cmyk", "gray"], &["RGB", "CMYK（印刷）", "灰度"], 244.0);
                ui.end_row();
                ui.label("ICC 配置");
                ui.label("跟随工作色彩配置（CMYK 使用当前 ICC）");
                ui.end_row();
                ui.label("");
                let mut embedded = d.bool("embedIcc");
                if ui.checkbox(&mut embedded, "嵌入 ICC 配置文件").changed() {
                    set(d, "embedIcc", embedded);
                }
                ui.end_row();
            } else {
                ui.label("颜色模式");
                ui.label("RGB（PNG 支持透明）");
                ui.end_row();
                ui.label("色彩配置");
                ui.label("导出 sRGB");
                ui.end_row();
            }
        });
    });

    ui.add_space(7.0);
    ui.group(|ui| {
        ui.set_min_width(455.0);
        ui.heading("输出设置");
        ui.add_space(4.0);
        egui::Grid::new("leo-output-setting").num_columns(2).spacing([18.0, 9.0]).show(ui, |ui| {
            if d.str("format") == "jpg" {
                ui.label("JPEG 品质");
                let mut quality = d.f64("quality", 100.0).round().clamp(1.0, 100.0) as u32;
                if ui.add(egui::Slider::new(&mut quality, 1..=100).show_value(true)).changed() {
                    set(d, "quality", quality);
                }
                ui.end_row();
            }

            ui.label("分辨率（DPI）");
            ui.horizontal(|ui| {
                let mut ppi = d.f64("ppi", 300.0).clamp(1.0, 4608.0);
                if ui.add(egui::DragValue::new(&mut ppi).speed(1.0).range(1.0..=4608.0).suffix(" dpi")).changed() {
                    set(d, "ppi", ppi);
                }
                if let Some(i) = widgets::dropdown(ui, "leo-img-dpi", "常用预设", &["72", "150", "300", "600", "1200"], 105.0) {
                    if let Some(p) = [72, 150, 300, 600, 1200].get(i) { set(d, "ppi", *p); }
                }
            });
            ui.end_row();

            if d.str("format") == "jpg" {
                ui.label("压缩方法");
                select(ui, d, "method", &["baseline", "optimized", "progressive"],
                    &["基线（标准）", "基线（优化）", "渐进式"], 244.0);
                ui.end_row();
            }

            ui.label("画板范围");
            select(ui, d, "scope", &["current", "all", "range"],
                &["当前画板", "全部画板", "指定范围"], 244.0);
            ui.end_row();
            if d.str("scope") == "range" {
                ui.label("指定画板");
                let mut range = d.str("range");
                if ui.add(egui::TextEdit::singleline(&mut range).desired_width(210.0).hint_text("例如 1-3, 6")).changed() {
                    set(d, "range", range);
                }
                ui.end_row();
            }

            ui.label("文件名后缀");
            select(ui, d, "nameMode", &["number", "artboard", "plain"],
                &["画板序号（_01）", "画板名称", "仅文档名"], 244.0);
            ui.end_row();

            ui.label("边缘处理");
            select(ui, d, "antiAlias", &["art", "type", "none"],
                &["优化图稿（推荐）", "优化文字", "无抗锯齿"], 244.0);
            ui.end_row();
        });
    });

    ui.add_space(7.0);
    ui.group(|ui| {
        ui.set_min_width(455.0);
        ui.heading("预计像素");
        let n = app.session.active().map_or(0, |s| s.doc.artboards.len());
        let index = (d.f64("currentBoard", 0.0) as usize).min(n.saturating_sub(1));
        let bounds = app.session.active().and_then(|s| s.doc.artboards.get(index)).map(|a| a.rect);
        if let Some(rect) = bounds {
            let scale = d.f64("ppi", 300.0) / 72.0;
            let (w, h) = vectorcraft_render::region_pixels(rect, scale);
            ui.label(egui::RichText::new(format!("{w} × {h} px")).size(22.0).strong());
            if d.str("scope") != "current" {
                ui.label("以上为当前画板，批量导出时每张画板按自身尺寸计算。");
            }
        } else {
            ui.label("请先打开含有画板的文件。");
        }
    });

    ui.add_space(7.0);
    ui.group(|ui| {
        ui.set_min_width(455.0);
        ui.heading("导出路径");
        ui.horizontal(|ui| {
            let mut folder = d.str("folder");
            if ui.add(egui::TextEdit::singleline(&mut folder).desired_width(345.0).hint_text("选择导出文件夹")).changed() {
                set(d, "folder", folder);
            }
            if app.services.pick_folder.is_some() && ui.button("选择…").clicked() {
                crate::picks::folder_field(app, d, "folder");
            }
        });
        let mut open = d.bool("openLocation");
        if ui.checkbox(&mut open, "导出后打开文件夹").changed() { set(d, "openLocation", open); }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("使用 VectorCraft 原生渲染与 ICC 编码器。ICO 暂未包含在这一预览版。").small().weak());
    false
}

fn params(d: &Dialog, document_cmyk: bool) -> Result<Value, String> {
    let format = d.str("format");
    if format != "jpg" && format != "png" { return Err("请选择 JPEG 或 PNG".into()); }
    let ppi = d.f64("ppi", 300.0);
    if !ppi.is_finite() || !(1.0..=4608.0).contains(&ppi) {
        return Err("分辨率必须为 1–4608 DPI".into());
    }
    let folder = d.str("folder");
    if folder.trim().is_empty() { return Err("请先选择导出文件夹".into()); }
    let scope = d.str("scope");
    let name = match d.str("nameMode").as_str() {
        "artboard" => "<filename>_<name>",
        "plain" => "<filename>",
        _ => "<filename>_##",
    };
    let mut row = json!({
        "format": format,
        "ppi": ppi,
        "suffix": "",
        "antiAlias": d.str("antiAlias"),
        "background": if format == "jpg" { "white" } else { "transparent" }
    });
    if format == "jpg" {
        let color = d.str("colorModel");
        row["colorModel"] = json!(if ["rgb", "cmyk", "gray"].contains(&color.as_str()) {
            color.as_str()
        } else if document_cmyk { "cmyk" } else { "rgb" });
        row["quality"] = json!(d.f64("quality", 100.0).round().clamp(1.0, 100.0) as u32);
        row["method"] = json!(d.str("method"));
        row["scans"] = json!(3);
        row["embedIcc"] = json!(d.bool("embedIcc"));
    }
    let mut p = json!({
        "folder": folder,
        "formats": [row],
        "folders": "none",
        "name": name,
        "openLocation": d.bool("openLocation")
    });
    match scope.as_str() {
        "current" => p["artboard"] = json!(d.f64("currentBoard", 0.0) as u32),
        "range" => {
            let range = d.str("range");
            if range.trim().is_empty() { return Err("请输入画板范围，例如 1-3, 6".into()); }
            p["range"] = json!(range);
        }
        "all" => p["range"] = json!("all"),
        _ => return Err("未知的画板范围".into()),
    }
    if scope == "current" && d.str("nameMode") == "number" {
        p["counterStart"] = json!((d.f64("currentBoard", 0.0) as u64) + 1);
    }
    Ok(p)
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let cmyk = app.session.active().is_some_and(|s| s.doc.color_mode == ColorMode::Cmyk);
    let p = params(d, cmyk)?;
    // Validate artboard choices before closing the dialog.
    let count = app.session.active().map_or(0, |s| s.doc.artboards.len());
    let range = p.get("range").and_then(Value::as_str).map(str::to_string);
    if let Some(range) = range {
        fileio::ArtboardPick { range: Some(range), ..Default::default() }.resolve(count)?;
    } else if p["artboard"].as_u64().is_none_or(|i| i as usize >= count) {
        return Err("当前画板无效，请重新选择。".into());
    }
    let r = io::export_for_screens(app, p)?;
    app.ui.dialog = None;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jpeg_uses_explicit_cmyk_profile_and_dpi() {
        let d = Dialog::new(KIND, json!({"format":"jpg", "colorModel":"cmyk",
            "embedIcc":true,"ppi":1200.0,"quality":100,"method":"optimized",
            "antiAlias":"art","scope":"current","currentBoard":5,"nameMode":"number",
            "folder":"C:/output"}));
        let p = params(&d, true).unwrap();
        assert_eq!(p["formats"][0]["colorModel"], "cmyk");
        assert_eq!(p["formats"][0]["ppi"], 1200.0);
        assert_eq!(p["formats"][0]["embedIcc"], true);
        assert_eq!(p["artboard"], 5);
        assert_eq!(p["counterStart"], 6);
    }
    #[test]
    fn transparent_png_and_batch_board_range() {
        let d = Dialog::new(KIND, json!({"format":"png","ppi":300,"scope":"range",
            "range":"1-3, 6","nameMode":"artboard","folder":"C:/out","antiAlias":"art"}));
        let p = params(&d, false).unwrap();
        assert_eq!(p["formats"][0]["background"], "transparent");
        assert_eq!(p["range"], "1-3, 6");
        assert_eq!(p["name"], "<filename>_<name>");
    }
}

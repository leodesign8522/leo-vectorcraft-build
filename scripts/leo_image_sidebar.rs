
// ============================================================================
// Leo Design v0.4.1 - non-modal dockable sidebar, retains the export dialog core.
// ============================================================================

/// The new Window > Leo Design Image Pro Export dock panel.
pub fn show_sidebar(app: &mut VectorcraftApp, ui: &mut egui::Ui) {
    // egui's temporary memory lives across frames and never modifies a document.
    let id = egui::Id::new("leo-design-image-export-sidebar-v1");
    let mut d = ui.ctx().data_mut(|data| data.get_temp::<Dialog>(id)).unwrap_or_else(|| sidebar_defaults(app));
    // Current means the artboard now shown, not the one active when the panel first opened.
    let n = app.session.active().map_or(0, |s| s.doc.artboards.len());
    let board = crate::panels::artboards::selected(app, n);
    set(&mut d, "currentBoard", board);
    let mut do_export = false;
    egui::ScrollArea::vertical().id_salt("leo-image-export-scroll").auto_shrink([false, true]).show(ui, |ui| {
        ui.set_min_width(190.0);
        ui.heading("Leo Design");
        ui.label(egui::RichText::new("图像专业导出").strong());
        ui.label(egui::RichText::new("JPEG · PNG · ICC · 批量画板").small().weak());
        ui.separator();

        ui.label(egui::RichText::new("图像设置").strong());
        ui.label("导出格式");
        select(ui, &mut d, "format", &["jpg", "png"],
            &["JPEG (.jpg)", "透明 PNG (.png)"], ui.available_width().clamp(180.0, 280.0));

        if d.str("format") == "jpg" {
            ui.label("颜色模式");
            select(ui, &mut d, "colorModel", &["rgb", "cmyk", "gray"],
                &["RGB", "CMYK（印刷）", "灰度"], ui.available_width().clamp(180.0, 280.0));
            ui.label(egui::RichText::new("ICC：跟随当前工作配置").small().weak());
            let mut embed = d.bool("embedIcc");
            if ui.checkbox(&mut embed, "嵌入 ICC 配置文件").changed() { set(&mut d, "embedIcc", embed); }
        } else {
            ui.label(egui::RichText::new("PNG 以 RGB 输出，支持透明背景").small().weak());
        }
        ui.add_space(8.0);
        ui.separator();

        ui.label(egui::RichText::new("输出设置").strong());
        if d.str("format") == "jpg" {
            ui.label("JPEG 品质");
            let mut q = d.f64("quality", 100.0).round().clamp(1.0, 100.0) as u32;
            if ui.add(egui::Slider::new(&mut q, 1..=100)).changed() { set(&mut d, "quality", q); }
        }
        ui.label("分辨率（DPI）");
        ui.horizontal_wrapped(|ui| {
            let mut ppi = d.f64("ppi", 300.0).clamp(1.0, 4608.0);
            if ui.add(egui::DragValue::new(&mut ppi).range(1.0..=4608.0).speed(1.0).suffix(" dpi")).changed() {
                set(&mut d, "ppi", ppi);
            }
            if let Some(i) = widgets::dropdown(ui, "leo-sidebar-ppi", "预设", &["72", "150", "300", "600", "1200"], 72.0)
                && let Some(ppi) = [72, 150, 300, 600, 1200].get(i)
            {
                set(&mut d, "ppi", *ppi);
            }
        });
        if d.str("format") == "jpg" {
            ui.label("压缩方式");
            select(ui, &mut d, "method", &["baseline", "optimized", "progressive"],
                &["基线（标准）", "基线（优化）", "渐进式"], ui.available_width().clamp(180.0, 280.0));
        }
        ui.label("画板范围");
        select(ui, &mut d, "scope", &["current", "all", "range"],
            &["当前画板", "全部画板", "指定范围"], ui.available_width().clamp(180.0, 280.0));
        if d.str("scope") == "range" {
            let mut range = d.str("range");
            if ui.add(egui::TextEdit::singleline(&mut range).desired_width(ui.available_width()).hint_text("1-3, 6")).changed() {
                set(&mut d, "range", range);
            }
        }
        ui.label("文件名");
        select(ui, &mut d, "nameMode", &["number", "artboard", "plain"],
            &["画板序号（_01）", "画板名称", "仅文档名"], ui.available_width().clamp(180.0, 280.0));
        ui.label("边缘处理");
        select(ui, &mut d, "antiAlias", &["art", "type", "none"],
            &["优化图稿", "优化文字", "不抗锯齿"], ui.available_width().clamp(180.0, 280.0));
        ui.add_space(8.0);
        ui.separator();

        ui.label(egui::RichText::new("预计像素").strong());
        if let Some(rect) = app.session.active()
            .and_then(|s| s.doc.artboards.get(board))
            .map(|a| a.rect)
        {
            let (w, h) = vectorcraft_render::region_pixels(rect, d.f64("ppi", 300.0) / 72.0);
            ui.label(egui::RichText::new(format!("{w} × {h} px")).size(18.0).strong());
        } else {
            ui.label("请先打开带画板的文件。");
        }
        ui.add_space(8.0);
        ui.separator();

        ui.label(egui::RichText::new("导出路径").strong());
        let mut folder = d.str("folder");
        if ui.add(egui::TextEdit::singleline(&mut folder).desired_width(ui.available_width())
            .hint_text("导出文件夹")).changed()
        {
            set(&mut d, "folder", folder);
        }
        if ui.button("选择文件夹…").clicked() {
            if let Some(folder) = crate::picks::folder(app) {
                set(&mut d, "folder", folder);
            }
        }
        let mut open_folder = d.bool("openLocation");
        if ui.checkbox(&mut open_folder, "导出后打开文件夹").changed() {
            set(&mut d, "openLocation", open_folder);
        }
        ui.add_space(10.0);
        do_export = ui.add_sized([ui.available_width().max(160.0), 30.0], egui::Button::new("导出 JPEG / PNG")).clicked();
        ui.label(egui::RichText::new("使用原生导出引擎，原稿内容不变").small().weak());
    });
    // Save state before running a job (which may require opening a file dialog or an alert).
    ui.ctx().data_mut(|data| data.insert_temp(id, d.clone()));
    if do_export {
        if let Err(err) = sidebar_export(app, &d) {
            app.status(err);
        }
    }
}

fn sidebar_defaults(app: &VectorcraftApp) -> Dialog {
    let n = app.session.active().map_or(0, |s| s.doc.artboards.len());
    let index = crate::panels::artboards::selected(app, n);
    let cmyk = app.session.active().is_some_and(|s| s.doc.color_mode == ColorMode::Cmyk);
    Dialog::new(KIND, json!({
        "format": "jpg",
        "colorModel": if cmyk { "cmyk" } else { "rgb" },
        "embedIcc": true,
        "quality": 100,
        "ppi": 300.0,
        "method": "baseline",
        "antiAlias": "art",
        "scope": "current",
        "currentBoard": index,
        "range": format!("1-{}", n.max(1)),
        "nameMode": "number",
        "folder": fileio::export_folder().unwrap_or_default(),
        "openLocation": false
    }))
}

fn sidebar_export(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let cmyk = app.session.active().is_some_and(|s| s.doc.color_mode == ColorMode::Cmyk);
    let p = params(d, cmyk)?;
    let n = app.session.active().map_or(0, |s| s.doc.artboards.len());
    if let Some(range) = p.get("range").and_then(Value::as_str) {
        fileio::ArtboardPick { range: Some(range.into()), ..Default::default() }.resolve(n)?;
    } else if p["artboard"].as_u64().is_none_or(|i| i as usize >= n) {
        return Err("无效的当前画板，请重新选择。".into());
    }
    // Do not close or create a modal dialog. The sidebar remains available for the next export.
    io::export_for_screens(app, p)
}

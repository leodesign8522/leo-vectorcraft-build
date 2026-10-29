//! Leo Design: direct access to unit preferences and document color mode
//! from dedicated right-dock icon panels. Uses existing undoable engine commands.

use egui::Ui;
use serde_json::json;
use vectorcraft_doc::{ColorMode, Unit};

use crate::{VectorcraftApp, widgets};

fn unit_name(key: &str) -> &str {
    match key {
        "points" => "点 (pt)",
        "picas" => "派卡 (pc)",
        "inches" => "英寸 (in)",
        "millimeters" => "毫米 (mm)",
        "centimeters" => "厘米 (cm)",
        "pixels" => "像素 (px)",
        "feetInches" => "英尺与英寸",
        "meters" => "米 (m)",
        "yards" => "码 (yd)",
        "feet" => "英尺 (ft)",
        _ => key,
    }
}

/// Dedicated, dockable quick-units control. The General preference also
/// changes the active document unit through the engine's prefs.set command.
/// Other unit categories remain preference values, saved by the host.
pub fn show_units(app: &mut VectorcraftApp, ui: &mut Ui) {
    ui.heading("Leo Design · 单位");
    ui.label("快速切换单位，无需打开首选项");
    ui.add_space(7.0);

    let prefs = app.session.prefs.to_json();
    let names: Vec<&str> = Unit::ALL.iter().map(|u| unit_name(u.key())).collect();
    for (key, label) in [
        ("unitsGeneral", "常规"),
        ("unitsStroke", "描边"),
        ("unitsType", "文字"),
        ("unitsAsianType", "东亚文字"),
    ] {
        let current = if key == "unitsGeneral" {
            app.session.general_unit().key()
        } else {
            prefs[key].as_str().unwrap_or("points")
        };
        let chosen_name = unit_name(current);
        ui.push_id(key, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized([67.0, 24.0], egui::Label::new(label));
                if let Some(index) = widgets::dropdown(ui, "leo-unit", chosen_name, &names, 155.0)
                    && let Some(unit) = Unit::ALL.get(index)
                {
                    match app.run("prefs.set", json!({"key": key, "value": unit.key()})) {
                        Ok(_) => {
                            // Same immediate persistence path as the Preferences dialog.
                            app.ui.engine_prefs = app.session.prefs.to_json();
                            app.prefs_changed = true;
                        }
                        Err(e) => app.ui.status = e,
                    }
                }
            });
        });
    }
    ui.add_space(6.0);
    ui.label(egui::RichText::new("常规单位随当前文档一起更新；其他单位保存为程序首选项。").small().weak());
}

/// Actual document mode, not Color panel's RGB/CMYK display model.
/// Uses the same file.documentColorMode command as File > Document Color Mode.
pub fn show_document_color(app: &mut VectorcraftApp, ui: &mut Ui) {
    ui.heading("Leo Design · 文档颜色模式");
    ui.add_space(5.0);
    let current = app.session.active().map(|st| st.doc.color_mode);
    let name = match current {
        Some(ColorMode::Cmyk) => "CMYK",
        Some(_) => "RGB",
        None => "未打开文档",
    };
    ui.label(format!("当前文档：{name}"));
    ui.add_space(7.0);
    ui.add_enabled_ui(current.is_some(), |ui| {
        ui.horizontal(|ui| {
            for (label, mode, is_selected) in [
                ("RGB 颜色", "rgb", current.is_some_and(|m| m != ColorMode::Cmyk)),
                ("CMYK 颜色", "cmyk", current == Some(ColorMode::Cmyk)),
            ] {
                if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                    if let Err(e) = app.run("file.documentColorMode", json!({"mode": mode})) {
                        app.ui.status = e;
                    }
                }
            }
        });
    });
    ui.add_space(7.0);
    ui.label(egui::RichText::new("这里修改的是文档颜色模式，和颜色面板的显示锁定不同。建议切换前保存文件。").small().weak());
}

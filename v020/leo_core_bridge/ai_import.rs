//! Explicit native-data AI open with an honest import report.
use crate::{FilePick, VectorcraftApp};
use serde_json::{Value, json};
use vectorcraft_doc::{NodeKind, TextKind};

pub fn open(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let path = p
        .get("path")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            crate::picks::open(
                app,
                &FilePick {
                    filters: vec![FilePick::filter("Illustrator document", &["ai"])],
                    ..Default::default()
                },
            )
        });
    let Some(path) = path else {
        return Ok(json!({"cancelled": true}));
    };
    if !path.to_ascii_lowercase().ends_with(".ai") {
        return Err("请选择 .ai 文件。".into());
    }
    let bytes = app.services.read.as_ref().ok_or("无文件读取服务")?(&path)?;
    // An explicit editable-first action: never ask the PDF page to reconstruct text frames.
    // The engine still reports unsupported data and protects the source from ordinary Save.
    let result = crate::io::open_document(
        app,
        &path,
        &bytes,
        Some(path.clone()),
        &json!({"editingData": "always", "layers": true, "textAs": "text"}),
    )?;
    crate::io::note_recent(app, &path);
    let mut texts = 0usize;
    let mut areas = 0usize;
    let mut groups = 0usize;
    let mut overflow = 0usize;
    if let Ok(state) = app.session.doc() {
        state.doc.walk(|n| match &n.kind {
            NodeKind::Text(t) => {
                texts += 1;
                if !matches!(t.kind, TextKind::Point)
                    && vectorcraft_text::layout(vectorcraft_text::FontDb::global(), t).overflow
                {
                    overflow += 1;
                }
                if matches!(t.kind, TextKind::Area { .. }) {
                    areas += 1;
                }
            }
            NodeKind::Group { .. } => groups += 1,
            _ => {}
        });
    }
    let report = report(&result, texts, areas, groups, overflow);
    crate::dialogs::confirm::tell(app, "AI 增强导入报告", &report);
    app.status("AI 导入完成，请检查导入报告中的字体和外观警告。");
    Ok(result)
}

fn report(r: &Value, texts: usize, areas: usize, groups: usize, overflow: usize) -> String {
    let source = if r["editingDataRead"].as_bool() == Some(true) {
        "已读取 AI 原始编辑数据。"
    } else if r["restored"].as_bool() == Some(true) {
        "已恢复文件中的 VectorCraft 原生文档。"
    } else {
        "未能读取原始编辑数据，已使用页面内容；原始图层、区域文字框和字形可能无法完整保留。"
    };
    let mut out = format!(
        "{source}\n可编辑文字：{texts}；区域文字框：{areas}；组：{groups}。\n原始 AI 文件保持不变；请另存为 VectorCraft 文档。\n"
    );
    if overflow > 0 {
        out.push_str(&format!(
            "\n文字溢出：{overflow} 个文字对象未能全部放入框内，请检查字体及框尺寸。\n"
        ));
    }
    if let Some(notes) = r["warnings"].as_array().filter(|v| !v.is_empty()) {
        out.push_str("\n需要检查（包含缺失字体、外观降级或文字框调整）：\n");
        for note in notes.iter().filter_map(Value::as_str) {
            out.push_str("• ");
            out.push_str(note);
            out.push('\n');
        }
    } else {
        out.push_str("\n导入器未报告警告；仍需核对文字换行和效果。");
    }
    out
}

#[cfg(test)]
mod leo_ai_import_tests {
    use super::*;
    #[test]
    fn fallback_is_never_reported_as_native_success() {
        let text = report(
            &json!({"editingDataRead": false, "warnings": ["missing font: Example"]}),
            3,
            1,
            2,
            1,
        );
        assert!(text.contains("未能读取原始编辑数据"));
        assert!(text.contains("missing font: Example"));
        assert!(text.contains("区域文字框：1"));
        assert!(!text.contains("已读取 AI 原始编辑数据"));
    }
    #[test]
    fn cancel_and_bad_extension_preserve_document() {
        let mut app = VectorcraftApp::new(
            vectorcraft_engine::Session::new(),
            crate::Services::default(),
        );
        let count = app.session.documents().len();
        assert_eq!(open(&mut app, &json!({})).unwrap()["cancelled"], true);
        assert!(open(&mut app, &json!({"path": "wrong.svg"})).is_err());
        assert_eq!(app.session.documents().len(), count);
    }
}

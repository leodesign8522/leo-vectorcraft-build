//! Leo Design Workspace Exchange — standalone, versioned JSON backups.
//! Never writes to the VectorCraft UI prefs; imports go through native state.
//! The separate shortcut JSON command is provided by official VectorCraft v0.18.

use serde_json::{json, Value};
use crate::{VectorcraftApp, FilePick};
use crate::workspaces::{self, Workspace};

const FORMAT: &str = "leo-vectorcraft-workspace";
const VERSION: u64 = 1;
const MAX_BYTES: usize = 2_000_000;

fn payload(w: &Workspace) -> Value {
    json!({"format": FORMAT, "version": VERSION, "workspace": w})
}

fn current_workspace(app: &VectorcraftApp, p: &Value) -> Result<Workspace, String> {
    let name = p.get("name").and_then(Value::as_str)
        .unwrap_or(&app.ui.workspace).trim();
    if name.eq_ignore_ascii_case(&app.ui.workspace) {
        // Export the visible layout, including unsaved changes to its panels.
        return Ok(workspaces::capture(&app.ui, &app.ui.workspace));
    }
    app.ui.workspace_changes.iter()
        .find(|w| w.name.eq_ignore_ascii_case(name)).cloned()
        .or_else(|| workspaces::find_saved(&app.ui, name))
        .ok_or_else(|| format!("工作区不存在：{name}"))
}

pub fn export(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let w = current_workspace(app, p)?;
    let v = payload(&w);
    let bytes = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    let pick = FilePick {
        filters: vec![FilePick::filter("Leo Design Workspace", &["json"])],
        ..FilePick::named("Leo Design Workspace.json")
    };
    let path = p.get("path").and_then(Value::as_str).map(str::to_string)
        .or_else(|| crate::picks::save(app, &pick));
    match path {
        Some(path) => {
            let write = app.services.write.as_mut().ok_or("无文件写入服务")?;
            write(&path, &bytes)?;
            Ok(json!({"path": path, "workspace": w.name}))
        }
        None if app.file_dialog_open() => Err("cancelled".into()),
        None => match app.services.download.as_mut() {
            Some(download) => download("Leo Design Workspace.json", &bytes)
                .map(|()| json!({"workspace": w.name})),
            None => Ok(v),
        },
    }
}

fn decode(v: &Value) -> Result<Workspace, String> {
    if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err("不是 Leo Design 工作区文件。".into());
    }
    if v.get("version").and_then(Value::as_u64) != Some(VERSION) {
        return Err("不支持的工作区文件版本。".into());
    }
    let w: Workspace = serde_json::from_value(
        v.get("workspace").cloned().ok_or("工作区内容缺失。")?
    ).map_err(|e| format!("工作区格式不兼容：{e}"))?;
    if w.name.trim().is_empty() || w.name.chars().count() > 96 {
        return Err("工作区名称为空或超过 96 个字符。".into());
    }
    if w.docking.as_ref().is_some_and(|layout| !crate::panel_docking::valid(layout)) {
        return Err("工作区布局不兼容当前 VectorCraft 版本。".into());
    }
    Ok(w)
}

fn unused_name(app: &VectorcraftApp, original: &str) -> Result<String, String> {
    let original = original.trim();
    if workspaces::find_saved(&app.ui, original).is_none() {
        return Ok(original.to_string());
    }
    for i in 1..=999 {
        let name = format!("{original} (导入{i})");
        if workspaces::find_saved(&app.ui, &name).is_none() {
            return Ok(name);
        }
    }
    Err("同名工作区太多，请先重命名旧工作区。".into())
}

fn install(app: &mut VectorcraftApp, v: &Value) -> Result<Value, String> {
    // Validate all input before changing preferences.
    let mut w = decode(v)?;
    let old_name = w.name.clone();
    let new_name = unused_name(app, &old_name)?;
    w.name = new_name.clone();
    app.ui.custom_workspaces.push(w);
    // Keep the existing workspace and its changes; switch using native API.
    workspaces::switch(&mut app.ui, &new_name)?;
    workspaces::sync(&app.ui);
    app.prefs_changed = true;
    app.status(format!("已导入工作区：{new_name}"));
    Ok(json!({"workspace": new_name, "renamed": new_name != old_name}))
}

pub fn import(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let v = if let Some(data) = p.get("data") {
        data.clone()
    } else {
        let pick = FilePick {
            filters: vec![FilePick::filter("Leo Design Workspace", &["json"])],
            ..Default::default()
        };
        let path = p.get("path").and_then(Value::as_str).map(str::to_string)
            .or_else(|| crate::picks::open(app, &pick))
            .ok_or("cancelled")?;
        let read = app.services.read.as_ref().ok_or("无文件读取服务")?;
        let bytes = read(&path)?;
        if bytes.len() > MAX_BYTES {
            return Err("工作区文件超过 2MB。".into());
        }
        serde_json::from_slice::<Value>(&bytes)
            .map_err(|e| format!("JSON 解析失败：{e}"))?
    };
    install(app, &v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_exchange_roundtrip() {
        let w = Workspace::default();
        assert_eq!(decode(&payload(&w)).unwrap().name, w.name);
    }

    #[test]
    fn format_and_version_are_checked() {
        let w = Workspace::default();
        let mut v = payload(&w);
        v["format"] = json!("untrusted");
        assert!(decode(&v).is_err());
        let mut v = payload(&w);
        v["version"] = json!(200);
        assert!(decode(&v).is_err());
    }

    #[test]
    fn blank_name_rejected() {
        let mut w = Workspace::default();
        w.name.clear();
        assert!(decode(&payload(&w)).is_err());
    }
}

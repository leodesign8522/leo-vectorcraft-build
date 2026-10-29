#!/usr/bin/env python3
"""Leo Design VectorCraft v0.5.7: portable workspace + shortcut preset I/O.

Run on the v0.17.0 source after the Leo Design Workspace Pro + panel sizing patches.
Workspace and shortcuts are separate JSON files; no full ui.json replacement.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_workspace_shortcut_io.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])


def once(rel, before, after, label):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    n = s.count(before)
    if n != 1:
        raise SystemExit(f"FAIL {label}: expected one anchor, found {n}")
    p.write_text(s.replace(before, after, 1), encoding="utf-8", newline="\n")
    print("OK", label)


workspace_io = r'''
// Leo Design workspace JSON exchange. Exports only layout and panel sizes,
// not shortcuts, ICC settings, color preferences, or other personal settings.
fn leo_export_workspace(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or(&app.ui.workspace).trim();
    let workspace = if name.eq_ignore_ascii_case(&app.ui.workspace) {
        capture(&app.ui, &app.ui.workspace)
    } else {
        app.ui.workspace_changes.iter().find(|w| w.name.eq_ignore_ascii_case(name))
            .cloned().or_else(|| find_saved(&app.ui, name))
            .ok_or_else(|| format!("工作区不存在：{name}"))?
    };
    let payload = json!({
        "format": "leo-vectorcraft-workspace",
        "version": 1,
        "workspace": workspace,
        "panelSizes": app.ui.leo_panel_sizes
    });
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
    let pick = crate::FilePick {
        filters: vec![crate::FilePick::filter("Leo Design Workspace JSON", &["json"])],
        ..crate::FilePick::named("Leo Design Workspace.json")
    };
    let path = p.get("path").and_then(Value::as_str).map(str::to_string)
        .or_else(|| crate::picks::save(app, &pick));
    match path {
        Some(path) => {
            let write = app.services.write.as_mut().ok_or("无法写入工作区文件")?;
            write(&path, &bytes)?;
            Ok(json!({"path": path, "workspace": workspace.name}))
        }
        None if app.file_dialog_open() => Err("cancelled".into()),
        None => match app.services.download.as_mut() {
            Some(download) => download("Leo Design Workspace.json", &bytes)
                .map(|_| json!({"workspace": workspace.name})),
            None => Ok(payload),
        }
    }
}

// Parse before modifying UI state: a bad file must leave the user's layout
// completely untouched. Duplicate names get a safe new name, not overwritten.
fn leo_import_workspace(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let data = if let Some(v) = p.get("data") {
        v.clone()
    } else {
        let pick = crate::FilePick {
            filters: vec![crate::FilePick::filter("Leo Design Workspace JSON", &["json"])],
            ..Default::default()
        };
        let path = p.get("path").and_then(Value::as_str).map(str::to_string)
            .or_else(|| crate::picks::open(app, &pick))
            .ok_or("cancelled")?;
        let read = app.services.read.as_ref().ok_or("无法读取工作区文件")?;
        let bytes = read(&path)?;
        if bytes.len() > 2_000_000 {
            return Err("工作区 JSON 超过 2 MB，请确认文件来源。".into());
        }
        serde_json::from_slice::<Value>(&bytes).map_err(|e| format!("JSON 解析失败：{e}"))?
    };
    if data.get("format").and_then(Value::as_str) != Some("leo-vectorcraft-workspace")
        || data.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err("不支持的工作区文件格式或版本。".into());
    }
    let mut workspace: Workspace = serde_json::from_value(
        data.get("workspace").cloned().ok_or("工作区数据缺失")?
    ).map_err(|e| format!("工作区内容无效：{e}"))?;

    let original = workspace.name.trim().to_string();
    if original.is_empty() || original.chars().count() > 96 {
        return Err("工作区名称为空或过长。".into());
    }
    if workspace.docking.as_ref().is_some_and(|layout| !crate::panel_docking::valid(layout)) {
        return Err("工作区包含无效或此版本不支持的面板布局。".into());
    }
    crate::state::FloatingPanels::sanitize(
        &mut workspace.floating_panels, &mut workspace.toolbar_pos
    );
    workspace.docking_icons.retain(|id| crate::state::rail_panel(id).is_some());
    workspace.docking_hidden.retain(|id, _| crate::menus::normalize_panel(id).is_some());
    workspace.docking_collapsed.retain(|id| crate::menus::normalize_panel(id).is_some());
    workspace.open_panel = workspace.open_panel
        .filter(|id| crate::menus::normalize_panel(id).is_some());
    // Optional panel dimensions are validated before touching the state.
    let mut sizes = std::collections::BTreeMap::<String, [f32; 2]>::new();
    if let Some(value) = data.get("panelSizes") {
        let parsed: std::collections::BTreeMap<String, [f32; 2]> =
            serde_json::from_value(value.clone()).map_err(|e| format!("面板尺寸无效：{e}"))?;
        for (key, size) in parsed {
            if crate::menus::normalize_panel(&key).is_some()
                && size.iter().all(|n| n.is_finite())
            {
                sizes.insert(key, [
                    size[0].clamp(220.0, 900.0),
                    if size[1] <= 0.0 { 0.0 } else { size[1].clamp(160.0, 950.0) }
                ]);
            }
        }
    }
    // Import never silently overwrites a saved workspace.
    let mut name = original.clone();
    if find_saved(&app.ui, &name).is_some() {
        for i in 1..=999 {
            let proposed = format!("{original} (导入{i})");
            if find_saved(&app.ui, &proposed).is_none() {
                name = proposed;
                break;
            }
        }
        if find_saved(&app.ui, &name).is_some() {
            return Err("同名工作区过多，无法生成新名称。".into());
        }
    }
    workspace.name = name.clone();
    app.ui.custom_workspaces.push(workspace);
    switch(&mut app.ui, &name)?;
    app.ui.leo_panel_sizes.extend(sizes);
    sync(&app.ui);
    app.prefs_changed = true;
    app.status(format!("已导入工作区：{name}"));
    Ok(json!({"workspace": name, "renamed": name != original}))
}

'''
once(
    "crates/ui-egui/src/workspaces.rs",
    "// ---------- commands ----------",
    workspace_io + "// ---------- commands ----------",
    "Workspace I/O functions",
)
once(
    "crates/ui-egui/src/workspaces.rs",
    '        "window.workspace.list" => Ok(json!({"current": app.ui.workspace, "workspaces": names(&app.ui), "custom": app.ui.custom_workspaces})),',
    '''        "window.workspace.list" => Ok(json!({"current": app.ui.workspace, "workspaces": names(&app.ui), "custom": app.ui.custom_workspaces})),
        "window.workspace.export" => leo_export_workspace(app, p),
        "window.workspace.import" => leo_import_workspace(app, p),''',
    "Workspace import/export commands",
)
once(
    "crates/ui-egui/src/workspaces.rs",
    '''    v.push(Item::Cmd("New Workspace…", "window.workspace.new", Value::Null));
    v.push(Item::Cmd("Manage Workspaces…", "window.workspace.manage", Value::Null));''',
    '''    v.push(Item::Cmd("New Workspace…", "window.workspace.new", Value::Null));
    v.push(Item::Cmd("Manage Workspaces…", "window.workspace.manage", Value::Null));
    v.push(Item::Sep);
    v.push(Item::Cmd("导出当前工作区…", "window.workspace.export", Value::Null));
    v.push(Item::Cmd("导入工作区…", "window.workspace.import", Value::Null));''',
    "Workspace menu and top bar commands",
)
once(
    "crates/ui-egui/src/menus.rs",
    '''    ("window.workspace.list", "List Workspaces", "", "{}"),''',
    '''    ("window.workspace.list", "List Workspaces", "", "{}"),
    ("window.workspace.export", "导出当前工作区…", "", "{path?, name?}"),
    ("window.workspace.import", "导入工作区…", "", "{path? | data?}"),''',
    "Register workspace I/O command metadata",
)
once(
    "crates/ui-egui/src/menus.rs",
    '''                c("Keyboard Shortcuts…", "edit.keyboardShortcuts"),
                c("Preferences…", "edit.preferences"),''',
    '''                c("Keyboard Shortcuts…", "edit.keyboardShortcuts"),
                sub("Leo Design · 快捷键备份", vec![
                    c("导出快捷键…", "shortcuts.export"),
                    c("导入快捷键…", "shortcuts.import"),
                ]),
                c("Preferences…", "edit.preferences"),''',
    "Add direct shortcut import/export under Edit",
)
print("SUCCESS: independent Leo Design workspace export/import; native shortcuts I/O menu")

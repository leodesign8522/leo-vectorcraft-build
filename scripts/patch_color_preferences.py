#!/usr/bin/env python3
"""Persist native VectorCraft colour working spaces across desktop restarts.

Targets upstream VectorCraft v0.17.0. Refuses to patch unexpected source layouts.
The colour-management profile files themselves remain in VectorCraft's existing
portable 'VectorCraftData/Color Profiles' storage.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python scripts/patch_color_preferences.py PATH_TO_VECTORCRAFT_ROOT")

root = Path(sys.argv[1])


def replace_once(path, original, replacement, label):
    source = path.read_text(encoding="utf-8")
    count = source.count(original)
    if count != 1:
        raise SystemExit(f"FAIL: {label}: expected one upstream anchor, found {count}")
    path.write_text(source.replace(original, replacement, 1), encoding="utf-8", newline="\n")


ui_prefs = root / "crates/ui-egui/src/prefs_dialog.rs"
replace_once(
    ui_prefs,
    "/// UI-only fields shown in the dialog (backed by `UiState`, not engine prefs).",
    '''/// Leo Design: persist working ICC settings via the UI crate's existing CMS dependency.
pub fn leo_color_settings_bytes() -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec_pretty(&vectorcraft_color::cms::active_settings())
}

pub fn leo_restore_color_settings(bytes: &[u8]) -> Result<(), String> {
    let st = serde_json::from_slice::<vectorcraft_color::cms::ColorSettings>(bytes)
        .map_err(|e| e.to_string())?;
    vectorcraft_color::cms::set_active(&st).map_err(|e| e.to_string())
}

/// UI-only fields shown in the dialog (backed by `UiState`, not engine prefs).''',
    "Colour settings persistence helpers in existing UI CMS dependency",
)

main = root / "apps/vectorcraft/src/main.rs"
replace_once(
    main,
    "fn load_prefs(app: &mut VectorcraftApp, saved: Option<vectorcraft_ui_egui::UiState>) {",
    """/// Leo Design: the working ICC colour settings live alongside portable ui.json.
/// The actual imported ICC files are separately kept by VectorCraft in Color Profiles.
/// Storing only the profile *names* here leaves the original file bytes untouched.
fn leo_color_settings_path() -> Option<std::path::PathBuf> {
    Some(prefs_path()?.with_file_name("Leo Color Settings.json"))
}

fn leo_restore_color_settings() {
    let Some(path) = leo_color_settings_path() else { return };
    let Ok(bytes) = std::fs::read(&path) else { return };
    if let Err(e) = vectorcraft_ui_egui::prefs_dialog::leo_restore_color_settings(&bytes) {
        // A missing or invalid ICC must never prevent the editor from starting.
        log::warn!("Leo Design: saved colour settings at {} were not restored: {e}", path.display());
    }
}

fn load_prefs(app: &mut VectorcraftApp, saved: Option<vectorcraft_ui_egui::UiState>) {""",
    "Colour settings persistence helper",
)
replace_once(
    main,
    "    vectorcraft_ui_egui::prefs_dialog::restore(app);\n}\n\nfn save_prefs",
    "    vectorcraft_ui_egui::prefs_dialog::restore(app);\n"
    "    // Profile store was already attached before this call, so saved ICC names resolve.\n"
    "    leo_restore_color_settings();\n"
    "}\n\nfn save_prefs",
    "Restore selected ICC settings at application startup",
)
replace_once(
    main,
    """        if let Ok(bytes) = prefs_bytes(app) {
            // Preferences are best effort: a failed write keeps the previous file.
            let _ = fileio::write_atomic(&p, &bytes);
        }
    }
}

/// The preferences file's contents:""",
    """        if let Ok(bytes) = prefs_bytes(app) {
            // Preferences are best effort: a failed write keeps the previous file.
            let _ = fileio::write_atomic(&p, &bytes);
        }
        // Persist RGB/CMYK working space, rendering intent and BPC together.
        // Use the same atomic writing used for UI prefs (and never modify the artwork).
        if let Ok(bytes) = vectorcraft_ui_egui::prefs_dialog::leo_color_settings_bytes() {
            if let Some(color_path) = leo_color_settings_path() {
                let _ = fileio::write_atomic(&color_path, &bytes);
            }
        }
    }
}

/// The preferences file's contents:""",
    "Persist ICC colour choice with normal preferences",
)

panel = root / "crates/ui-egui/src/panels/separations.rs"
replace_once(
    panel,
    """fn run(app: &mut VectorcraftApp, id: &str, p: Value) {
    if let Err(e) = app.run(id, p) {
        app.ui.status = e;
    }
}""",
    """fn run(app: &mut VectorcraftApp, id: &str, p: Value) {
    match app.run(id, p) {
        // Trigger host persistence immediately, rather than requiring a normal exit.
        Ok(_) if matches!(id, "edit.colorSettings" | "edit.assignProfile") => {
            app.prefs_changed = true;
        }
        Err(e) => app.ui.status = e,
        _ => {}
    }
}""",
    "Save selected working profile immediately",
)

print("SUCCESS: Leo Design working RGB/CMYK ICC preferences persist across restarts.")

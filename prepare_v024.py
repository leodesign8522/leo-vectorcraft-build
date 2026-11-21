from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
upstream = Path(sys.argv[1])
guide_paths = sorted((upstream / 'crates/tools/src/guides').rglob('*.rs'))
if not guide_paths:
    raise SystemExit('Official guides source is missing')
native_guides = {path: path.read_bytes() for path in guide_paths}
dest = root / 'v023'

# Keep the previously shipped Core features; fail loudly at incompatible API anchors.
patches = ['core_bridge', 'essential_features', 'ai_import', 'svg_clipboard_fonts',
           'image_export', 'panel_usability', 'ico_export',
           'smart_interactions', 'live_corner_geometry_link',
           'quick_settings',
           'optional_pie_prefs', 'doc_info_selection', 'action_shortcuts', 'eyedropper_text']
ported = {'ai_import', 'live_corner_geometry_link', 'panel_usability'}
for name in patches:
    folder = root / 'v024' if name in ported else dest
    subprocess.run([sys.executable, str(folder / f'patch_{name}.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'scripts/patch_color_panel_lock.py'), sys.argv[1]], check=True)
for patch in (root / 'v024' / 'patch_text_glyph_bounds.py',
              root / 'v024' / 'patch_text_modes.py',
              root / 'v024' / 'patch_ai_script_positions.py',
              dest / 'patch_view_navigation.py',
              root / 'v024' / 'patch_direct_whole_selection.py',
              dest / 'patch_ai_export.py',
              root / 'v024' / 'patch_ai_bridge.py'):
    subprocess.run([sys.executable, str(patch), sys.argv[1]], check=True)

subprocess.run([sys.executable, str(root / 'v024' / 'patch_ruler_guide_consumers.py'), sys.argv[1]], check=True)

# Keep upstream's partial-geometry clipboard implementation and verify it.
clipboard = upstream / 'crates/engine/src/tests/clipboard.rs'
with clipboard.open('a', encoding='utf-8') as file:
    file.write((dest / 'selected_segments_clipboard_tests.rs').read_text(encoding='utf-8'))

current_guides = {path: path.read_bytes()
                  for path in (upstream / 'crates/tools/src/guides').rglob('*.rs')}
if current_guides != native_guides:
    raise SystemExit('Core Bridge modified the official snapping engine')

subprocess.run([sys.executable, str(root / 'v024' / 'patch_snapping_preference.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'patch_leo_snapping.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'patch_glyph_move.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'patch_move_crossings.py'), sys.argv[1]], check=True)

subprocess.run([sys.executable, str(root / 'v024' / 'patch_geometry_tools.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_handle_feedback.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_multi_handles.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_handle_operations.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_native_commands.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_ui_keys.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_panel.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_native_interactions.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'path_tools' / 'patch_trim_join.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'v024' / 'patch_live_transform_preview.py'), sys.argv[1]], check=True)

p = Path(sys.argv[1]) / 'apps/vectorcraft/src/main.rs'
s = p.read_text(encoding='utf-8')
old = 'println!("vectorcraft {}", env!("CARGO_PKG_VERSION"));'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.24 executable version identity anchor')
s = s.replace(old, 'let _ = std::io::Write::write_all(&mut std::io::stdout(), b"vectorcraft 0.24.1 (Leo Design Core Bridge 0.20.4; Leo Snapping)\\n");')
p.write_text(s, encoding='utf-8', newline='\n')

p = Path(sys.argv[1]) / 'crates/ui-egui/src/dialogs/about.rs'
s = p.read_text(encoding='utf-8')
old = '    crate::community::links(app, ui);'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.24 About build identity anchor')
s = s.replace(old, '    ui.label("Leo Design Core Bridge 0.20.4");\n' + old)
p.write_text(s, encoding='utf-8', newline='\n')

# Restore the native thumbnail option when a document is reopened.
p = upstream / 'crates/engine/src/cmd/fileio/load.rs'
s = p.read_text(encoding='utf-8')
for before, after in [
    ("    pub native_pdf_compatible: Option<bool>,", "    pub native_pdf_compatible: Option<bool>,\n    /// Whether this native document was saved with a thumbnail.\n    pub native_preview: Option<bool>,"),
    ("    let mut native_pdf_compatible = None;", "    let mut native_pdf_compatible = None;\n    let mut native_preview = None;"),
    ("            native_pdf_compatible = Some(info.pdf_compatible);", "            native_pdf_compatible = Some(info.pdf_compatible);\n            native_preview = Some(vectorcraft_format::preview(bytes).is_some());"),
    ("editing_data, native_pdf_compatible })", "editing_data, native_pdf_compatible, native_preview })"),
    ("converted, native_pdf_compatible, editing_data, .. }", "converted, native_pdf_compatible, native_preview, editing_data, .. }"),
    ("    if let Some(pdf) = native_pdf_compatible {", "    if let Some(preview) = native_preview {\n        st.save_options.insert(\"preview\".into(), json!(preview));\n    }\n    if let Some(pdf) = native_pdf_compatible {"),
]:
    if s.count(before) != 1:
        raise SystemExit(f'Unexpected native thumbnail loader source: {before}')
    s = s.replace(before, after)
p.write_text(s, encoding='utf-8', newline='\n')

with (upstream / 'crates/engine/src/tests/saveoptions.rs').open('a', encoding='utf-8', newline='\n') as f:
    f.write('''
#[test]
fn reopened_native_thumbnail_follows_edits_and_respects_opt_out() {
    let dir = Folder::new("reopened-thumbnail");
    for compress in [false, true] {
        let path = dir.file(if compress { "compressed.vectorcraft" } else { "plain.vectorcraft" });
        let thumbnail = || vectorcraft_format::preview(&std::fs::read(&path).unwrap());
        let mut first = three_boards();
        first.execute("file.saveAs", &json!({"path":path, "preview":true, "compress":compress})).unwrap();
        let original = thumbnail().expect("initial preview");
        let mut reopened = Session::new();
        reopened.execute("document.open", &json!({"path":path})).unwrap();
        reopened.execute("shape.rectangle", &json!({"x":65,"y":65,"width":20,"height":20})).unwrap();
        reopened.execute("document.save", &json!({})).unwrap();
        let updated = thumbnail().expect("preview after reopening and saving");
        assert_ne!(original, updated);
        reopened.execute("document.save", &json!({"preview":false})).unwrap();
        let mut opted_out = Session::new();
        opted_out.execute("document.open", &json!({"path":path})).unwrap();
        opted_out.execute("document.save", &json!({})).unwrap();
        assert!(thumbnail().is_none());
    }
}
''')

# Imported formats do not carry a native thumbnail preference.
p = upstream / 'crates/engine/src/cmd/fileio/importer.rs'
s = p.read_text(encoding='utf-8')
before = 'native_pdf_compatible: None,'
if s.count(before) != 1:
    raise SystemExit('Unexpected custom importer initializer')
p.write_text(s.replace(before, before + '\n        native_preview: None,'), encoding='utf-8', newline='\n')

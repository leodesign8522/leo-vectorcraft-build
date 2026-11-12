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
ported = {'ai_import', 'live_corner_geometry_link'}
for name in patches:
    folder = root / 'v024' if name in ported else dest
    subprocess.run([sys.executable, str(folder / f'patch_{name}.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'scripts/patch_color_panel_lock.py'), sys.argv[1]], check=True)
for patch in (root / 'v024' / 'patch_text_glyph_bounds.py',
              root / 'v024' / 'patch_text_modes.py',
              dest / 'patch_view_navigation.py',
              root / 'v024' / 'patch_direct_whole_selection.py',
              dest / 'patch_ai_export.py',
              dest / 'patch_ai_bridge.py'):
    subprocess.run([sys.executable, str(patch), sys.argv[1]], check=True)

# Keep upstream's partial-geometry clipboard implementation and verify it.
clipboard = upstream / 'crates/engine/src/tests/clipboard.rs'
with clipboard.open('a', encoding='utf-8') as file:
    file.write((dest / 'selected_segments_clipboard_tests.rs').read_text(encoding='utf-8'))

current_guides = {path: path.read_bytes()
                  for path in (upstream / 'crates/tools/src/guides').rglob('*.rs')}
if current_guides != native_guides:
    raise SystemExit('Core Bridge modified the official snapping engine')

p = Path(sys.argv[1]) / 'apps/vectorcraft/src/main.rs'
s = p.read_text(encoding='utf-8')
old = 'println!("vectorcraft {}", env!("CARGO_PKG_VERSION"));'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.24 executable version identity anchor')
s = s.replace(old, 'let _ = std::io::Write::write_all(&mut std::io::stdout(), b"vectorcraft 0.24.0 (Leo Design Core Bridge 0.18.0)\\n");')
p.write_text(s, encoding='utf-8', newline='\n')

p = Path(sys.argv[1]) / 'crates/ui-egui/src/dialogs/about.rs'
s = p.read_text(encoding='utf-8')
old = '    crate::community::links(app, ui);'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.24 About build identity anchor')
s = s.replace(old, '    ui.label("Leo Design Core Bridge 0.18.0");\n' + old)
p.write_text(s, encoding='utf-8', newline='\n')

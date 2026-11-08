from pathlib import Path
import subprocess
import sys

root = Path(__file__).parent
dest = root / 'v022'
dest.mkdir(exist_ok=True)

# Keep the previously shipped Core features; fail loudly at incompatible API anchors.
patches = ['core_bridge', 'essential_features', 'ai_import', 'svg_clipboard_fonts',
           'image_export', 'panel_usability', 'ico_export', 'docked_panel_sizes',
           'workspace_flyout_sizes', 'smart_interactions', 'live_corner_geometry_link',
           'direct_marquee_edges', 'quick_settings',
           'optional_pie_prefs', 'doc_info_selection', 'action_shortcuts', 'eyedropper_text']
for name in patches:
    folder = 'v022' if name in ['essential_features', 'smart_interactions'] else 'v021'
    subprocess.run([sys.executable, str(root / folder / f'patch_{name}.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(root / 'scripts/patch_color_panel_lock.py'), sys.argv[1]], check=True)
subprocess.run([sys.executable, str(dest / 'patch_anchor_drag_feedback.py'), sys.argv[1]], check=True)
for source, target in [('geometry_regressions.rs', 'guides/leo_snapping_engine.rs'),
                       ('rotation_regressions.rs', 'guides/leo_snapping_engine.rs'),
                       ('direct_regressions.rs', 'direct/tests.rs'),
                       ('select_regressions.rs', 'select/tests.rs')]:
    with (Path(sys.argv[1]) / 'crates/tools/src' / target).open('a', encoding='utf-8') as f:
        f.write((dest / source).read_text(encoding='utf-8'))
# v0.22 implements partial clipboard geometry natively; retain it and test it.
p = Path(sys.argv[1]) / 'crates/engine/src/tests/clipboard.rs'
with p.open('a', encoding='utf-8') as f:
    f.write((dest / 'selected_segments_clipboard_tests.rs').read_text(encoding='utf-8'))
    f.write((dest / 'session_regressions.rs').read_text(encoding='utf-8'))

p = Path(sys.argv[1]) / 'apps/vectorcraft/src/main.rs'
s = p.read_text(encoding='utf-8')
old = 'println!("vectorcraft {}", env!("CARGO_PKG_VERSION"));'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.22 executable version identity anchor')
s = s.replace(old, 'let _ = std::io::Write::write_all(&mut std::io::stdout(), b"vectorcraft 0.22.0 (Leo Design Core Bridge 0.14.5; Snapping Engine 1.7)\\n");')
p.write_text(s, encoding='utf-8', newline='\n')

p = Path(sys.argv[1]) / 'crates/ui-egui/src/dialogs/about.rs'
s = p.read_text(encoding='utf-8')
old = '    crate::community::links(app, ui);'
if s.count(old) != 1:
    raise SystemExit('FAIL v0.22 About build identity anchor')
s = s.replace(old, '    ui.label("Leo Design Core Bridge 0.14.5 / Snapping Engine 1.7");\n' + old)
p.write_text(s, encoding='utf-8', newline='\n')

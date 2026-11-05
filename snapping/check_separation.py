#!/usr/bin/env python3
"""Enforce product ownership against the actual patched source, before compilation."""
from pathlib import Path
import sys

root, flavor = Path(sys.argv[1]), sys.argv[2]
if flavor not in ('core', 'snapping'):
    raise SystemExit('Unsupported edition; Pie Menu has been archived')
tools = root / 'crates/tools/src'
module = tools / 'guides/leo_snapping.rs'
prefs = (root / 'crates/engine/src/prefs.rs').read_text(encoding='utf-8')
direct = (tools / 'direct.rs').read_text(encoding='utf-8')
targets = (tools / 'guides/targets.rs').read_text(encoding='utf-8')
assert module.is_file(), 'Integrated snapping resolver missing'
assert (tools / 'guides/leo_rotation.rs').is_file(), 'Rotation snapping resolver missing'
assert 'self.leo_snap_from(p, from, tol)' in targets
assert 'MoveGroupAnchor' in direct
assert 'smart_guide_horizontal_color' in prefs
spec = (root / 'crates/engine/src/cmd/prefscmds.rs').read_text(encoding='utf-8')
assert '"Leo Core Bridge 吸附"' in spec
assert 'Leo Design 快捷键备份' not in (root / 'crates/ui-egui/src/menus/tree.rs').read_text(encoding='utf-8')
for rel in ['apps/vectorcraft/src/clipboard.rs','crates/ui-egui/src/shortcuts.rs','crates/ui-egui/src/shortcuts/paste.rs']:
    text = (root / rel).read_text(encoding='utf-8')
    for token in ['leo_pixpin', 'leo_screenshot_paste', 'leo_read_screenshot']:
        assert token not in text, f'Removed screenshot enhancement in {rel}: {token}'
for p in (root / 'crates/ui-egui/src').rglob('*.rs'):
    s = p.read_text(encoding='utf-8')
    for token in ('pub mod leo_pie_menu', 'crate::leo_pie_menu', 'ui.leoPieSettings'):
        assert token not in s, f'Archived Pie hook {token} in {p}'
print(f'PASS {flavor}: integrated Core Bridge, removed screenshot/menu enhancements, and no active Pie Menu')

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
if flavor == 'core':
    assert not module.exists(), 'Custom snap module leaked into Core'
    for token in ('leo_point_lock', 'leo_snap_from', 'EXTENSION_PREVIEW_COLOR', 'extension_preview('):
        assert token not in targets, f'Custom snapping leaked into Core: {token}'
    assert 'MoveGroupAnchor' not in direct, 'Group-anchor snap adapter leaked into Core'
    assert 'smart_guide_horizontal_color' not in prefs, 'Custom guide colour preferences leaked into Core'
else:
    assert module.is_file(), 'Independent snapping resolver missing'
    assert 'self.leo_snap_from(p, from, tol)' in targets
    assert 'MoveGroupAnchor' in direct
    assert 'smart_guide_horizontal_color' in prefs
    spec = (root / 'crates/engine/src/cmd/prefscmds.rs').read_text(encoding='utf-8')
    assert '"Leo Snapping"' in spec
for p in (root / 'crates/ui-egui/src').rglob('*.rs'):
    s = p.read_text(encoding='utf-8')
    for token in ('pub mod leo_pie_menu', 'crate::leo_pie_menu', 'ui.leoPieSettings'):
        assert token not in s, f'Archived Pie hook {token} in {p}'
print(f'PASS {flavor}: independent snapping ownership and no active Pie Menu')

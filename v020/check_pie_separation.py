#!/usr/bin/env python3
"""Fail the build if pie UI/input hooks leak into the Core-only executable."""
from pathlib import Path
import sys

root, flavor = Path(sys.argv[1]), sys.argv[2]
src = root / 'crates/ui-egui/src'
module = src / 'leo_pie_menu.rs'
hooks = {
    'shortcuts.rs': 'leo_pie_menu::keyboard_frame',
    'canvas.rs': 'leo_pie_menu::mouse_frame',
    'menus/tree.rs': 'ui.leoPieSettings',
}
assert flavor in ('core', 'pie')
if flavor == 'core':
    assert not module.exists(), 'Pie code must not be compiled into Core Bridge'
    for p in src.rglob('*.rs'):
        text = p.read_text(encoding='utf-8')
        for token in ('crate::leo_pie_menu', 'pub mod leo_pie_menu', 'ui.leoPieSettings',
                      'leo_pie_open', 'leo_pie_editor', 'leo_pie_owns_canvas'):
            assert token not in text, f'Unexpected pie hook {token} in {p}'
    assert 'pub leo_pies: serde_json::Value' in (src / 'state.rs').read_text(encoding='utf-8')
else:
    assert module.is_file(), 'Optional flavor must contain pie code'
    for rel, token in hooks.items():
        assert token in (src / rel).read_text(encoding='utf-8'), f'Missing {token}'
print(f'Verified {flavor} source separation')

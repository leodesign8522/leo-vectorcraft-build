#!/usr/bin/env python3
"""Apply the snapping modules included in Core Bridge."""
from pathlib import Path
import subprocess
import sys

repo = Path(__file__).resolve().parents[1]
root = Path(sys.argv[1]).resolve()
scripts = [
    'snapping/patch_group_anchor.py',
    'snapping/adapters/patch_smart_guide_colors.py',
    'snapping/adapters/patch_anchor_drag_snap.py',
    'snapping/adapters/patch_anchor_extension_axes.py',
    'snapping/adapters/patch_anchor_visuals.py',
    'snapping/adapters/patch_anchor_ruler_snap.py',
    'snapping/adapters/patch_anchor_drag_tests.py',
    'snapping/patch_resolver.py',
    'snapping/patch_guide_tests.py',
    'snapping/patch_rotation.py',
    'snapping/patch_move_contact.py',
]
for script in scripts:
    subprocess.run([sys.executable, str(repo / script), str(root)], check=True)
print('PASS Core Bridge snapping modules integrated')


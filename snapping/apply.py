#!/usr/bin/env python3
"""The only entry point for the optional Snapping edition. Never run from Core."""
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
]
for script in scripts:
    subprocess.run([sys.executable, str(repo / script), str(root)], check=True)
print('PASS Leo Design Snapping v0.1.1 integrated independently of Core')

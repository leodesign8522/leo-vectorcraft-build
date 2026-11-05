#!/usr/bin/env python3
"""Run real patch adapters against pinned upstream source fixtures, not string mocks."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

repo = Path(__file__).resolve().parents[1]
fixture = Path(sys.argv[1]).resolve()
sys.argv[1:] = []

class Integration(unittest.TestCase):
    def test_unified_core_integrates_snapping_and_preserves_native_clipboard(self):
        with tempfile.TemporaryDirectory(prefix='leo-snapping-', dir=fixture.parent) as tmp:
            assert Path(tmp).resolve().parent == fixture.parent
            root = Path(tmp) / 'upstream'
            shutil.copytree(fixture, root)
            native = ['crates/tools/src/direct.rs', 'crates/tools/src/guides.rs',
                      'crates/tools/src/guides/targets.rs', 'crates/tools/src/guides/snap.rs',
                      'crates/engine/src/prefs.rs', 'crates/engine/src/cmd/prefscmds.rs']
            before = {p: (root / p).read_bytes() for p in native}
            clipboard = ['apps/vectorcraft/src/clipboard.rs', 'crates/ui-egui/src/shortcuts/paste.rs']
            original_clipboard = {p: (root / p).read_bytes() for p in clipboard}
            subprocess.run([sys.executable, str(repo / 'v021/patch_smart_interactions.py'), str(root)], check=True)
            for p in native:
                self.assertEqual(before[p], (root / p).read_bytes(), p)
            subprocess.run([sys.executable, str(repo / 'snapping/apply.py'), str(root)], check=True)
            subprocess.run([sys.executable, str(repo / 'snapping/check_separation.py'), str(root), 'core'], check=True)
            for p in clipboard:
                self.assertEqual(original_clipboard[p], (root / p).read_bytes(), p)
            prefs = (root / 'crates/ui-egui/src/prefs_dialog.rs').read_text(encoding='utf-8')
            self.assertIn('if cat == "Leo Core Bridge 吸附"', prefs)
            self.assertNotIn('if cat == "Smart Guides"', prefs)

unittest.main()

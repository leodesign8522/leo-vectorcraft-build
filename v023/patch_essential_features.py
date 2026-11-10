#!/usr/bin/env python3
"""Font and paragraph compatibility; Ctrl+Enter stays with official v0.23.
Only apply after base Core Bridge; all anchors are checked. These port the
historical v0.17 behavior to v0.21 source before rebuilding the main program.
"""
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_essential_features.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])
project = Path(__file__).resolve().parents[1]

for script in [
    "patch_illustrator_text_bridge.py",
    "patch_pdf_cjk_font_names.py",
    "patch_chinese_paragraph.py",
]:
    print(f"Leo Design v0.21 port: {script}", flush=True)
    subprocess.run([sys.executable, str((project / "v021" if script == "patch_pdf_cjk_font_names.py" else project / "scripts") / script), str(root)], check=True)

print("SUCCESS: font/paragraph compatibility retained; Ctrl+Enter uses official v0.23 implementation")

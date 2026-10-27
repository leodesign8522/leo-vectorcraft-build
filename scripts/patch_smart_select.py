#!/usr/bin/env python3
"""Leo Design Smart Select v0.1.0: Ctrl+left-click borrows Group Selection.

Designed for VectorCraft v0.17.0. Fails closed if the expected source differs.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python patch_smart_select.py PATH_TO_tooling.rs")

path = Path(sys.argv[1])
source = path.read_text(encoding="utf-8")
anchor = '        if id == "directSelection" && mods.cmd && mods.alt {'
marker = '        // Leo Design: Ctrl-click in Selection borrows Group Selection.'

if marker in source:
    raise SystemExit("Leo Design patch is already applied; refusing to apply twice.")
if source.count(anchor) != 1:
    raise SystemExit("Expected VectorCraft v0.17.0 tooling.rs layout not found; refusing to patch.")

change = (
    marker + '\n'
    '        // Ctrl+Alt retains the upstream Select Behind behaviour.\n'
    '        if id == "selection" && mods.cmd && !mods.alt {\n'
    '            return Some("groupSelection");\n'
    '        }\n'
)
patched = source.replace(anchor, change + anchor, 1)
assert patched.count(marker) == 1
path.write_text(patched, encoding="utf-8", newline="\n")
print("SUCCESS: Ctrl+left-click selects inside groups using native Group Selection.")

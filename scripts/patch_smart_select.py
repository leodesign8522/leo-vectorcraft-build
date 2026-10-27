#!/usr/bin/env python3
"""Apply the Leo Design Smart Select patch to VectorCraft v0.17.0.

Usage: python scripts/patch_smart_select.py vectorcraft/crates/engine/src/tooling.rs
The script fails closed if upstream source no longer matches the expected anchors.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: python scripts/patch_smart_select.py PATH_TO_vectorcraft/crates/engine/src/tooling.rs")

tooling_path = Path(sys.argv[1])
tooling = tooling_path.read_text(encoding="utf-8")
tooling_marker = '        // Leo Design: Ctrl-click in Selection borrows Group Selection.'

if tooling_marker in tooling:
    raise SystemExit("Leo Design tooling patch is already applied; refusing to apply twice.")

tooling_anchor = '        if id == "directSelection" && mods.cmd && mods.alt {'
if tooling.count(tooling_anchor) != 1:
    raise SystemExit("Expected VectorCraft v0.17.0 tooling.rs layout not found; refusing to patch.")

tooling_change = (
    tooling_marker + '\n'
    '        // Ctrl+Alt retains the upstream Select Behind behaviour.\n'
    '        if id == "selection" && mods.cmd && !mods.alt {\n'
    '            return Some("groupSelection");\n'
    '        }\n'
)
tooling = tooling.replace(tooling_anchor, tooling_change + tooling_anchor, 1)
tooling_path.write_text(tooling, encoding="utf-8", newline="\n")

# After the temporary Group Selection click, keep that child as the black-arrow target.
# Otherwise SelectionTool resolves every nested hit to the group's top object on the next drag.
root = tooling_path.parents[3]
select_path = root / "crates/tools/src/select.rs"
select = select_path.read_text(encoding="utf-8")
select_marker = '                        // Leo Design: keep a selected group child as the black-arrow target.'
if select_marker in select:
    raise SystemExit("Leo Design select.rs patch is already applied; refusing to apply twice.")

select_anchor = '''                    Some(h) => {
                        let top = h.top_object(cx.isolation);
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
if select.count(select_anchor) != 1:
    raise SystemExit("Expected VectorCraft v0.17.0 select.rs hit-test block not found; refusing to patch.")

select_change = '''                    Some(h) => {
                        let top = h.top_object(cx.isolation);
                        // Leo Design: keep a selected group child as the black-arrow target.
                        // A normal click on another child still selects the whole group as usual.
                        let target = if h.leaf != top && cx.selection.contains(h.leaf) { h.leaf } else { top };
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
select = select.replace(select_anchor, select_change, 1)

# Scope replacements to the object-selection block only.
start = select.index(select_change)
end_anchor = '''                    }
                    None => {
                        self.state = State::Marquee'''
end = select.index(end_anchor, start)
block = select[start:end]
block = block.replace('cx.selection.contains(top)', 'cx.selection.contains(target)')
block = block.replace('deselect = Some(top)', 'deselect = Some(target)')
block = block.replace('"id": top.0', '"id": target.0')
block = block.replace('"ids": [top.0]', '"ids": [target.0]')
block = block.replace('key = Some(top)', 'key = Some(target)')
select = select[:start] + block + select[end:]
select_path.write_text(select, encoding="utf-8", newline="\n")
print("SUCCESS: Ctrl-click uses Group Selection, and the selected child remains movable with the black arrow.")

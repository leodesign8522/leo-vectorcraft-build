"""Persist the Leo snapping mode in normal VectorCraft preferences."""
from pathlib import Path
import sys

root = Path(sys.argv[1])


def edit(file, before, after):
    path = root / file
    source = path.read_text(encoding="utf-8")
    if source.count(before) != 1:
        raise SystemExit(f"Unexpected snapping preference anchor in {file}: {before}")
    path.write_text(source.replace(before, after, 1), encoding="utf-8", newline="\n")


edit("crates/engine/src/prefs.rs",
     "    pub smart_guide_color: String,",
     "    pub smart_guide_color: String,\n    pub leo_snapping: bool,")
edit("crates/engine/src/prefs.rs",
     '            smart_guide_color: s("#ff3dfc"),',
     '            smart_guide_color: s("#ff3dfc"),\n            leo_snapping: true,')
edit("crates/engine/src/cmd/prefscmds.rs",
     '    p!("smartGuideColor", "Smart Guides", "Display Options", "Color", color),',
     '    p!("leoSnapping", "Smart Guides", "", "Leo Design 吸附（关闭时使用官方吸附）", bool),\n'
     '    p!("smartGuideColor", "Smart Guides", "Display Options", "Color", color),')
edit("crates/tools/src/context.rs",
     "    pub smart_guides: bool,",
     "    pub smart_guides: bool,\n    /// Use the Leo Design geometry engine for supported drag gestures.\n    pub leo_snapping: bool,\n    /// Show ruler guides as rotation targets even when guides are locked.\n    pub leo_ruler_guides_visible: bool,")
edit("crates/engine/src/tooling.rs",
     "            smart_guides,\n            guides:",
     "            smart_guides,\n            leo_snapping: self.prefs.leo_snapping,\n            leo_ruler_guides_visible: view.guides,\n            guides:")
edit("crates/tools/src/lib.rs",
     "            smart_guides: true,\n            guides: true,",
     "            smart_guides: true,\n            leo_snapping: false,\n            leo_ruler_guides_visible: true,\n            guides: true,")
edit("crates/tools/src/lib.rs",
     "    pub fn cx<'a>(d: &'a Document, s: &'a Selection, p: &'a PaintDefaults) -> ToolContext<'a> {",
     """    pub fn leo_cx<'a>(d: &'a Document, s: &'a Selection, p: &'a PaintDefaults) -> ToolContext<'a> {
        ToolContext { leo_snapping: true, ..cx(d, s, p) }
    }

    pub fn cx<'a>(d: &'a Document, s: &'a Selection, p: &'a PaintDefaults) -> ToolContext<'a> {""")

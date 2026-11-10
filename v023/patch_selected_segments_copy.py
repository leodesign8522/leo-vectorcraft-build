#!/usr/bin/env python3
"""LC-027: Copy/Cut respects direct-selected segments and preserves native clipboard resources."""
from pathlib import Path
import sys

root = Path(sys.argv[1])

def change(rel, old, new):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    if s.count(old) != 1:
        raise SystemExit(f"FAIL LC-027 {rel}: anchor count={s.count(old)}")
    p.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")

change("crates/engine/src/cmd/edit.rs",
       "    let roots = if st.selection.is_empty() && !guides.is_empty() { vec![] } else { selected_roots(s)? };\n    let st = s.doc()?;\n    let mut clip = Clipboard::copy(st, &roots);",
       "    let roots = if st.selection.is_empty() && !guides.is_empty() { vec![] } else if st.selection.anchors.is_empty() { selected_roots(s)? } else { selected_roots_to_transform(s)? };\n    let st = s.doc()?;\n    let mut clip = Clipboard::copy_selected(st, &roots);")
change("crates/engine/src/cmd/clipboard.rs", "mod flavours;", "mod flavours;\nmod selected_segments;")
change("crates/engine/src/cmd/clipboard.rs",
       "    pub(crate) fn copy(st: &DocState, roots: &[NodeId]) -> Self {\n        let d = &*st.doc;",
       """    pub(crate) fn copy(st: &DocState, roots: &[NodeId]) -> Self {
        Self::copy_geometry(st, roots, false)
    }

    pub(crate) fn copy_selected(st: &DocState, roots: &[NodeId]) -> Self {
        Self::copy_geometry(st, roots, true)
    }

    fn copy_geometry(st: &DocState, roots: &[NodeId], partial: bool) -> Self {
        let d = &*st.doc;""")
change("crates/engine/src/cmd/clipboard.rs", "            nodes.push(n.clone());",
       """            let Some(mut copied) = (if partial { selected_segments::selected_node(n, &st.selection) } else { Some(n.clone()) }) else { continue };
            // A direct-selected compound member receives its compound's paint when detached.
            if partial && st.selection.anchors.contains_key(id) {
                if let Some(parent) = anc.iter().rev().skip(1).filter_map(|id| d.node(*id))
                    .find(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Compound { .. })) {
                    copied.appearance = parent.appearance.clone();
                }
            }
            nodes.push(copied);""")
target = root / "crates/engine/src/cmd/clipboard/selected_segments.rs"
target.write_text(Path(__file__).with_name("leo_selected_segments.rs").read_text(encoding="utf-8"), encoding="utf-8", newline="\n")
print("PASS LC-027 selected-segment clipboard extraction")
with (root / "crates/engine/src/tests/clipboard.rs").open("a", encoding="utf-8") as f:
    f.write(Path(__file__).with_name("selected_segments_clipboard_tests.rs").read_text(encoding="utf-8"))

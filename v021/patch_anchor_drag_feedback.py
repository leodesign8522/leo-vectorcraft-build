#!/usr/bin/env python3
"""Install the focused Direct Selection anchor guide on native VectorCraft."""
from pathlib import Path
import sys

root = Path(sys.argv[1]) / "crates/tools/src"


def edit(path, old, new):
    file = root / path
    text = file.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"Unexpected anchor-drag patch location: {path}: {old[:90]!r}")
    file.write_text(text.replace(old, new), encoding="utf-8", newline="\n")


targets = "guides/targets.rs"
edit(targets, "pub struct Targets {", "pub struct Targets {\n    pub(super) drag_guide: std::cell::Cell<Option<DragGuide>>,")
edit(targets, "use super::snap::", '#[path = "anchor_drag_feedback.rs"]\nmod anchor_drag_feedback;\nuse anchor_drag_feedback::DragGuide;\n\nuse super::snap::')
(root / "guides/anchor_drag_feedback.rs").write_text(
    Path(__file__).with_name("anchor_drag_feedback.rs").read_text(encoding="utf-8"),
    encoding="utf-8", newline="\n",
)
edit("guides/snap.rs", "impl PointSnap {", '''impl PointSnap {
    pub(crate) fn direct_anchor_drag(&self, cx: &ToolContext, p: Point, from: Option<&Leave>, shift: bool) -> (Point, Vec<Overlay>) {
        let Some((targets, tolerance)) = &self.targets else { return self.snap_from(cx, p, from) };
        if !cx.smart_guides || cx.snap_to_pixel || cx.snap_to_grid {
            targets.clear_drag_guide();
            return self.snap_from(cx, p, from);
        }
        if shift {
            targets.clear_drag_guide();
            let (q, mut overlays) = self.snap_from(cx, p, from);
            // A Shift constraint alone is not a snap.
            if !overlays.is_empty() {
                targets.color_drag_hit(q, &mut overlays);
            }
            return (q, overlays);
        }
        let acquire = cx.tol(tolerance.clamp(0.0, 8.0));
        targets.snap_direct_drag(p, from, cx.tol(*tolerance), acquire)
    }
''')
edit("direct.rs", "snap.snap_from(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()))",
     "snap.direct_anchor_drag(cx, grab + (p - start), shift.as_ref().or(self.line_end.as_ref()), ev.mods.shift)")
print("PASS native snapping with focused Direct Selection axis and extension feedback")

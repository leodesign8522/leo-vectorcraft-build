#!/usr/bin/env python3
"""Wire editable text outlines and path contacts into move snapping."""
from pathlib import Path
import sys

root = Path(sys.argv[1]) / "crates/tools/src"


def replace(path, old, new):
    source = path.read_text(encoding="utf-8")
    if source.count(old) != 1:
        raise SystemExit(f"Cannot patch {path.name}: {old[:100]!r}")
    path.write_text(source.replace(old, new, 1), encoding="utf-8", newline="\n")


targets = root / "guides/targets.rs"
select = root / "select.rs"
resolver = root / "guides/leo_snapping.rs"
snap = root / "guides/snap.rs"

replace(targets, "mod leo_rotation;\n", '''mod leo_rotation;
#[path = "leo_move_contact.rs"]
mod leo_move_contact;
''')
(targets.parent / "leo_move_contact.rs").write_text(
    Path(__file__).with_name("leo_move_contact.rs").read_text(encoding="utf-8"),
    encoding="utf-8",
    newline="\n",
)

replace(targets, "    pub(super) style: GuideStyle,", """    pub(super) style: GuideStyle,
    /// Maximum length of an ordinary alignment hint during a direct-selection drag.
    pub(super) compact_hint: Option<f64>,""")
replace(targets, """        let color = if (a.y-b.y).abs() <= 1e-6 {""", """        let a = match self.compact_hint {
            Some(limit) if a.distance(b) > limit => b + (a - b) * (limit / a.distance(b)),
            _ => a,
        };
        let color = if (a.y-b.y).abs() <= 1e-6 {""")
replace(targets, """    /// Its segments, and its anchors as lines too: a drawn point lines up with them.
    Drawing,""", """    /// Path and text-outline contacts for dragging a whole selection.
    MoveContacts,
    /// Its segments, and its anchors as lines too: a drawn point lines up with them.
    Drawing,""")
replace(targets, """        let mut t = Self::collect(cx.doc, exclude, None).styled(cx);""", """        Self::move_targets(cx, exclude, Gather::Bounds)
    }

    pub(crate) fn for_move_contacts(cx: &ToolContext, exclude: &[NodeId]) -> Self {
        Self::move_targets(cx, exclude, Gather::MoveContacts)
    }

    fn move_targets(cx: &ToolContext, exclude: &[NodeId], gather: Gather) -> Self {
        let mut t = Self::collect_inner(cx.doc, exclude, None, None, gather).styled(cx);""")
replace(targets, """            let Some(b) = n.geometric_bounds() else { return };
            if let Some(v) = visible""", """            let ink = matches!(gather, Gather::Segments | Gather::MoveContacts)
                .then(|| Self::text_outline(doc, n)).flatten();
            let Some(b) = ink.as_ref().map(Shape::bounding_box).or_else(|| n.geometric_bounds()) else { return };
            if let Some(v) = visible""")
replace(targets, """            add_rect(&mut t, b, Kind::Edge);""", """            if let Some(ink) = ink {
                let count = budget.min(segment_budget);
                for seg in ink.segments().take(count) {
                    t.points.push((seg.start(), Kind::Anchor));
                    t.segments.push((seg.bounding_box(), seg));
                    budget -= 1;
                    segment_budget -= 1;
                }
            }
            add_rect(&mut t, b, Kind::Edge);""")
replace(targets, """
    /// Is a target point (an anchor, a centre) within `tol` of `p`?
    pub(crate) fn has_point_near(&self, p: Point, tol: f64) -> bool {
        self.nearest_point(p, tol).is_some()
    }
""", "")

replace(select, """    bounds: Option<Rect>,
    targets: Option<Targets>,""", """    bounds: Option<Rect>,
    text_ink: Option<Rect>,
    targets: Option<Targets>,""")
replace(select, """        Self {
            bounds: selection_bounds(cx),
            targets: guides.then(|| Targets::for_move(cx, &cx.selection.objects)),
            points: points(&cx.selection.objects),
            copy_targets: guides.then(|| Targets::for_move(cx, &[])),""", """        let text_ink = if guides && let [id] = cx.selection.objects.as_slice() {
            cx.doc.node(*id).and_then(|node| Targets::text_ink_bounds(cx.doc, node))
        } else {
            None
        };
        Self {
            bounds: selection_bounds(cx),
            text_ink,
            targets: guides.then(|| Targets::for_move_contacts(cx, &cx.selection.objects)),
            points: points(&cx.selection.objects),
            copy_targets: guides.then(|| Targets::for_move_contacts(cx, &[])),""")
replace(select, "Self { bounds, targets: None, points: None, copy_targets: None, copy_points: None, rulers: None }",
        "Self { bounds, text_ink: None, targets: None, points: None, copy_targets: None, copy_points: None, rulers: None }")
replace(select, """        if let (Some(t), Some(b)) = (targets, self.bounds) {
            // The point the selection was grabbed by, near another object's anchor or centre,
            // lands on it, rather than the bounds lining up a little beside it (#955).
            if t.has_point_near(start + d, cx.snap_tol()) {
                let (q, ov) = t.snap_point(start + d, cx.snap_tol());""", """        if let (Some(t), Some(b)) = (targets, self.text_ink.or(self.bounds)) {
            // Prefer contact with real geometry near the grabbed point over axis alignment.
            if let Some((q, ov)) = t.move_contact(start + d, cx.snap_tol(), None) {""")
replace(select, """        if let (Some(t), Some(b)) = (targets, self.bounds) {
            if t.has_point_near(start + d, cx.snap_tol()) {
                let (q, ov) = t.snap_point_along(start + d, u, cx.snap_tol());""", """        if let (Some(t), Some(b)) = (targets, self.text_ink.or(self.bounds)) {
            if let Some((q, ov)) = t.move_contact(start + d, cx.snap_tol(), Some(u)) {""")
replace(select, """    cx.doc.node(h.leaf).and_then(|n| n.path_data()).and_then(|pd| hold_on_path(pd, p, reach)).unwrap_or(p)""", """    cx.doc.node(h.leaf).and_then(|node| {
        if let Some(path) = node.path_data() {
            return hold_on_path(path, p, reach);
        }
        if (cx.smart_guides || cx.snap_to_point) && !cx.snap_to_grid && !cx.snap_to_pixel {
            return Targets::hold_text_outline(cx.doc, node, p, cx.tol(HOVER_REACH));
        }
        None
    }).unwrap_or(p)""")

replace(snap, """        Self::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection).with_visible_rulers(cx))""", """        Self::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection)
            .with_visible_rulers(cx).compact_move_hints(cx))""")
replace(resolver, """                    let mut ov: Vec<Overlay> = self.line(f.at, hit).into_iter().collect();
                    if kind == "intersect" { ov.extend(self.label(hit, "intersect")); }""",
        """                    let mut ov: Vec<Overlay> = self.line(f.at, hit).into_iter().collect();
                    if kind == "intersect" { ov.extend(self.contact_hint(hit, "intersect")); }""")
replace(resolver, """        if let Some(q) = self.crossing(p, tol) {
            return (q, self.label(q, "intersect").into_iter().collect());
        }""", """        if let Some(q) = self.crossing(p, tol) {
            return (q, self.contact_hint(q, "intersect"));
        }""")
replace(resolver, """            return (q, self.label(q, Kind::Edge.label()).into_iter().collect());""",
        """            return (q, self.contact_hint(q, Kind::Edge.label()));""")
replace(resolver, """                return (q, self.label(q, kind.label()).into_iter().collect());""",
        """                return (q, self.contact_hint(q, kind.label()));""")
replace(resolver, """            self.leo_point_lock.set(Some((q, kind)));
            return (q, self.label(q, kind.label()).into_iter().collect());""", """            self.leo_point_lock.set(Some((q, kind)));
            return (q, self.contact_hint(q, kind.label()));""")

print("PASS editable glyph outline and real path move contacts")

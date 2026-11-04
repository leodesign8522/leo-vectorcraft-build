#!/usr/bin/env python3
"""Leo Design Smart Select + Group Anchor Move for official VectorCraft v0.21.0.

Brings the previously tested v0.17 gesture patches across the source split.
Fail-closed: every old/new anchor is counted, never silently edited.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_smart_interactions.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def rep(path, a, b, title):
    p = root/path
    s = p.read_text(encoding="utf-8")
    count = s.count(a)
    if count != 1:
        raise SystemExit(f"FAIL {title}: {path} anchor count={count}")
    p.write_text(s.replace(a,b,1), encoding="utf-8", newline="\n")
    print("OK", title)

direct="crates/tools/src/direct.rs"
rep(direct,
    '    /// Dragging a curved segment: it follows the pointer',
    '''    /// Leo Design: grab a descendant path anchor and move the selected
    /// parent group as a rigid object, preserving relative node geometry.
    MoveGroupAnchor { start: Point, grab: Point, began: bool },
    /// Dragging a curved segment: it follows the pointer''',
    "group-anchor drag state"
)
rep(direct,
    '''                if let Some((id, si, ai, grab)) = hit_anchor(cx, p, point) {
                    return self.press_anchor(cx, ev, p, (id, si, ai, grab));''',
    '''                if let Some((id, si, ai, grab)) = hit_anchor(cx, p, point) {
                    if ev.mods.cmd && !ev.mods.shift && !ev.mods.alt
                        && cx.selection.objects.len() == 1
                        && let Some(selected) = cx.selection.objects.first().copied()
                        && selected != id
                        && cx.doc.node(selected).is_some_and(|n| n.children().is_some())
                        && cx.doc.ancestry(id).is_some_and(|anc| anc.contains(&selected))
                    {
                        self.state = State::MoveGroupAnchor { start: p, grab, began: false };
                        return vec![];
                    }
                    return self.press_anchor(cx, ev, p, (id, si, ai, grab));''',
    "Ctrl+anchor on selected group starts rigid group drag"
)
rep(direct,
    '''            (PointerKind::Drag, State::MoveObject { start, grab, hold, began, .. }) => self.drag_move_object(cx, ev, p, (start, grab, hold, began)),''',
    '''            (PointerKind::Drag, State::MoveGroupAnchor { start, grab, began }) => {
                let mut out = vec![];
                if !began {
                    if p.distance(start) < cx.tol(3.0) {
                        return out;
                    }
                    out.push(Action::Begin("Move Group by Anchor".into()));
                    self.state = State::MoveGroupAnchor { start, grab, began: true };
                    self.anchor_snap = Some(PointSnap::new(cx, || Targets::for_anchor_drag(cx.doc, cx.selection)));
                }
                let mut d = move_delta(start, p, false, cx.constrain_angle);
                let unsnapped = d;
                self.guides.clear();
                if let Some(snap) = &self.anchor_snap {
                    let (q, guides) = snap.snap_from(cx, grab + (p - start), None);
                    (d, self.guides) = (q - grab, guides);
                }
                self.snapped = snapped(d, unsnapped, false, &self.guides);
                out.push(Action::Preview("object.transform".into(), json!({
                    "matrix": matrix_json(vectorcraft_geom::Affine::translate(d)),
                    "copy": false
                })));
                out
            }
            (PointerKind::Drag, State::MoveObject { start, grab, hold, began, .. }) => self.drag_move_object(cx, ev, p, (start, grab, hold, began)),''',
    "rigid preview and snap to another object anchor"
)
rep(direct,
    '''                | State::MoveObject { began, .. }
                | State::Segment { began, .. }''',
    '''                | State::MoveObject { began, .. }
                | State::MoveGroupAnchor { began, .. }
                | State::Segment { began, .. }''',
    "commit grouped-anchor drag"
)
rep(direct,
    '''                if matches!(self.state, State::Handle { .. } | State::MoveAnchors { .. } | State::MoveObject { .. }) {''',
    '''                if matches!(self.state, State::Handle { .. } | State::MoveAnchors { .. } | State::MoveObject { .. } | State::MoveGroupAnchor { .. }) {''',
    "show Smart Guide overlays while moving parent group"
)
rep(direct,
    '''                | State::MoveAnchors { began: true, .. }
                | State::MoveObject { began: true, .. } => {''',
    '''                | State::MoveAnchors { began: true, .. }
                | State::MoveObject { began: true, .. }
                | State::MoveGroupAnchor { began: true, .. } => {''',
    "snapped cursor for group-anchor drag"
)
print("SUCCESS Leo Design Selection & Group Anchor Move port to v0.21")



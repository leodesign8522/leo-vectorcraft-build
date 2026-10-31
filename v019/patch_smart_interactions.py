#!/usr/bin/env python3
"""Leo Design Smart Select + Group Anchor Move for official VectorCraft v0.19.0.

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

rep(
    "crates/engine/src/tooling.rs",
    '        if id == "directSelection" && mods.cmd && mods.alt {',
    '''        // Leo Design: Ctrl while using the Selection tool temporarily
        // borrows Group Selection, including while dragging child objects.
        // Ctrl+Alt is reserved for the normal Select Behind behavior.
        if id == "selection" && mods.cmd && !mods.alt {
            return Some("groupSelection");
        }
        if id == "directSelection" && mods.cmd && mods.alt {''',
    "Ctrl+click temporarily activates Group Selection",
)
file = root/"crates/tools/src/select.rs"
s = file.read_text(encoding="utf-8")
a = '''                    Some(h) => {
                        let top = h.top_object(scope);
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
b = '''                    Some(h) => {
                        let top = h.top_object(scope);
                        // Leo Design: after Ctrl-click selects a nested group child,
                        // keep that exact leaf as the black-arrow drag target.
                        let target = if h.leaf != top && cx.selection.contains(h.leaf) {
                            h.leaf
                        } else {
                            top
                        };
                        let mut out = vec![];
                        let (mut deselect, mut key) = (None, None);'''
if s.count(a)!=1: raise SystemExit(f"FAIL Selection nested target anchor count={s.count(a)}")
s=s.replace(a,b,1)
start=s.index(b)
end_anchor = '''                    }
                    None => {
                        self.press_nothing'''
if end_anchor not in s[start:]: raise SystemExit("FAIL Selection marquee anchor")
end=s.index(end_anchor,start)
block=s[start:end]
needles=['cx.selection.contains(top)', 'deselect = Some(top)', '"id": top.0', '"ids": [top.0]', 'key = Some(top)']
# Ensure we did not accidentally eliminate all key matches in revised upstream.
if not any(x in block for x in needles):raise SystemExit("FAIL selection target references disappeared")
for old,new in [
 ('cx.selection.contains(top)', 'cx.selection.contains(target)'),
 ('deselect = Some(top)', 'deselect = Some(target)'),
 ('"id": top.0', '"id": target.0'),
 ('"ids": [top.0]', '"ids": [target.0]'),
 ('key = Some(top)', 'key = Some(target)'),
]:
    block=block.replace(old,new)
s=s[:start]+block+s[end:]
file.write_text(s,encoding="utf-8",newline="\n")
print("OK selected nested child can be dragged without ungrouping")

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
print("SUCCESS Leo Design Selection & Group Anchor Move port to v0.19")

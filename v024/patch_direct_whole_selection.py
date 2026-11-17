"""Preserve a selected container during Direct Selection drags of its art."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
p=root/'crates/tools/src/direct.rs'
s=p.read_text(encoding='utf-8')
def edit(old,new):
    global s
    assert s.count(old)==1,(old[:100],s.count(old))
    s=s.replace(old,new)
edit('    click_segment: Option<(NodeId, usize, usize)>,',
     '    click_segment: Option<(NodeId, usize, usize)>,\n    /// A fill click drills into a selected group only after a no-drag release.\n    leo_click_leaf: Option<NodeId>,')
edit('            click_segment: None,','            click_segment: None,\n            leo_click_leaf: None,')
edit('                self.click_segment = None;', '                self.click_segment = None;\n                self.leo_click_leaf = None;')
edit('fn nearest_anchor(pd:', '''/// Descendants of an entirely selected container move with that container. A
/// later click can still drill down to a leaf or a single anchor.
fn leo_whole_ancestor(cx: &ToolContext, id: NodeId) -> bool {
    cx.doc.ancestry(id).is_some_and(|ids| ids.into_iter().any(|owner| {
        owner != id && cx.selection.contains(owner)
            && cx.selection.partial(owner).is_none()
            && !cx.selection.segments.contains_key(&owner)
            && cx.doc.node(owner).is_some_and(|n| !n.is_layer())
            && in_scope(cx.doc, cx.isolation, owner)
    }))
}

fn nearest_anchor(pd:''')
edit('            let whole = cx.selection.contains(id) && cx.selection.partial(id).is_none();',
     '            let whole = (cx.selection.contains(id) && cx.selection.partial(id).is_none()) || leo_whole_ancestor(cx, id);')
edit('            cx.selection.contains(id) && !cx.selection.segments.contains_key(&id) && cx.selection.partial(id).is_none_or(|s| s.contains(&(si, ai)));',
     '            (cx.selection.contains(id) && !cx.selection.segments.contains_key(&id) && cx.selection.partial(id).is_none_or(|s| s.contains(&(si, ai))))\n                || leo_whole_ancestor(cx, id);')
edit('        // On a gradient mesh a click picks the point,', '''        if !self.group && !ev.mods.shift && leo_whole_ancestor(cx, h.leaf)
            && cx.doc.node(h.leaf).is_some_and(|n| n.path_data().is_some())
        {
            self.state = State::MoveObject { start: p, grab: None, hold: hold_point(cx, &h, p), began: false, deselect: None };
            if ev.mods.alt { self.click_group = next_group(cx, &h.ancestry); }
            else {
                if matches!(h.kind, HitKind::Stroke | HitKind::Outline) {
                    if let Some(seg) = segment_at(cx, h.leaf, p) { self.click_segment = Some((h.leaf, seg.si, seg.seg)); }
                } else { self.leo_click_leaf = Some(h.leaf); }
            }
            return vec![];
        }
        // On a gradient mesh a click picks the point,''')
edit('        let group = self.click_group.take();', '''        let leaf = self.leo_click_leaf.take();
        if !began && let Some(id) = leaf {
            return vec![Action::Exec("select.set".into(), json!({"ids": [id.0]}))];
        }
        let group = self.click_group.take();''')
p.write_text(s,encoding='utf-8',newline='\n')
with (root/'crates/engine/src/tests/clipboard.rs').open('a',encoding='utf-8') as f:
    f.write('\n'+(Path(__file__).parent.parent / 'v023' / 'whole_selection_regressions.rs').read_text(encoding='utf-8'))
print('Applied Direct Selection whole-group drag preservation')

"""Minimal native handle marquee and batch transformations, after multi_handles."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
p = root / 'crates/tools/src/direct.rs'
s = p.read_text(encoding='utf-8')
def edit(a, b):
    global s
    if s.count(a) != 1:
        raise SystemExit('Unexpected handle operations anchor: ' + a)
    s = s.replace(a, b, 1)

edit('    leo_handle_dragged: bool,', '    leo_handle_dragged: bool,\n    leo_marquee: Option<(Point, Point, bool)>,')
edit('            leo_handle_dragged: false,', '            leo_handle_dragged: false,\n            leo_marquee: None,')
edit('        !matches!(self.state, State::Idle) || self.guide.busy()',
     '        self.leo_marquee.is_some() || !matches!(self.state, State::Idle) || self.guide.busy()')
edit('        if key == ToolKey::Escape {', '''        // L equalizes to the first picked handle; S mirrors each picked handle.
        if !self.group && self.leo_marquee.is_none() && matches!(self.state, State::Idle) && !self.leo_handles.is_empty()
            && self.leo_handle_revision == cx.revision.1
            && matches!(key, ToolKey::Letter('l' | 's')) && !mods.cmd && !mods.alt {
            let mut handles = Vec::new();
            let mut length = None;
            for (key, old) in &self.leo_handles {
                let Some(q) = leo_handle_position(cx, *key) else { return vec![]; };
                if q.distance(*old) > 1e-6 { return vec![]; }
                let Some((_, _, _, a)) = handle_anchors(cx).into_iter()
                    .find(|(id,si,ai,_)| (*id,*si,*ai)==(key.0,key.1,key.2)) else { return vec![]; };
                let n = q.distance(a.p);
                let target = *length.get_or_insert(n);
                // Symmetry is deterministic when both sides are selected: the
                // first picked side of each anchor is the reference.
                handles.push((key, a.p, q, target / n));
            }
            let symmetry = key == ToolKey::Letter('s');
            let mut changes = Vec::new();
            let mut seen = BTreeSet::new();
            for (k, anchor, q, scale) in handles {
                if symmetry && !seen.insert((k.0,k.1,k.2)) { continue; }
                let q = if symmetry { q } else { anchor + (q-anchor)*scale };
                changes.push(json!({"id":k.0.0,"subpath":k.1,"anchor":k.2,"which":if k.3 {"out"} else {"in"},"x":q.x,"y":q.y}));
                if symmetry {
                    let opposite = anchor + (anchor-q);
                    changes.push(json!({"id":k.0.0,"subpath":k.1,"anchor":k.2,"which":if k.3 {"in"} else {"out"},"x":opposite.x,"y":opposite.y}));
                }
            }
            self.leo_handles.clear();
            return vec![Action::Exec("path.leoSetHandles".into(),json!({"handles":changes,"independent":!symmetry}))];
        }
        if key == ToolKey::Escape {
            self.leo_marquee = None;''')
edit('        (matches!(self.state, State::Corner(_))', '''        (!self.group && matches!(self.state, State::Idle) && self.leo_handle_revision == cx.revision.1
            && !self.leo_handles.is_empty() && matches!(key, ToolKey::Letter('l' | 's')))
            || (matches!(self.state, State::Corner(_))''')
edit('        self.bracket_hover = (ev.kind == PointerKind::Move).then_some(ev.pos);', '''        // Ctrl+Alt drag selects visible artwork handles only. Shift toggles.
        if !self.group && ev.kind == PointerKind::Down && ev.mods.cmd && ev.mods.alt
            && matches!(self.state, State::Idle) {
            if self.leo_handle_revision != cx.revision.1 { self.leo_handles.clear(); }
            self.leo_marquee = Some((ev.pos,ev.pos,ev.mods.shift));
            self.leo_handle_hover = None;
            return vec![];
        }
        if let Some((start,_,toggle)) = self.leo_marquee {
            if ev.kind == PointerKind::Drag { self.leo_marquee = Some((start,ev.pos,toggle)); }
            if ev.kind == PointerKind::Up {
                if !toggle { self.leo_handles.clear(); }
                let rect = Rect::from_points(start,ev.pos);
                for (id,si,ai,a) in handle_anchors(cx) {
                    if !cx.doc.node(id).is_some_and(|n|matches!(&n.kind,NodeKind::Path{guide:false,..})) {continue;}
                    for (side,q) in [(false,a.h_in),(true,a.h_out)] {
                        let key=(id,si,ai,side);
                        if rect.contains(q) && leo_handle_position(cx,key).is_some() {
                            if toggle && self.leo_handles.iter().any(|(k,_)|*k==key) {
                                self.leo_handles.retain(|(k,_)|*k!=key);
                            } else { self.leo_handles.push((key,q)); }
                        }
                    }
                }
                self.leo_handle_revision=cx.revision.1;
                self.leo_marquee=None;
            }
            return vec![];
        }
        self.bracket_hover = (ev.kind == PointerKind::Move).then_some(ev.pos);''')
edit('    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {', '''    fn overlays(&self, cx: &ToolContext) -> Vec<Overlay> {
        if let Some((start,cur,_)) = self.leo_marquee { return vec![Overlay::Marquee(Rect::from_points(start,cur))]; }''')
p.write_text(s,encoding='utf-8',newline='\n')
with (root/'crates/tools/src/direct/tests.rs').open('a',encoding='utf-8') as f:
    f.write((Path(__file__).parent/'handle_operations_tests.rs').read_text(encoding='utf-8'))

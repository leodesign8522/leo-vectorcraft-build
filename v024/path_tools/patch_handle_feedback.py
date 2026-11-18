"""Native Direct Selection handle hover and persistent selection feedback.
Apply after the v0.24 migration; fail if upstream anchors changed.
"""
from pathlib import Path
import sys

root = Path(sys.argv[1])
file = root / "crates/tools/src/direct.rs"
s = file.read_text(encoding="utf-8")

def edit(before, after):
    global s
    if s.count(before) != 1:
        raise SystemExit(f"Unexpected Path Tools integration anchor: {before}")
    s = s.replace(before, after, 1)

edit("    bracket_hover: Option<Point>,",
     """    bracket_hover: Option<Point>,
    leo_handle_hover: Option<(NodeId, usize, usize, bool)>,
    // Last explicitly picked handle; use its position and selection revision to
    // avoid retaining a marker for a different anchor after a topology edit.
    leo_handle_selected: Option<((NodeId, usize, usize, bool), Point, u64)>,""")
edit("            bracket_hover: None,",
     """            bracket_hover: None,
            leo_handle_hover: None,
            leo_handle_selected: None,""")
edit("fn anchors_json(v: &[AnchorRef]) -> Value {",
     """// Resolve only visible, selectable, nonzero handles, with the same
// visibility rules as the actual native handle hit test.
fn leo_handle_position(cx: &ToolContext, key: (NodeId, usize, usize, bool)) -> Option<Point> {
    let (id, si, ai, out) = key;
    let editable = cx.doc.editable();
    if !editable(id) || !in_scope(cx.doc, cx.isolation, id) { return None; }
    let (_, _, _, a) = handle_anchors(cx).into_iter().find(|(i,s,a,_)| (*i,*s,*a)==(id,si,ai))?;
    let q = if out { a.h_out } else { a.h_in };
    (q.distance(a.p)>1e-6).then_some(q)
}

fn leo_handle_marker(out: &mut Vec<Overlay>, p: Point, selected: bool) {
    // Contrasting outer ring, independent of the layer color. Different
    // fill/size distinguish hover from selection, even without color.
    out.push(Overlay::CurvePoint {p, color:[0x20,0x24,0x2b], filled:false, size:if selected {15.0} else {12.0}});
    out.push(Overlay::CurvePoint {p, color:if selected {[0x00,0xc8,0xff]} else {[0xff,0xd5,0x40]}, filled:selected, size:if selected {10.0} else {9.0}});
}

fn anchors_json(v: &[AnchorRef]) -> Value {""")
edit("        self.bracket_hover = None;\n    }\n\n    fn id",
     "        self.bracket_hover = None;\n        self.leo_handle_hover = None;\n    }\n\n    fn id")
edit("        // Up and Down while a corner widget is pressed:",
     """        if key == ToolKey::Escape {
            self.leo_handle_selected = None;
            self.leo_handle_hover = None;
        }
        // Up and Down while a corner widget is pressed:""")
edit("        self.bracket_hover = (ev.kind == PointerKind::Move).then_some(ev.pos);",
     """        self.bracket_hover = (ev.kind == PointerKind::Move).then_some(ev.pos);
        if matches!(ev.kind, PointerKind::Down | PointerKind::DoubleClick) {
            self.leo_handle_selected = None;
            self.leo_handle_hover = None;
        }
        if ev.kind == PointerKind::Move {
            self.leo_handle_hover = if !self.group && matches!(self.state, State::Idle) {
                hit_handle(cx, ev.pos, cx.point_tol()).filter(|&key| leo_handle_position(cx,key).is_some())
            } else { None };
        }""")
edit("            (PointerKind::Up, State::Handle { .. } | State::SpineHandle { .. }) => {",
     """            (PointerKind::Up, State::Handle { id, si, ai, out }) => {
                let key = (id,si,ai,out);
                self.leo_handle_selected = leo_handle_position(cx,key).map(|p|(key,p,cx.revision.1));
                self.state = State::Idle;
                self.guides.clear();
                vec![Action::Commit]
            }
            (PointerKind::Up, State::SpineHandle { .. }) => {""")
edit("                out.extend(self.guide.overlays(cx));\n                out",
     """                out.extend(self.guide.overlays(cx));
                if !self.group {
                    let active = match self.state {
                        State::Handle { id, si, ai, out } => leo_handle_position(cx,(id,si,ai,out)).map(|p|((id,si,ai,out),p)),
                        State::Idle => self.leo_handle_selected.and_then(|(key,old,revision)| {
                            if revision != cx.revision.1 { return None; }
                            let p=leo_handle_position(cx,key)?;
                            (p.distance(old)<1e-6).then_some((key,p))
                        }),
                        _ => None,
                    };
                    if matches!(self.state, State::Idle) {
                        if let Some(key)=self.leo_handle_hover.filter(|k|active.is_none_or(|(a,_)| a!=*k)) {
                            if let Some(p)=leo_handle_position(cx,key) { leo_handle_marker(&mut out,p,false); }
                        }
                    }
                    if let Some((_,p))=active { leo_handle_marker(&mut out,p,true); }
                }
                out""")
file.write_text(s, encoding="utf-8", newline="\n")
tests = root / "crates/tools/src/direct/tests.rs"
with tests.open("a", encoding="utf-8") as f:
    f.write((Path(__file__).parent / "handle_feedback_tests.rs").read_text(encoding="utf-8"))

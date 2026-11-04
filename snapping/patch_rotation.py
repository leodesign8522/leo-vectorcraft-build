#!/usr/bin/env python3
"""Rotation-specific geometry and feedback; this adapter is never loaded by Core."""
from pathlib import Path
import sys
root = Path(sys.argv[1])

def change(rel, old, new):
    p = root / rel
    s = p.read_text(encoding='utf-8')
    if s.count(old) != 1:
        raise SystemExit(f'Unexpected rotation patch anchor: {rel}: {old[:90]}')
    p.write_text(s.replace(old, new, 1), encoding='utf-8', newline='\n')

targets = 'crates/tools/src/guides/targets.rs'
change(targets, 'pub struct Targets {\n',
       'pub struct Targets {\n    pub(super) leo_rotation_lock: std::cell::Cell<Option<(Point, f64, Point, bool)>>,\n')
change(targets, 'mod leo_snapping;\n', 'mod leo_snapping;\n#[path = "leo_rotation.rs"]\nmod leo_rotation;\n')
(root / 'crates/tools/src/guides/leo_rotation.rs').write_text(
    Path(__file__).with_name('leo_rotation.rs').read_text(encoding='utf-8'), encoding='utf-8', newline='\n')

change('crates/tools/src/guides/snap.rs', 'impl PointSnap {\n', '''impl PointSnap {
    pub fn for_rotation(cx: &ToolContext) -> Self {
        Self::new(cx, || Targets::leo_for_rotation(cx))
    }

    /// Rotate the actual grabbed anchor/corner along its circle, never the free mouse position.
    pub fn snap_rotation(&self, cx: &ToolContext, origin: Point, from: Point, pointer: Point, grab: Point, shift: bool)
        -> (Affine, f64, Vec<Overlay>) {
        let (raw, degrees) = crate::bbox::rotate_for_drag(origin, from, pointer, shift);
        let Some((targets, tolerance)) = &self.targets else { return (raw, degrees, vec![]) };
        if shift || !(cx.smart_guides || cx.snap_to_point) || (from - origin).hypot() <= 1e-9 {
            targets.leo_clear_rotation();
            return (raw, degrees, vec![]);
        }
        let moved = raw * grab;
        let radius = grab.distance(origin);
        let Some((hit, overlays)) = targets.leo_rotation_hit(origin, radius, moved, cx.tol(*tolerance), cx.smart_guides) else {
            return (raw, degrees, vec![]);
        };
        let (a, b) = (moved - origin, hit - origin);
        let correction = a.cross(b).atan2(a.dot(b));
        let turn = Affine::translate(origin.to_vec2()) * Affine::rotate(correction) * Affine::translate(-origin.to_vec2());
        (turn * raw, degrees - correction.to_degrees(), overlays)
    }

''')

transform = 'crates/tools/src/xform/transform.rs'
change(transform,
       'self.snap = Some(PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)));',
       'self.snap = Some(if self.kind == TransformKind::Rotate { PointSnap::for_rotation(cx) } else { PointSnap::new(cx, || Targets::collect(cx.doc, &cx.selection.objects, None)) });')
change(transform, '''                let (p, guides) = self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p));
                self.hover = guides;
                let (m, text) = self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds);''', '''                let (m, text) = if self.kind == TransformKind::Rotate {
                    let (m, deg, guides) = self.snap.as_ref().unwrap().snap_rotation(cx, d.origin, d.pick, p, d.pick, ev.mods.shift);
                    self.hover = guides;
                    (m, format!("{:.1}°", -deg))
                } else {
                    let (p, guides) = self.snap.as_ref().map_or((p, vec![]), |s| s.snap(cx, p));
                    self.hover = guides;
                    self.matrix_for(d.origin, d.pick, p, ev.mods.shift, d.bounds)
                };''')
change(transform, '''        if let (Some(d), TransformKind::Reflect | TransformKind::Rotate) = (self.drag, self.kind)
            && d.began''', '''        if let (Some(d), TransformKind::Reflect | TransformKind::Rotate) = (self.drag, self.kind)
            && !(self.kind == TransformKind::Rotate && !self.hover.is_empty())
            && d.began''')

select = 'crates/tools/src/select.rs'
change(select, '''    Rotating {
        center: Point,
        start: Point,
    },''', '''    Rotating {
        center: Point,
        start: Point,
        grab: Point,
    },''')
change(select, 'self.state = State::Rotating { center: bx.center(), start: p };', '''self.targets = Some(PointSnap::for_rotation(cx));
                    self.guides.clear();
                    let grab = bx.corners().into_iter().min_by(|a, b| a.distance(p).total_cmp(&b.distance(p))).unwrap();
                    self.state = State::Rotating { center: bx.center(), start: p, grab };''')
change(select, '(PointerKind::Drag, State::Rotating { center, start, .. }) => {',
       '(PointerKind::Drag, State::Rotating { center, start, grab }) => {')
change(select, '                let (a, deg) = rotate_for_drag(center, start, p, m.shift);', '''                let (a, deg, guides) = self.targets.as_ref().map_or_else(|| {
                    let (a, deg) = rotate_for_drag(center, start, p, m.shift);
                    (a, deg, vec![])
                }, |snap| snap.snap_rotation(cx, center, start, p, grab, m.shift));
                self.guides = guides;''')

free = 'crates/tools/src/xform/free.rs'
change(free, 'self.targets = op.handle().is_some().then(|| PointSnap::new(cx, || Targets::for_scale(cx)));',
       'self.targets = if matches!(op, Op::Rotate) { Some(PointSnap::for_rotation(cx)) } else { op.handle().is_some().then(|| PointSnap::new(cx, || Targets::for_scale(cx))) };')
change(free, '                let (a, deg) = rotate_for_drag(d.rect.center(), d.start, p, shift);', '''                let grab = rect_corners(d.rect).into_iter().min_by(|a, b| a.distance(d.start).total_cmp(&b.distance(d.start))).unwrap();
                let (a, deg, guides) = self.targets.as_ref().map_or_else(|| {
                    let (a, deg) = rotate_for_drag(d.rect.center(), d.start, p, shift);
                    (a, deg, vec![])
                }, |snap| snap.snap_rotation(cx, d.rect.center(), d.start, p, grab, shift));
                self.guides = guides;''')

with (root / 'crates/tools/src/xform/transform.rs').open('a', encoding='utf-8') as f:
    f.write(Path(__file__).with_name('leo_rotation_tool_tests.rs').read_text(encoding='utf-8'))
print('PASS independent rotation edge snapping for Rotate, Selection and Free Transform')

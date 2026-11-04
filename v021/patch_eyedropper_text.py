#!/usr/bin/env python3
"""Copy character dimensions in the same effective units as the Character panel."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
path = root / 'crates/engine/src/cmd/appearance.rs'
source = path.read_text(encoding='utf-8')

def change(old, new):
    global source
    if source.count(old) != 1:
        raise SystemExit(f'Unexpected Eyedropper patch anchor: {old[:100]}')
    source = source.replace(old, new, 1)

change('    let picked = Picked::of(&node);', '''    let picked = Picked::of(&node);
    // Validate every destination before touching paint defaults, samples, or undo history.
    let character_styles = picked.local_character_styles(&node, &s.doc()?.doc, &targets, attrs.character && !append)?;''')
change('picked.run(st, &placed, &attrs)', 'picked.run(st, &placed, &attrs, character_styles.get(&id))')
change('picked.apply(n, &attrs, append);', 'picked.apply(n, &attrs, append, character_styles.get(id));')
change('    fn run(&self, st: &mut CharStyle, placed: &Appearance, a: &EyedropperAttrs) {\n        if let Some((cs, _)) = self.text.as_ref().filter(|_| a.character) {',
       '''    fn run(&self, st: &mut CharStyle, placed: &Appearance, a: &EyedropperAttrs, character: Option<&CharStyle>) {
        if let Some(cs) = character.filter(|_| a.character) {''')
change('    fn apply(&self, n: &mut Node, a: &EyedropperAttrs, append: bool) {',
       '    fn apply(&self, n: &mut Node, a: &EyedropperAttrs, append: bool, character: Option<&CharStyle>) {')
change('self.run(&mut r.style, &placed, a);', 'self.run(&mut r.style, &placed, a, character);')

helper = '''    /// Copy what the Character panel shows, compensating each destination's own affine.
    /// The object transform, origin, orientation, contents and unselected runs stay intact.
    fn local_character_styles(
        &self,
        source: &Node,
        doc: &vectorcraft_doc::Document,
        targets: &[NodeId],
        enabled: bool,
    ) -> Result<std::collections::BTreeMap<NodeId, CharStyle>> {
        let mut out = std::collections::BTreeMap::new();
        let Some((style, _)) = self.text.as_ref().filter(|_| enabled) else { return Ok(out) };
        if !targets.iter().any(|id| doc.node(*id).is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))) {
            return Ok(out);
        }
        let NodeKind::Text(source) = &source.kind else { return Ok(out) };
        let effective = source.effective_char_style(style)
            .ok_or_else(|| bad("appearance.copyFrom", "sampled text transform is collapsed or unrepresentable"))?;
        for id in targets {
            let Some(NodeKind::Text(target)) = doc.node(*id).map(|n| &n.kind) else { continue };
            let scale = target.style_scale()
                .ok_or_else(|| bad("appearance.copyFrom", "destination text transform is collapsed or unrepresentable"))?;
            let mut local = effective.clone();
            local.size /= scale.points;
            local.leading = local.leading.map(|v| v / scale.points);
            local.baseline_shift /= scale.points;
            local.tcy_up_down /= scale.points;
            local.tcy_left_right /= scale.points;
            local.h_scale /= scale.horizontal;
            if local.size <= 0.0 || local.h_scale <= 0.0 ||
                [local.size, local.leading.unwrap_or(0.0), local.baseline_shift,
                 local.tcy_up_down, local.tcy_left_right, local.h_scale]
                .iter().any(|v| !v.is_finite() || v.abs() > crate::MAX_COORD) {
                return Err(bad("appearance.copyFrom", "sampled character dimensions cannot be represented with this text transform"));
            }
            out.insert(*id, local);
        }
        Ok(out)
    }

'''
change('    /// The appearance placed on an object whose box (in its paint space) is `to`: placed gradients',
       helper + '    /// The appearance placed on an object whose box (in its paint space) is `to`: placed gradients')
path.write_text(source, encoding='utf-8', newline='\n')
tests = root / 'crates/engine/src/tests/eyedropper.rs'
with tests.open('a', encoding='utf-8') as f:
    f.write(Path(__file__).with_name('leo_eyedropper_tests.rs').read_text(encoding='utf-8'))
print('PASS Eyedropper effective character dimensions, preflight and regression tests')

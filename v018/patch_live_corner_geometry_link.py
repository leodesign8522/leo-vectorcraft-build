#!/usr/bin/env python3
"""Leo Design Live Corner Geometry Link for VectorCraft v0.18.0

When moving a normal (unrounded) anchor on a LiveShape::Path, transform the
anchor in the preserved uncut base path, then regenerate rounded geometry.
The native path_mut() intentionally destroys all live shapes on anchor edits.
That behavior remains for other live shape types and rounded cut handles,
while plain endpoint edits on paths with live corners now behave like Affinity.

Uses LiveCorners::sources() to map displayed anchor indexes to the uncut
base; only ever modifies a live base anchor that has no active radius.
This is a narrow, lossless patch, and does NOT attempt to modify radius-cut
handles as if they were the original corner vertex.
"""
from pathlib import Path
import sys
if len(sys.argv)!=2: raise SystemExit("Usage: patch_live_corner_geometry_link.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])
def rep(path,old,new,label):
    f=root/path;s=f.read_text(encoding="utf-8");n=s.count(old)
    if n!=1: raise SystemExit(f"FAIL {label}: {path}, matches={n}")
    f.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
    print("OK",label)

rep("crates/engine/src/cmd/path.rs",
    '''use vectorcraft_doc::{Document, Node, NodeId, NodeKind, Scaling};''',
    '''use vectorcraft_doc::{Document, LiveCorners, LiveShape, Node, NodeId, NodeKind, Scaling};''',
    "include live geometry types")

helper=r'''
/// Leo Design: for an unrounded anchor on a live-corner path, edit its source
/// geometry (rather than expanding the live shape as path_mut() does), then
/// regenerate its rounded outline with the existing native corner algorithm.
/// Returns the selected anchors after regeneration; None means use the stock
/// destructive edit, e.g. a radius-cut handle, rectangle or non-live path.
fn transform_live_corner_source(
    doc: &mut Document,
    id: NodeId,
    selected: &BTreeSet<(usize, usize)>,
    xf: vectorcraft_geom::Affine,
) -> Option<BTreeSet<(usize, usize)>> {
    let node = doc.node(id)?;
    let NodeKind::Path { live: Some(live), .. } = &node.kind else { return None };
    if !matches!(&**live, LiveShape::Path { .. }) || selected.is_empty() {
        return None;
    }
    let lc = LiveCorners::of(node)?;
    let mut original = BTreeSet::<usize>::new();
    for &(si, ai) in selected {
        let source = *lc.sources().get(si)?.get(ai)?;
        // The source vertex of a rounded corner has two visible cut anchors.
        // A drag on those is *not* an edit to the original sharp vertex.
        // Keep stock handle editing unless an unrounded source anchor was hit.
        if lc.radius(source) > 1e-9 {
            return None;
        }
        original.insert(source);
    }
    // Map the global uncut vertex numbers onto the base's subpaths. The
    // mapping works for any number of subpaths, including open paths.
    let mut mapped = Vec::with_capacity(original.len());
    let mut first = 0usize;
    for (si, sub) in lc.base.subpaths.iter().enumerate() {
        for index in original.iter().copied().filter(|i| *i >= first && *i < first + sub.anchors.len()) {
            mapped.push((si, index - first));
        }
        first += sub.anchors.len();
    }
    if mapped.len() != original.len() {
        return None;
    }
    // Checked all inputs before mutating anything.
    let node = doc.node_mut(id)?;
    let NodeKind::Path { path, live: Some(live), .. } = &mut node.kind else { return None };
    let LiveShape::Path { base, .. } = &mut **live else { return None };
    for (si, ai) in mapped {
        let a = base.anchor_mut(si, ai)?;
        *a = a.transform(xf);
    }
    *path = live.to_path(); // updates tangencies on every drag preview
    let source_map = LiveCorners::of(node)?;
    let next = source_map.sources().iter().enumerate().flat_map(|(si, sources)| {
        sources.iter().enumerate().filter_map(|(ai, src)| original.contains(src).then_some((si, ai)))
    }).collect();
    Some(next)
}

'''
rep("crates/engine/src/cmd/path.rs",
    '''fn append_anchor(s: &mut Session, p: &Value) -> Result<Value> {''',
    helper+'''fn append_anchor(s: &mut Session, p: &Value) -> Result<Value> {''',
    "preserve and regenerate live-corner outline when uncut anchor changes")
rep("crates/engine/src/cmd/path.rs",
    '''    s.edit(label, |doc, _| {
        for id in &sel.objects {
            match sel.anchors.get(id) {
                Some(set) => {
                    let path = path_mut(doc, *id)?;
                    for &(si, ai) in set {
                        if let Some(a) = path.anchor_mut(si, ai) {
                            *a = a.transform(xf);
                        }
                    }
                }''',
    '''    s.edit(label, |doc, selection| {
        for id in &sel.objects {
            match sel.anchors.get(id) {
                Some(set) => {
                    // Leo Design: moving a different anchor must not destroy
                    // an already rounded corner on the same original path.
                    if let Some(next) = transform_live_corner_source(doc, *id, set, xf) {
                        selection.anchors.insert(*id, next);
                        continue;
                    }
                    let path = path_mut(doc, *id)?;
                    for &(si, ai) in set {
                        if let Some(a) = path.anchor_mut(si, ai) {
                            *a = a.transform(xf);
                        }
                    }
                }''',
    "keep live fillet when direct-selection moves unrounded anchor")
rep("crates/engine/src/cmd/path.rs",
    '''    s.edit("Move", |d, _| {
        let path = path_mut(d, id)?;
        let a = path.anchor_mut(si, ai).ok_or_else(|| EngineError::Other("no such anchor".into()))?;
        *a = a.transform(vectorcraft_geom::Affine::translate(pos - a.p));
        Ok(())
    })?;''',
    '''    s.edit("Move", |d, selection| {
        let old = d.node(id).and_then(|n| n.path_data())
            .and_then(|path| path.anchor(si, ai)).map(|a| a.p)
            .ok_or_else(|| EngineError::Other("no such anchor".into()))?;
        let refs = BTreeSet::from([(si, ai)]);
        if let Some(next) = transform_live_corner_source(
            d, id, &refs, vectorcraft_geom::Affine::translate(pos - old)
        ) {
            if selection.anchors.contains_key(&id) {
                selection.anchors.insert(id, next);
            }
            return Ok(());
        }
        let path = path_mut(d, id)?;
        let a = path.anchor_mut(si, ai).ok_or_else(|| EngineError::Other("no such anchor".into()))?;
        *a = a.transform(vectorcraft_geom::Affine::translate(pos - a.p));
        Ok(())
    })?;''',
    "keep live corner on explicit path.moveAnchor endpoint")
print("SUCCESS Leo Design Live Corner Geometry Link port (non-destructive endpoint movement)")

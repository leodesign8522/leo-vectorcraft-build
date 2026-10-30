#!/usr/bin/env python3
"""Leo Design v0.6.8 — A (Direct Selection) marquee selects path edges.

Base VectorCraft v0.18 marquee only tests whether anchor points fall inside a
selection rectangle, so crossing the middle of a line silently selects
nothing. This patch adds exact line-vs-rect and bounded recursive Bezier-vs-rect
tests, and a transaction-safe multi-path segment selection command.

Preserves anchor selection priority, Shift toggle, Group Selection marquee,
native snapping / path.moveAnchors, and existing selection tests.
"""
from pathlib import Path
import sys
if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_direct_marquee_edges.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def rep(rel, old, new, label):
    p = root / rel
    original = p.read_text(encoding="utf-8")
    n = original.count(old)
    if n != 1:
        raise SystemExit(f"FAIL {label}: expected unique anchor, found {n} in {rel}")
    p.write_text(original.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("OK", label)

helpers = r'''
/// Leo Design: a segment with both endpoints *outside* a marquee can still
/// cross it, which the stock Direct Selection marquee did not recognize.
/// Liang–Barsky clipping handles vertical/horizontal/diagonal lines and
/// touching the border without selecting a remote line's bounding box.
fn leo_line_crosses_rect(r: &Rect, a: Point, b: Point) -> bool {
    if r.contains(a) || r.contains(b) {
        return true;
    }
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    for (p, q) in [
        (-dx, a.x - r.x0),
        (dx, r.x1 - a.x),
        (-dy, a.y - r.y0),
        (dy, r.y1 - a.y),
    ] {
        if p.abs() < 1e-12 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lo = lo.max(t);
            } else {
                hi = hi.min(t);
            }
            if lo > hi {
                return false;
            }
        }
    }
    true
}

/// Curves are recursively split, with a convex-hull box check so an offscreen
/// curved edge isn't selected solely because its *overall* bounds overlap.
/// Max depth keeps untrusted artwork predictable even for unusual handles.
fn leo_curve_crosses_rect(r: &Rect, controls: [Point; 4], remaining: u8) -> bool {
    if controls.iter().all(|p| p.x < r.x0)
        || controls.iter().all(|p| p.x > r.x1)
        || controls.iter().all(|p| p.y < r.y0)
        || controls.iter().all(|p| p.y > r.y1)
    {
        return false;
    }
    if controls.iter().any(|p| r.contains(*p)) {
        // A control point being inside alone does not prove the Bezier curve
        // crosses this box. Only the curve endpoints are guaranteed on curve.
        if r.contains(controls[0]) || r.contains(controls[3]) {
            return true;
        }
    }
    if remaining == 0 {
        return leo_line_crosses_rect(r, controls[0], controls[3]);
    }
    let mid = |a: Point, b: Point| Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let a = mid(controls[0], controls[1]);
    let b = mid(controls[1], controls[2]);
    let c = mid(controls[2], controls[3]);
    let d = mid(a, b);
    let e = mid(b, c);
    let f = mid(d, e);
    leo_curve_crosses_rect(r, [controls[0], a, d, f], remaining - 1)
        || leo_curve_crosses_rect(r, [f, e, c, controls[3]], remaining - 1)
}

fn leo_segment_crosses_marquee(r: &Rect, path: &vectorcraft_geom::SubPath, seg: usize) -> bool {
    if seg >= path.segment_count() {
        return false;
    }
    let bezier = path.segment(seg);
    if path.segment_is_line(seg) {
        leo_line_crosses_rect(r, bezier.p0, bezier.p3)
    } else {
        leo_curve_crosses_rect(r, [bezier.p0, bezier.p1, bezier.p2, bezier.p3], 10)
    }
}

'''
rep(
    "crates/tools/src/direct.rs",
    "fn anchors_json(v: &[AnchorRef]) -> Value {",
    helpers + "fn anchors_json(v: &[AnchorRef]) -> Value {",
    "Add line and cubic marquee-intersection helpers"
)

rep(
    "crates/tools/src/direct.rs",
    '''                // Anchors inside the rect, on editable paths inside isolation mode.
                let mut sel: Vec<(NodeId, Vec<AnchorRef>)> = vec![];
                walk_pickable(cx.doc, cx.isolation, |n| {
                    if let NodeKind::Path { path, .. } = &n.kind {
                        let v: Vec<AnchorRef> = path.anchors().filter(|(_, _, a)| r.contains(a.p)).map(|(s, i, _)| (s, i)).collect();
                        if !v.is_empty() {
                            sel.push((n.id, v));
                        }
                    }
                });
                // Objects without anchors to pick''',
    '''                // Anchors inside the rect, on editable paths inside isolation mode.
                // Leo Design: ALSO collect segments crossed by a small marquee
                // even if both endpoint anchors remain outside (Illustrator A).
                let mut sel: Vec<(NodeId, Vec<AnchorRef>)> = vec![];
                let mut edge_hits: Vec<(NodeId, Vec<(usize, usize)>)> = vec![];
                walk_pickable(cx.doc, cx.isolation, |n| {
                    if let NodeKind::Path { path, .. } = &n.kind {
                        let v: Vec<AnchorRef> = path.anchors().filter(|(_, _, a)| r.contains(a.p)).map(|(s, i, _)| (s, i)).collect();
                        if !v.is_empty() {
                            sel.push((n.id, v));
                        } else {
                            let mut edges = vec![];
                            for (si, subpath) in path.subpaths.iter().enumerate() {
                                for seg in 0..subpath.segment_count() {
                                    if leo_segment_crosses_marquee(&r, subpath, seg) {
                                        edges.push((si, seg));
                                    }
                                }
                            }
                            if !edges.is_empty() {
                                edge_hits.push((n.id, edges));
                            }
                        }
                    }
                });
                // Objects without anchors to pick''',
    "Detect crossed path segments without needing both anchor endpoints"
)

rep(
    "crates/tools/src/direct.rs",
    '''                let editable = cx.doc.editable();
                sel.retain(|(id, _)| editable(*id));
                let items: Vec<Value> = sel.iter().map(|(id, v)| json!({"id": id.0, "anchors": anchors_json(v)})).collect();
                let art = Action::Exec("select.anchorsMany".into(), json!({"items": items, "mode": if toggle { "toggle" } else { "set" }}));''',
    '''                let editable = cx.doc.editable();
                sel.retain(|(id, _)| editable(*id));
                edge_hits.retain(|(id, _)| editable(*id));
                // Native anchor selection wins if the marquee actually includes
                // anchor handles. Otherwise a crossed edge selects its segment
                // and both endpoints are moved together by path.moveAnchors.
                if sel.is_empty() && !edge_hits.is_empty() {
                    let items: Vec<Value> = edge_hits.iter().map(|(id, edges)| {
                        json!({"id": id.0, "segments": edges.iter().map(|(s, i)| json!([s, i])).collect::<Vec<_>>()})
                    }).collect();
                    let art = Action::Exec("select.segmentsMany".into(),
                        json!({"items": items, "mode": if toggle { "toggle" } else { "set" }}));
                    return std::iter::once(art)
                        .chain(crate::rulerguide::marquee_select(cx, r, false)).collect();
                }
                let items: Vec<Value> = sel.iter().map(|(id, v)| json!({"id": id.0, "anchors": anchors_json(v)})).collect();
                let art = Action::Exec("select.anchorsMany".into(), json!({"items": items, "mode": if toggle { "toggle" } else { "set" }}));''',
    "Select crossed path segments using native selection model"
)

rep(
    "crates/engine/src/cmd/select.rs",
    '''        cmd!(
            "select.anchorsMany",
            "Select Anchors",''',
    '''        cmd!(
            "select.segmentsMany",
            "Select Marquee Segments",
            [],
            None,
            "{items: [{id, segments: [[subpath, segment]…]}], mode?: \\"set\\"|\\"add\\"|\\"toggle\\"}: select multiple crossed path edges from a Direct Selection marquee",
            has_doc,
            segments_many
        ),
        cmd!(
            "select.anchorsMany",
            "Select Anchors",''',
    "Register select.segmentsMany command"
)

rep(
    "crates/engine/src/cmd/select.rs",
    '''fn anchors(s: &mut Session, p: &Value) -> Result<Value> {''',
    r'''/// Atomic multi-object segment marquee: no partial selection if any
/// document ID or segment reference is invalid.
fn segments_many(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "select.segmentsMany";
    let raw = p.get("items").and_then(Value::as_array)
        .ok_or_else(|| bad(C, "items must be an array"))?;
    let mode = match str_param(p, "mode") {
        None | Some("set") => "set",
        Some("add") => "add",
        Some("toggle") => "toggle",
        Some(m) => return Err(bad(C, format!("unknown mode {m:?}"))),
    };
    let mut items = vec![];
    for it in raw {
        let id = checked_id(s, it.get("id").ok_or_else(|| bad(C, "each item needs an id"))?, C)?;
        let refs: BTreeSet<_> = checked_refs(it.get("segments"), C)?.into_iter().collect();
        let endpoints = segment_endpoints(&s.doc()?.doc, id, &refs, C)?;
        items.push((id, refs, endpoints));
    }
    s.select(|d, sel| {
        if mode == "set" {
            sel.clear();
        }
        for (id, refs, endpoints) in items {
            if refs.is_empty() {
                continue;
            }
            if mode == "set" || mode == "add" {
                sel.add(id);
                sel.anchors.entry(id).or_default().extend(endpoints);
                sel.segments.entry(id).or_default().extend(refs);
            } else {
                let seg_entry = sel.segments.entry(id).or_default();
                let mut removed = BTreeSet::new();
                let mut added = BTreeSet::new();
                let path = d.node(id).and_then(|n| n.path_data());
                for &(si, seg) in &refs {
                    let Some(sp) = path.as_ref().and_then(|p| p.subpaths.get(si)) else { continue };
                    let n = sp.anchors.len().max(1);
                    let points = [(si, seg), (si, (seg + 1) % n)];
                    if seg_entry.remove(&(si, seg)) {
                        removed.extend(points);
                    } else {
                        seg_entry.insert((si, seg));
                        added.extend(points);
                    }
                }
                let mut remaining = BTreeSet::new();
                for &(si, seg) in seg_entry.iter() {
                    if let Some(sp) = path.as_ref().and_then(|p| p.subpaths.get(si)) {
                        let n = sp.anchors.len().max(1);
                        remaining.extend([(si, seg), (si, (seg + 1) % n)]);
                    }
                }
                let anchors = sel.anchors.entry(id).or_default();
                for pt in removed {
                    if !remaining.contains(&pt) {
                        anchors.remove(&pt);
                    }
                }
                anchors.extend(added);
                if seg_entry.is_empty() {
                    sel.segments.remove(&id);
                }
                if anchors.is_empty() {
                    sel.remove(id);
                } else {
                    sel.add(id);
                }
            }
        }
    })?;
    ok()
}

fn anchors(s: &mut Session, p: &Value) -> Result<Value> {''',
    "Implement atomic multi-object segment selection"
)

tests = r'''
// Leo Design: Direct Selection marquee must pick the intersected edge, not
// require its endpoints. These pure geometry tests keep the hit test stable.
#[test]
fn leo_marquee_crosses_middle_of_straight_edge() {
    let r = Rect::new(95.0, 120.0, 105.0, 180.0);
    let path = vectorcraft_geom::SubPath::polyline(
        &[Point::new(100.0, 100.0), Point::new(100.0, 200.0)], false);
    assert!(leo_segment_crosses_marquee(&r, &path, 0));
    assert!(!r.contains(Point::new(100.0, 100.0)));
    assert!(!r.contains(Point::new(100.0, 200.0)));
    // Just a nearby bounding rectangle must not select the edge.
    assert!(!leo_segment_crosses_marquee(&Rect::new(140.0, 120.0, 145.0, 180.0), &path, 0));
}

#[test]
fn leo_marquee_diagonal_and_bezier_crossings() {
    let r = Rect::new(45.0, 45.0, 55.0, 55.0);
    assert!(leo_line_crosses_rect(&r, Point::new(0.0, 0.0), Point::new(100.0, 100.0)));
    assert!(!leo_line_crosses_rect(&r, Point::new(0.0, 90.0), Point::new(100.0, 90.0)));
    let controls = [
        Point::new(0.0, 0.0), Point::new(15.0, 100.0),
        Point::new(85.0, 100.0), Point::new(100.0, 0.0),
    ];
    let curve_r = Rect::new(45.0, 70.0, 55.0, 80.0);
    assert!(leo_curve_crosses_rect(&curve_r, controls, 10));
    assert!(!leo_curve_crosses_rect(&Rect::new(45.0, 5.0, 55.0, 10.0), controls, 10));
}

#[test]
fn leo_marquee_ignores_open_path_nonexistent_closing_edge() {
    let path = vectorcraft_geom::SubPath::polyline(
        &[Point::new(0.0, 0.0), Point::new(100.0, 0.0)], false);
    assert!(!leo_segment_crosses_marquee(&Rect::new(0.0, 20.0, 100.0, 30.0), &path, 1));
}
'''
p=root/"crates/tools/src/direct/tests.rs"
p.write_text(p.read_text(encoding="utf-8")+"\n"+tests,encoding="utf-8",newline="\n")
print("OK three segment marquee regression tests")
print("SUCCESS Leo Design A-tool marquee selection of path edges")

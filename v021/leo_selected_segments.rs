//! Extract direct-selected geometry without changing the source document.
use vectorcraft_doc::{Node, NodeKind, Selection};
use vectorcraft_geom::{PathData, SubPath};

pub(super) fn selected_node(node: &Node, selection: &Selection) -> Option<Node> {
    let Some(anchors) = selection.anchors.get(&node.id) else { return Some(node.clone()) };
    let NodeKind::Path { path, .. } = &node.kind else { return Some(node.clone()) };
    let explicit = selection.segments.get(&node.id).filter(|s| !s.is_empty());
    let mut subpaths = Vec::new();
    for (si, sp) in path.subpaths.iter().enumerate() {
        let n = sp.anchors.len();
        let count = sp.segment_count();
        let picked: Vec<bool> = (0..count).map(|seg| {
            explicit.map_or_else(
                || anchors.contains(&(si, seg)) && anchors.contains(&(si, (seg + 1) % n)),
                |set| set.contains(&(si, seg)),
            )
        }).collect();
        if count == 0 || !picked.iter().any(|v| *v) {
            continue;
        }
        if picked.iter().all(|v| *v) {
            subpaths.push(sp.clone());
            continue;
        }
        // Start just after a missing edge, so a run crossing the closed-path seam stays joined.
        let start = if sp.closed { (picked.iter().position(|v| !*v).unwrap() + 1) % count } else { 0 };
        let mut run = Vec::new();
        for step in 0..count {
            let seg = (start + step) % count;
            if picked[seg] {
                if run.is_empty() {
                    let mut a = sp.anchors[seg];
                    a.h_in = a.p;
                    run.push(a);
                }
                run.push(sp.anchors[(seg + 1) % n]);
            } else if !run.is_empty() {
                let last = run.last_mut().unwrap();
                last.h_out = last.p;
                subpaths.push(SubPath::new(std::mem::take(&mut run), false));
            }
        }
        if let Some(last) = run.last_mut() {
            last.h_out = last.p;
            subpaths.push(SubPath::new(run, false));
        }
    }
    if subpaths.is_empty() {
        return None;
    }
    let mut result = node.clone();
    if let NodeKind::Path { path, live, clipping, guide, .. } = &mut result.kind {
        *path = PathData::new(subpaths);
        // A live rectangle/corner generator must never reconstruct the omitted edges on edit.
        *live = None;
        *clipping = false;
        *guide = false;
    }
    Some(result)
}

#[cfg(test)]
mod leo_segment_copy_geometry_tests {
    use super::*;
    use vectorcraft_doc::{Appearance, NodeId};
    use vectorcraft_geom::{Anchor, Point};

    fn fixture(closed: bool, segments: &[usize]) -> (Node, Selection) {
        let node = Node::path(NodeId(10), PathData::single(SubPath::polyline(&[
            Point::new(0.0, 0.0), Point::new(10.0, 0.0),
            Point::new(10.0, 10.0), Point::new(0.0, 10.0),
        ], closed)), Appearance::default_art());
        let mut sel = Selection::default();
        sel.set([node.id]);
        sel.segments.insert(node.id, segments.iter().map(|&s| (0, s)).collect());
        sel.anchors.insert(node.id, segments.iter().flat_map(|&s| [(0, s), (0, (s + 1) % 4)]).collect());
        (node, sel)
    }
    fn geometry(node: &Node) -> &PathData {
        let NodeKind::Path { path, .. } = &node.kind else { panic!("path") };
        path
    }

    #[test]
    fn one_edge_is_open_and_source_is_unchanged() {
        let (node, sel) = fixture(true, &[1]);
        let original = node.clone();
        let copied = selected_node(&node, &sel).unwrap();
        let p = geometry(&copied);
        assert_eq!(p.subpaths.len(), 1);
        assert!(!p.subpaths[0].closed);
        assert_eq!(p.anchor_count(), 2);
        assert_eq!(p.subpaths[0].anchors[0].p, Point::new(10.0, 0.0));
        assert_eq!(node, original);
    }

    #[test]
    fn disjoint_edges_never_gain_a_connecting_edge() {
        let (node, sel) = fixture(true, &[0, 2]);
        let copied = selected_node(&node, &sel).unwrap();
        let p = geometry(&copied);
        assert_eq!(p.subpaths.len(), 2);
        assert!(p.subpaths.iter().all(|sp| !sp.closed && sp.segment_count() == 1));
    }

    #[test]
    fn connected_edges_across_closed_seam_stay_joined() {
        let (node, sel) = fixture(true, &[3, 0]);
        let copied = selected_node(&node, &sel).unwrap();
        let p = geometry(&copied);
        assert_eq!(p.subpaths.len(), 1);
        assert_eq!(p.subpaths[0].segment_count(), 2);
        assert_eq!(p.subpaths[0].anchors[0].p, Point::new(0.0, 10.0));
        assert!(!p.subpaths[0].closed);
    }

    #[test]
    fn every_edge_keeps_closed_geometry_and_whole_object_copy_stays_whole() {
        let (node, mut sel) = fixture(true, &[0, 1, 2, 3]);
        assert_eq!(geometry(&selected_node(&node, &sel).unwrap()), geometry(&node));
        sel.anchors.clear();
        sel.segments.clear();
        assert_eq!(selected_node(&node, &sel), Some(node));
    }

    #[test]
    fn curved_edge_retains_exact_bezier_handles() {
        let (mut node, sel) = fixture(false, &[1]);
        if let NodeKind::Path { path, .. } = &mut node.kind {
            path.subpaths[0].anchors[1] = Anchor::with_handles(
                Point::new(10.0, 0.0), Point::new(9.0, -3.0), Point::new(15.0, 2.0));
            path.subpaths[0].anchors[2] = Anchor::with_handles(
                Point::new(10.0, 10.0), Point::new(15.0, 8.0), Point::new(8.0, 12.0));
        }
        let copied = selected_node(&node, &sel).unwrap();
        let original_segment = geometry(&node).subpaths[0].segment(1);
        assert_eq!(geometry(&copied).subpaths[0].segment(0), original_segment);
        assert_eq!(copied.appearance, node.appearance);
    }

    #[test]
    fn anchor_selection_infers_only_edges_with_both_endpoints_selected() {
        let (node, mut sel) = fixture(true, &[0]);
        sel.segments.clear();
        assert_eq!(geometry(&selected_node(&node, &sel).unwrap()).subpaths[0].segment_count(), 1);
    }
}

# Leo Design Snapping Engine v1 — Node Geometry (VectorCraft 0.21.1)

## Contract
- A drag of one or more directly selected anchors uses a **new target chooser**, not the native axis-alignment guide chooser.
- **Capture radius:** exactly 8 screen px. **Release radius:** 8 × 1.6 = 12.8 screen px. The document-unit radius is `px / zoom`; the same screen behavior applies at 25%, 100% and 400%.
- Actual anchors win over projected segments, even if a segment has a slightly shorter distance. Anchors on objects currently being moved are excluded by existing target gathering. Fixed anchors on partly selected paths remain eligible.
- When an anchor is captured, it remains locked until the pointer leaves the release radius; when a path is captured, the closest point moves continuously *on the same path* while within release radius. A real anchor within 8px may supersede a path lock.
- Paths are **finite straight segments** or true **Bezier curves**; their nearest-point result is the actual projected document coordinate. No bounds, centres, infinite extension lines, horizontal or vertical alignment guides may claim a v1 node hit.
- The preview command receives the returned exact coordinate. Feedback markers and labels are shown **only** for a captured geometric target; if labels/lines are disabled, the coordinate still snaps.

## Deliberately deferred
- Ordinary full-object movement, resizing, rotating, group anchors, construction-angle snapping and all remaining snapping feature switches are not claimed as part of this first node-geometry release.
- Shift-constrained movement still uses the existing constrained gesture while its geometry constraints await a separate port; pixel/grid toggles remain a *fallback only* when no geometry is captured.
- Existing Smart Guides and Snap to Point switches enable this first node engine. A dedicated settings UI for independent capture/release preferences will be built after the initial behavior has been validated on Windows.

## Automated checks
GitHub Actions (Windows): `cargo test --locked -p vectorcraft-tools leo_snapping_engine -- --nocapture`, then the native guide checks. Coverage includes:
1. Anchor equality, priority over a close path, and absence of false centres/alignment hits.
2. Finite segment projection (no unintended infinite extension).
3. Nearest point on an actual cubic Bezier.
4. 8px capture and 12.8px release hysteresis, including moving along a captured segment.
5. Zoom parity at 25%, 100% and 400%, and snapping while visual guides are hidden.
6. Pointer drag to `path.moveAnchors` preview delta for an anchor and a segment.

**Build/test pass is not Windows hands-on validation.** Before describing this as Affinity-quality, manually check tight geometry, fast pointer movement, dense nodes, zoom, and copy/Shift gestures.


## v1.1 screenshot regression (2026-11-06)
The screenshot shows a polygon corner almost touching the right border of a white artboard.
- The v1 target set excluded **artboard edges**, as they are not Path objects.
- Clicking an anchor on a wholly selected path enters the DirectSelection **MoveObject** state. It had still called native PointSnap rather than `direct_anchor_drag`.
Both are fixed in v1.1: finite edges of every artboard are included as real segment candidates, and whole-path anchor grabs call the same true-geometry engine as partial node moves. Added pointer-level regression tests for both cases.

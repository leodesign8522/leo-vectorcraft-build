# Core Bridge 0.14.3 / VectorCraft 0.22.0

Upstream tag: `v0.22.0`, commit `654749c16eb20b6d3182a96defb66e11d1a9649a`.
Recipe base: `leo-v021-migration`, commit `b971ec99aaf99613b1c7ec77a6a0f96891dea636`.

## 0.14.3 visible-reference extensions and yellow feedback

Single-anchor Direct Selection now includes stationary non-adjacent straight edges and other visible vector objects. A direction/offset index queries actual supporting lines without the 20k cutoff. Acquisition of these references requires their real segment to intersect the current screen frame (with an 8px boundary allowance); segment clipping handles rotated views and excludes diagonals whose bounding boxes alone overlap. The view is read on every query, so panning/zooming can reveal new references. An already-held line survives panning until its normal 12.8px release. Adjacent original-edge directions remain frozen at drag start.

True finite geometry keeps priority. Curves and hidden objects are excluded from external straight-edge references. The distinct extension overlay is bright yellow, with a dark dashed underlay, outlined marker and dark-backed Chinese label. Painted dashes are bounded to about 512 screen pixels; mathematical extension remains bidirectional. Shift retains its existing constraint.

Windows compilation, installer hash/version checks, viewport/index/native Direct Selection regressions and local actual-app control checks passed. The local cases cover all 16 corner/neighbor/direction combinations, foreign and non-adjacent edges, viewport changes during dragging, release, Commit/Undo/Redo and retained dense-document/Shift/Alt behavior. See [verification and exact hashes](VERIFIED_RELEASE.md). User documents remain local.

## 0.14.2 bidirectional original-edge constraints

Direct Selection of one anchor now gathers at most its two original straight neighbours at drag start. A separate extension lock projects onto these original lines in either direction, including a closed path and movement past the fixed neighbour. Its 8px acquisition and 12.8px release are independent of the immutable finite-geometry target snapshot. Finite geometry keeps priority; Shift retains its original constraint without a duplicate extension capture. Curves, whole-object moves and multiple-anchor selections do not gain these direction constraints.

Captured extensions emit cyan dashed lines, a point marker and an `extension` label, not a finite `path` hit. The complete Windows workflow, installer smoke test and local native-control cases passed for `893781382e36117022d990d086b3d7ecbfbe3575`. Local cases include outward/inward/past-end movement, frozen drag-start direction after a free preview, 12.7px hold versus 12.9px release, unchanged stationary nodes, Commit/Undo/Redo, retained dense-document snapping and Shift/Alt behavior. See [current evidence and hashes](VERIFIED_RELEASE.md). `tests/extension_runtime.py` uses synthetic geometry only and takes an isolated portable executable path.

## 0.14.1 dense-document follow-up

The 0.14.0 regressions missed the native target collector's 20,000-anchor budget: later paths were omitted in a dense document, so both free and Shift node drags had no target. Drag-start path snapshots now cover the full visible scene; finite geometry is queried near the pointer or translated bounds. Object alignment bounds are collected independently of anchor counts. Sticky locks retain the actual segment, so changing local candidates cannot change a lock's identity.

Synthetic late line and Bezier targets after 20,004 background anchors pass, including three zoom levels and both Shift states. A local recovery copy of the reported document reproduced the failure in 0.14.0 and passed in 0.14.1 during dragging, on commit and through Undo/Redo, with the stationary anchor and target unchanged. User documents and actual drawing coordinates remain local.

The complete Windows build and installer smoke test passed for `427b1c93cd78960034fd560ff571d3b2b4533754`. The additional Bezier regression in `bccec40aa82741c3bb9e7b137ff4ed1143cb007d` changes tests only. See [current release evidence](VERIFIED_RELEASE.md); the older verification below records the 0.14.0 scenarios and is not evidence for the dense-document case.

## Diagnosis

At the start of the migration, the previous successful Actions run, [42257643172](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42257643172), builds Core Bridge 0.13.2 against **0.21.1**. It is a replacement executable, not a dynamically loaded extension. An official 0.22.0 executable does not load that patched engine. Updating the official application cannot carry these source patches forward.

Independent code defects in the old engine:

- Shift node drags return through native `PointSnap::snap_from`, bypassing Leo geometry and its hysteresis.
- A Shift Down on an already-selected anchor immediately toggles it off in native Direct Selection. A real Session then has no selected anchor for the drag, even though a tool-only test with a frozen context can still report a Preview. Shift deselection is now deferred until a click releases without a drag, including whole-path and segment-endpoint selections.
- `MoveSnap` does not instantiate Leo snapping with grid or pixel snapping enabled; node and object behavior disagree.
- Direct Selection's Alt-copy point snapper uses the native constructor, which discards geometry targets with grid/pixel snapping.
- Shift object geometry projects nearby anchors onto the drag line and can report a hit at an unreachable point. Segment targets are skipped entirely.
- Whole-object sources use discrete anchors and nine box features, missing targets alongside the middle of a long edge.

These findings explain the earlier failures. The local recovery-copy reproduction above confirms the dense-document follow-up; it does not imply coverage of every possible document or operation.

Windows executable control-channel reproduction with generated rectangles: with Smart Guides and Snap to Grid enabled, a group intended for the artboard's right edge at 500 landed at approximately 498.8976; switching grid snapping off made the same operation land exactly at 500. A selected-endpoint Shift drag in the official executable also left the geometry unmoved and the selection empty. Both cases have explicit regression coverage; no user document or preferences are included in the repository.

## API migration

- Tool pointer events and Begin/Preview/Commit still use the same interfaces. All source anchors are checked and the complete recipe has been applied to a clean 0.22.0 checkout.
- Native shortcut dispatch now feeds Type Enter events in event order and skips Enter in its generic tool-key loop. The migrated Ctrl+Enter patch preserves that new loop.
- `Session::lent_tool` now handles Direct Selection Cmd with a combined branch. Smart Select inserts its Selection Cmd override before that branch.
- Native clipboard extraction now supports selected segments and creates one editable node per disconnected run. The obsolete clipboard implementation patch is omitted; command regressions check each run's exact endpoints, open-path state, Copy/Paste and Cut/Undo instead of the old compound-subpath object count.
- Dependencies remain the versions in the official `Cargo.lock`.

## Snapping behavior

Nodes use real finite anchors and line/Bezier segments, including four finite artboard edges. Shift captures only actual intersections along the locked line; a nearby off-line node cannot produce a marker. Acquisition is 8 screen pixels, held targets release beyond 12.8 screen pixels, converted with the active zoom.

Object, multiple-object and group moves return one translation. Continuous source contours add candidates between the old discrete features. Nearby curve pairs use bounded iterative nearest-point searches; candidates always resolve onto target geometry. This is not a proof of the global minimum for every possible pair of self-intersecting curves. Dense drawings cap contour-pair searches at 2048 per pointer sample; ordinary anchor and bounds targets remain available.

Shift object moves preserve the constrained direction and resolve actual path intersections. Geometry and alignment locks use the same capture/release thresholds. Alt switches target sets so originals remain targets for a copy. Grid/pixel options serve as fallback when Leo geometry is enabled.

Native grid-only edge snapping is retained when no Leo geometry is captured, including mixed anchor/object drags. Explicit perpendicular and tangent options still resolve onto finite target paths. Node-only axis alignment and unbounded source-extension rays do not count as physical captures; their old expectations are replaced with assertions that the node stays unsnapped and no hit guide appears.

## Verification and delivery

Run `python prepare_v022.py <pristine-upstream-checkout>` once on the pinned source. Reapplying to already patched source deliberately fails.

`.github/workflows/build-v022-core.yml` calls the Windows edition workflow. Artifacts upload only after the configured geometry, tool-pointer, session preview/commit/undo/redo, native guides, and retained Core regression commands pass, followed by the release build and version smoke test.

The installer uses an independent per-user directory, retains the official installation, verifies the installed executable against the tested artifact hash, and checks its build identity. Both artifact sets include source commit identifiers and SHA-256 checksums. The current About and `--version` identify Core Bridge 0.14.2 / Snapping Engine 1.4.

Historical 0.14.0 verification: all configured regression commands and the Windows x64 release build passed for `f96207bf7203f5b3d9054a581402054c83cf1587`. This includes all 73 Direct Selection tests, three real Session regressions, the adapted clipboard regressions, and the retained Core checks. The initial installer wrapper omitted VERSION.txt because of GUI stdout handling; explicit process waiting and stdout redirection fixed metadata capture. The verified EXE was repackaged without changing its bytes, installed successfully, and checked against its pinned SHA-256. Local executable tests passed with grid snapping enabled for rigid group movement, Undo/Redo, Shift+Alt copying with the original unchanged, and Shift node dragging onto an actual oblique path intersection.

See [verified release, hashes and evidence](VERIFIED_RELEASE.md). Windows control-channel test scripts are in `tests/runtime_smoke.py` and `tests/shift_runtime.py`; run them against an isolated extracted portable copy. They create synthetic documents and record results beside that copy.

# Core Bridge 0.15.0 / Illustrator-backed editable AI export

Adds an explicit UI export command without changing native AI saves. An immutable document snapshot replaces live text with uniquely named spot-colour markers while the native encoder retains nontext geometry and structure. A fixed JSON-driven Illustrator script recreates point/area text in the same hierarchy, maps supported character/paragraph attributes and transforms, and saves an actual AI file. A fresh native Illustrator document avoids imported-editing-data save failures. Artboard metadata is snapshotted before creating the destination; text attributes are acquired once from the final range reference. Empty frames seed and remove a private temporary glyph before saving.

Windows uses a hidden STA PowerShell COM worker with file/env arguments, bounded waiting, explicit font resolution and guarded owned-directory cleanup. Source nodes and selection remain unchanged; the final destination is written only after success. Assigned ICC, unsupported text structures and advanced attributes are rejected instead of silently losing data. RGB/CMYK mode and channel preservation is separately tested from ICC inheritance.

Six CI preflight tests, actual-command export/Illustrator edit-save-reopen checks and all configured retained regressions, release compilation and installer identity/hash verification pass. See [exact evidence and limitations](VERIFIED_RELEASE.md).

# Core Bridge 0.14.9 / VectorCraft 0.22.0

Whole selected containers now remain selected throughout Direct Selection drags from descendant anchors, strokes and fills. Click-only release still drills down; Alt ancestor cycling and copying stay native. Shift+Space is disabled at canvas pointer-down, with no navigation cursor; Space alone still pans. Three engine regressions, five UI state/navigation tests, 30 actual A-key group cases and retained suites pass. See [verification](VERIFIED_RELEASE.md).

# Core Bridge 0.14.8 / VectorCraft 0.22.0

Upstream tag: `v0.22.0`, commit `654749c16eb20b6d3182a96defb66e11d1a9649a`.
Recipe base: `leo-v021-migration`, commit `b971ec99aaf99613b1c7ec77a6a0f96891dea636`.

## 0.14.8 text mode sampling, state/defaults, grid priority and navigation

Eyedropper adds default-enabled independent pick/apply flags for text frame mode and glyph controls. Whole text targets take applicable modes without source geometry/content; range and append behaviour stays scoped. Paragraph mode controls show selected/checkmarked current state or mixed state across ordinary groups. New text factories enable glyph controls; new area text defaults to Auto Height, with a one-time UI preference migration that preserves opaque settings and subsequent user changes. Legacy object deserialization remains unchanged.

Grid masks smart geometry in engine ToolContext and guards existing node/resize/move/R capture snapshots, releasing locks when grid is enabled. Native grid fallback keeps constraints; pixel-only geometry behaviour is retained. The saved Smart Guides preference is not overwritten. Shift+Space no longer starts Rotate View or shows its cursor/dial, and pans like Space; explicit Rotate View and Cmd+Space remain.

Six new engine tests, eleven retained glyph tests, five real egui state/migration/navigation tests and retained suites pass. Native runtime checks cover actual I sampling, old-file compatibility, eight grid/Smart combinations and capture toggling, plus prior text, alignment, rotation, resize and extension cases. Windows x64 build and installer identity/hash verification pass. See [verification](VERIFIED_RELEASE.md).

## 0.14.7 alignment follows enabled glyph controls

The 0.14.6 controls used glyph ink while Align's separate Measure still used metric/frame bounds. Route enabled text (including ordinary grouped text) through the same ink measurement in Measure.of and its aggregate path. This fixes horizontal, vertical and combined centres, including selection, artboard and key-object references, and makes shared edge/distribution measurements consistent. Legacy global glyph alignment preferences and disabled text retain existing behaviour. Alignment translates whole text without reflow.

Eleven text regressions pass. Six actual Align UI-command cases are checked against temporary native outline geometry and undo restores editable text; all six fail on the 0.14.6 negative control and pass on 0.14.7. Retained text/render/save/reopen, rotation and resize runtime checks pass. Windows x64 build, preserved features and installer identity/hash validation pass. See [verification](VERIFIED_RELEASE.md).

## 0.14.6 non-destructive glyph selection controls

Character panel and Type menu add **贴合字形选框** (`type.glyphSelectionBounds`). A per-text persisted flag chooses laid-out glyph ink for selection and transform controls, without changing TextKind frames, transforms, runs, paragraph breaks or render geometry. The Type tool keeps its real editing frame. V handles and numeric transforms use the same tight measurement and scale editable area text without resizing its flow frame. Empty ink falls back to the native frame; transformed/effected objects retain native controls where needed. Mixed ordinary groups include the selected text's ink, while stale metric-frame outlines and Selection-only text ports are hidden in this mode. Turning the flag off restores native controls.

Eight engine regressions pass, including point/area/CJK text, exact rotated-outline measurement, old/new serialization, edits, actual V pointer scaling, numeric scaling, preview bounds, groups, and undo/redo. Five isolated native-app cases preserve rendered PNG bytes exactly and survive save/reopen and continued text editing. The actual Character button and selected-frame screenshots were checked. Windows x64 build, retained regressions and installer identity/hash checks pass. See [verification and exact hashes](VERIFIED_RELEASE.md).

## 0.14.5 R-tool fixed-pivot edge directions

R rotation previously snapped the mouse in XY, then inferred an angle; that did not solve edge-to-edge direction alignment. A separate immutable rotation snapshot now gathers the selected straight edges and stationary foreign straight edges, sorted by undirected angle. Acquisition queries wrapped angular intervals and filters real reference segments against the current screen; selected groups and hidden sources are excluded. Rotation stays about the user's pivot, with no translation or deformation. Equal supporting lines and parallel directions have distinct yellow Chinese labels. Full-anchor selections are supported; partial-anchor transforms and V bounding-box rotation retain their native behaviour.

Capture/release use 8/12.8 screen pixels converted via the maximum collected edge-endpoint/pointer radius, with exact chord travel rather than a fixed angular tolerance. A held angle survives camera movement until release. Shift keeps native 45-degree quantisation, Alt keeps copy-on-release, and captured angle readouts are normalised. Dashed feedback is bounded to real segment lengths or 512 screen pixels. Target collection has no 20k cutoff.

The 53 engine, 17 object, 77 Direct Selection, 83 Selection and 14 native transform-tool filtered tests pass, as do eight real Session cases (filters overlap). Windows compilation and installer version/hash smoke checks pass. Actual application-control checks cover nine grid/pixel/modifier combinations, collinearity, parallelism, a fixed custom pivot, full anchor selection, group rigidity, 8/12.8px hysteresis and Commit/Undo/Redo. A 0.14.4 negative control fails six ordinary/Alt edge-direction cases while its three Shift cases already pass. Retained resize/node/object/extension tests and the private local recovery-copy checks pass. See [verification and exact hashes](VERIFIED_RELEASE.md).

## 0.14.4 bounding-box resize geometry

The Selection tool's bounding-box handles still entered the official PointSnap/Targets::for_scale path: its 20k anchor budget omitted later targets, and grid/pixel options bypassed geometry. A dedicated scale constructor now uses the immutable full-scene collector and queries finite geometry around the resized box. Legal scale trajectories intersect actual anchors/segments, preserving the opposite handle, proportional scaling and Alt-centred scaling. Side handles have fixed 8px acquisition and 12.8px release; modifier changes reset the lock. Geometry precedes grid/pixel fallback. Object-bound alignment respects Smart Guides; ruler guides keep their configured availability. Yellow extension feedback remains the 0.14.3 node-drag feature.

Expanded native Selection coverage also found and fixed two earlier issues: unconstrained ruler acquisition needed an explicit distance check, and the shared object target set needed spacing boxes. The native resize tolerance expectation is deliberately rebased to Leo's fixed 8px contract; the rest of the original Selection suite runs unchanged. All 47 engine, 17 object, 77 Direct Selection and 83 Selection filtered cases pass, plus seven real Session cases (filters overlap). Windows compilation and installer version/hash checks pass. Actual local control checks cover 12 four-side/grid-mode combinations, hysteresis, Shift/Alt, Commit/Undo/Redo and retained node/object/extension behavior. A synthetic 0.14.3 negative control fails all 12 resize cases. See [verification and exact hashes](VERIFIED_RELEASE.md).

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

# Core Bridge 0.14.0 / VectorCraft 0.22.0

Upstream tag: `v0.22.0`, commit `654749c16eb20b6d3182a96defb66e11d1a9649a`.
Recipe base: `leo-v021-migration`, commit `b971ec99aaf99613b1c7ec77a6a0f96891dea636`.

## Diagnosis

The previous successful Actions run, [42257643172](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42257643172), builds Core Bridge 0.13.2 against **0.21.1**. It is a replacement executable, not a dynamically loaded extension. An official 0.22.0 executable does not load that patched engine. Updating the official application cannot carry these source patches forward.

Independent code defects in the old engine:

- Shift node drags return through native `PointSnap::snap_from`, bypassing Leo geometry and its hysteresis.
- A Shift Down on an already-selected anchor immediately toggles it off in native Direct Selection. A real Session then has no selected anchor for the drag, even though a tool-only test with a frozen context can still report a Preview. Shift deselection is now deferred until a click releases without a drag, including whole-path and segment-endpoint selections.
- `MoveSnap` does not instantiate Leo snapping with grid or pixel snapping enabled; node and object behavior disagree.
- Direct Selection's Alt-copy point snapper uses the native constructor, which discards geometry targets with grid/pixel snapping.
- Shift object geometry projects nearby anchors onto the drag line and can report a hit at an unreachable point. Segment targets are skipped entirely.
- Whole-object sources use discrete anchors and nine box features, missing targets alongside the middle of a long edge.

These source findings explain specific failures. An interactive reproduction against the user's exact document remains necessary to confirm every reported symptom.

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

The installer uses an independent per-user directory, retains the official installation, verifies the installed executable against the tested artifact hash, and checks its build identity. Both artifact sets include source commit identifiers and SHA-256 checksums. About and `--version` identify Core Bridge 0.14.0 / Snapping Engine 1.3.

Verified release: all configured regression commands and the Windows x64 release build passed for `f96207bf7203f5b3d9054a581402054c83cf1587`. This includes all 73 Direct Selection tests, three real Session regressions, the adapted clipboard regressions, and the retained Core checks. The initial installer wrapper omitted VERSION.txt because of GUI stdout handling; explicit process waiting and stdout redirection fixed metadata capture. The verified EXE was repackaged without changing its bytes, installed successfully, and checked against its pinned SHA-256. Local executable tests passed with grid snapping enabled for rigid group movement, Undo/Redo, Shift+Alt copying with the original unchanged, and Shift node dragging onto an actual oblique path intersection.

See [verified release, hashes and evidence](VERIFIED_RELEASE.md). Windows control-channel test scripts are in `tests/runtime_smoke.py` and `tests/shift_runtime.py`; run them against an isolated extracted portable copy. They create synthetic documents and record results beside that copy.

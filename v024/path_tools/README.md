# Leo Design Path Tools — Preview 4 candidate

Preview 4 adds the independent **Trim and Join** tool. It is an initial
open-path implementation, not complete AG Trim and Join parity. Existing
Core Bridge and the prior four tool implementations are retained; no pie menu.

[Windows x64 single installer](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/44573057281/artifacts/14557134414).
Extract the artifact ZIP and run Leo_Design_Path_Tools_Preview_4_Windows_x64_Setup.exe.
Built from 1d6cbcf43508e692a272e00895d29fd2e96e1faa.
Open Window > Leo Design Path Tools > 5 · Trim and Join.
Scroll the flyout or dock its tab into the main sidebar if controls are clipped.

## Trim and Join scope

Drag across unwanted portions of expanded editable open paths with plain
appearances. Structural artwork intersections define the cut boundaries,
rather than the position where the gesture crosses the artwork. Retained
Bézier pieces are subdivided directly rather than approximated by refitting.
A removed middle interval leaves separate subpaths under the original node.

Automatic joining requires compatible appearance/metadata, the same parent,
and an unambiguous pair of remaining endpoints at a new cut. J forces joining
for this gesture, N disables it, C cycles the dashed stroke color, and Escape
cancels. Alt makes a straight rubberband section; Alt+Shift constrains it to
45-degree directions. Prefer Selected limits trimming to selected path roots
when eligible paths are selected; other eligible paths can supply boundaries.

Closed paths are not trimmed. Locked/hidden/out-of-isolation art, clipping
paths, live shapes, brush/variable-width/profile paths and effect paths are
excluded from editing. Cross-style and cross-parent joins are excluded.
Limits: 512 eligible artwork segments, 512 gesture points, 8192 structural
intersection records, 2048 remaining pieces (256 with joining enabled).
Commands refuse excess complexity before editing. Oversized tool gestures
cancel as a whole. Collinear overlaps and stroke hits exactly on cut boundaries
are not resolved as deletions.

## Verification and unresolved items — 2026-11-20

[Run 44573057281](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/44573057281)
passed all configured regression steps, native Windows compilation, installer
generation and isolated installation. All 7 new trim engine regressions passed:
X cut/join/undo, no-join/selected preference, exact cubic subdivision/middle
removal, ambiguous junction/atomic invalid input, closed/locked protection,
atomic segment limit, and native drag/Escape/N gesture behavior.
The tool-level rubberband/angle/override/cancel test and the real egui
registered-tool button tests also passed. The configured prior Core Bridge
and four-tool regression gates passed; this is not every repository test.

The downloaded EXE was verified against the artifact and executable SHA256s.
An isolated local app using synthetic art passed actual X trimming at the
artwork intersection, J/N key routing, C preview color change, Escape,
single-step undo, and visible five-tool panel screenshot inspection. Native
Extend Path (Bézier, snap, direction lock), brush removal and node slide with
undo also passed in the new release EXE.

**Unresolved local H/L/S recheck:** the prior handle keyboard automation did
not select handles / change the SVG in this local session. The same recheck
also failed against Preview 3 in the same environment. Its reason is not
established. CI's H/L/S egui frame regressions passed, but that does not replace
a successful local keyboard recheck. Do not report this local check as passed,
or treat Preview 4 as an unqualified all-tools validated release. Earlier
capture attempts also encountered no-presented-frame replies; usable new-tool
screenshots were subsequently obtained. No user artwork was used or changed.

Installer smoke verified the installed EXE hash/version, required files,
receiver launcher/config and sender placement/hash in a synthetic Chinese
Illustrator installation directory. It did not test a live Illustrator document.

EXE SHA256: e61dd81325d931354a8b41a8f105979988fb47734365f42f5f81ec8fe3e1e942
Installer SHA256: fdd9ea46607238224ba3d506c20304779d1d769c442532db3fea14652d45d0c0

Earlier attempts failed on a document-cache borrow lifetime and an incorrect
ViewInfo test reference; both were corrected before the passing run above.
The README bundled at build time predates these local checks; this branch
records the post-build verification without rebuilding unchanged binaries.

## Historical Preview 3 verification

Baseline: VectorCraft 0.24.1 / Leo Design Core Bridge 0.20.4.
Preview 3 is built from 565d9aac7a5b571b161468b5a416226a64c8ce08.
Core Bridge is retained. No pie menu is restored.

## Download and entry point

[Single Windows x64 installer](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/44514167023/artifacts/14526728092).
Extract the ZIP and run Leo_Design_Path_Tools_Preview_3_Windows_x64_Setup.exe.
Open Window > Leo Design Path Tools. The panel also opens once on upgrade.
The panel uses native docking. If its flyout is short near the bottom of the
window, scroll its contents or drag its title/tab into the main right sidebar.
The new icon may be below the visible icon strip on shorter windows; the Window
menu remains available. This is a preview UI limitation.

## Implemented workflows

- PathScribe: native Direct Selection handle picks, Shift additive/toggle picks,
  batch dragging, H one-shot handle marquee (Shift toggles), L equal lengths and
  S one-shot mirrored symmetry. The panel offers equal length, symmetry,
  numeric rotation in degrees and numeric scaling in percent. Numeric panel
  transforms retain picks and refresh their document revision. Only visible,
  editable artwork-path handles participate; text-on-path keeps its native
  single-handle workflow. Multiple-anchor handle visibility must be enabled.
  Symmetry is a geometric operation, not a persistent coupling constraint.
- Extend Path: drag either endpoint of an editable open path. Native Bézier,
  straight and arc modes remain available. Added snapping to other editable
  open-path endpoints and a terminal-tangent drag-axis lock (checkbox or Shift).
  When locked, only endpoints on that axis qualify for snapping. Lock constrains
  drag direction; curved Bézier extension need not have a straight endpoint locus.
- Smart Remove Brush: select paths, choose the panel tool, then paint over
  anchors. Uses native Anchor Eraser Brush, with brush diameter and tolerance
  controls. A stroke is one undo transaction.
- Reposition Point: drag an interior anchor along its neighboring curves using
  native Slide Anchor. Choosing it from this panel enables the estimated
  fit-error guard. A tight tolerance can limit or refuse movement. Native
  Slide Anchor outside the panel preserves its existing default behavior.

Brush removal and slide operations refit curves within estimated tolerances.
They do not guarantee exact invariance for arbitrary complex curves. Arbitrary
curve families, every zoom and every closed-path topology have not been manually
validated. Extend Path requires open endpoints. The conservative engine commands
path.leoExtend, path.leoSmartRemove and path.leoReposition remain available, but
are separate from the native interactive brush/slide workflows.

## Verified on 2026-11-19

[Packaging run 44514167023](https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/44514167023)
passed all configured regression gates, Windows release compilation, installer
generation and isolated installation. Gates include real egui four-button and
idle H/L/S frame tests, full shortcut regressions, Path Tools engine/geometry
tests, original native Extend/Brush/Slide tests, Direct Selection, and the
configured Core Bridge regressions. This is not a claim that every repository
test or every real Illustrator installation was exercised.

The installer smoke checked installed EXE hash and version, required files,
receiver launcher/configuration and sender placement/hash in a synthetic
Chinese Illustrator installation directory. It did not interact with a person's
live Illustrator document.

The downloaded release EXE was separately SHA256-verified and exercised through
its local control interface in an isolated desktop instance with synthetic art:
- Visible four-tool panel and its controls inspected in native screenshots.
- H marquee, L equal lengths, S symmetry, SVG changes, undo and redo passed.
- Numeric panel input/clicks: 90 degree rotation and 200 percent scale changed
  both selected handles to expected coordinates; revision refresh and undo passed.
- Actual native canvas gestures: Bézier extension, endpoint snapping,
  tangent-axis lock, brush anchor removal and guarded node slide passed.
  Each gesture's undo behavior was checked.

EXE SHA256: 8b942bb28dc0950da4c8520038f02c5384c7f80208b2d93ad78ad989ffad8276
Installer SHA256: 476728e053e4f7f2d5b2c6bf2d64e6d7b942e6f2bb90ae82f168e178dbc8e2ee

Historical run 44512372140 failed an original Shift-slide regression after a
global guard change. Preview 3 fixes it by enabling the guard from the new panel,
while preserving the native default; its packaging run passed the original
regression. Preview 1 artifact 14440179511 is withdrawn because the later actual
EXE check found broken idle H/L/S routing. Preview 2 did not include the new
four-tool panel. Neither is the four-tool delivery.


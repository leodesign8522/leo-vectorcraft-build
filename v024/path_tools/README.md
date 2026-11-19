# Leo Design Path Tools — Preview 3

Development after Preview 3: an independent intersection-based Trim and Join
tool is being added. It is not in the linked Preview 3 installer. Its initial
scope is expanded open paths with plain appearances, exact cubic subdivision,
optional joining at unambiguous compatible endpoints, dashed stroke previews,
Alt rubberband / Shift angle constraint, J/N/C and Escape. Closed-path trimming,
variable-width/brush profile splitting and cross-style/layer joins are excluded.
Native compilation and interaction regressions for this addition are pending.

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

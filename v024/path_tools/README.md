# Leo Design Path Tools — development branch
Baseline: VectorCraft 0.24.1 / Core Bridge 0.20.4.
Not a release. The native Direct Selection tool supports gold hollow hover and
cyan filled selected handle marks, Shift-click additive/toggle picks and batch
drag of selected artwork-path handles. Shift during drag remains the original
angle constraint; Alt retains independent handle editing. Both explicitly
selected handles on one anchor move directly; noncollinear results become
corners. Engine command validates the whole batch before editing and records
one undo transaction. Multi-pick requires native multiple-handle visibility.
Text-on-path handles retain the native single-handle workflow.
Development additions (native CI verified): press H in Direct Selection, then
drag a handle marquee; Shift toggles enclosed handles. The mode ends on release
or Escape. L sets selected handle lengths to the first picked handle's length;
S mirrors each picked handle about its anchor (first pick wins if both sides
are picked). Each batch uses the existing atomic undo command and clears picks.
Only visible, editable artwork handles participate; text-on-path is excluded.

Conservative engine adapters (native CI verified): path.leoExtend appends a
tangent or polynomial cubic continuation at an open path's final endpoint;
path.leoSmartRemove merges an interior join only within the supplied tolerance;
path.leoReposition moves a join on a recoverable source cubic. These require
expanded, editable, open artwork paths. Rejected operations leave art/history
unchanged. These commands are not interactive brush or slide tools.

Remaining: dedicated toolbar, numeric input, handle rotation/scaling, extension
endpoint snapping/direction lock, Smart Remove Brush gesture, Reposition Point
drag gesture, closed-path support and Windows application/installer validation.
The existing Core Bridge single-installer pipeline is retained; no pie menu
is restored. No Path Tools installer has been produced.

Validation on 2026-11-18, source commit 579562772dd9c850e75968208b97a32f37f66167:
- Windows native run 44317336544: 3 Path Tools tests, 88 Direct Selection
  regressions, 2 engine command regressions passed.
- Windows geometry run 44317336520 passed.
- Full migration, Python syntax and core/no-pie separation checks passed.
- Application GUI, complete application build and installer were not validated
  by these focused test workflows.

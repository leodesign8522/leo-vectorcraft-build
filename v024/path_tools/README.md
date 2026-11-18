# Leo Design Path Tools — development branch
Baseline: VectorCraft 0.24.1 / Core Bridge 0.20.4.
This is not a release. Geometry primitives are tested separately. The native
Direct Selection enhancement adds gold hollow hover and cyan filled selected
handle marks, keeping original drag, constraints and snapping. Release retains
the marker; Escape or a new press clears it. Changes are applied through
prepare_v024.py. The native test workflow must pass before packaging.
Remaining: multi-handle selection/transform; dedicated Path Tools toolbar;
extend/cut, smart remove brush and reposition tools with actual engine
transactions. No installer has been built on this branch yet.

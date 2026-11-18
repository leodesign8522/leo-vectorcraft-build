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
Remaining: dedicated toolbar, handle marquee and numeric transformations,
extend/cut, smart remove brush and reposition interactions. No installer yet.

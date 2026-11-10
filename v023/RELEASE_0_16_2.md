# Core Bridge 0.16.2 — editable AI export for tabular digits

This fixes editable AI export rejecting point or area text with the OpenType `tnum` feature. The Illustrator bridge now maps it to tabular lining figures and resets ordinary runs to the default figure style, retaining live text. The character preflight reports actual unsupported field names instead of examples that can misidentify the cause.

The change preserves guards for unmapped OpenType features, variable font axes and other unsupported content. It does not silently strip those properties.

Validation:
- Production character mapper: MiSans Bold font lookup, tnum mapping, ordinary-run reset, missing features, unknown-feature rejection.
- Negative control: the same regression fails against 0.16.1.
- Rust export preflight: RGB and CMYK mixed runs retain their styles and source documents remain unchanged; unknown-feature diagnostics identify `features`.
- Windows x64 build, all configured retained regressions and installed-EXE identity/hash checks: passed.
- Editable AI Rust export preflight: 14 tests passed. Separate snapping regression workflow also passed.
- Local Illustrator COM roundtrip was unavailable because access was denied; live Illustrator export of the user's document has not been rechecked in this session.

Source commit: 1814d99111e919252b4c7cbec68c8e121c19a5b2
Build: https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42650970909

Install the single combined installer, then launch **Leo Design VectorCraft 0.22 Bridge 0.16.2**. Use **文件 → 导出 → 导出可编辑 AI（通过 Illustrator）**. Installed Illustrator and matching fonts are still required.

Installer: https://github.com/leodesign8522/leo-vectorcraft-build/actions/runs/42650970909/artifacts/13693443332
Installer archive SHA-256: `951dbf993d6901c2c7cf08d5d92123b82b49a40de5630b01beb10f010b7a1ba4`

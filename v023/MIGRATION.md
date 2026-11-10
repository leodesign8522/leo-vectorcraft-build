# Core Bridge 0.17.0 / VectorCraft 0.23.0 migration

Upstream is pinned to `5d0e892e2419f63bc0ea3c743ddff2e81e15fd3a` (official v0.23.0).
The retained feature baseline is Core Bridge 0.16.2, commit
`874dcf05820a43d434036fa73c41f448da31a0c5`.

The custom Ctrl+Enter text-editing handler is removed from this migration.
Official text editing handles that shortcut. Action shortcuts and navigation
remain independent, and must preserve the official input tests.

Adapted import provenance to preserve native PDF save metadata; ICO registration
keeps the new HEIF and JPEG XL readers. Live-corner edits retain upstream pixel
alignment code. Action-set import sanitizes shortcuts on both import paths, and
the glyph-selection overlay patch follows the new selection_marks module.

Retain the unified snapping engine, finite and guide intersections, group
selection and movement, glyph bounds, text modes, image/ICO export, workspace
settings, action shortcuts, document statistics and the existing AI bridge.
The deprecated pie-menu and screenshot-paste overrides remain disabled.

Validation status: clean-source patch replay, Rust formatting, all configured
retained regressions, official Ctrl+Enter tests, Windows release compilation and
isolated installer identity/hash checks passed on GitHub Actions run 42825601622.
See [the exact build and scope](RELEASE_0_17_0.md). User-desktop mouse behavior and
live Illustrator COM round trips were not rechecked in this migration.

"""Adapt Core Bridge artwork consumers to the v0.24.1 ruler-guide node."""
from pathlib import Path
import sys

root = Path(sys.argv[1])


def replace(relative_path, before, after):
    path = root / relative_path
    source = path.read_text(encoding="utf-8")
    if source.count(before) != 1:
        raise SystemExit(f"Unexpected ruler-guide compatibility anchor: {relative_path}: {before}")
    path.write_text(source.replace(before, after, 1), encoding="utf-8", newline="\n")


document_info = "crates/engine/src/cmd/docinfo.rs"
replace(
    document_info,
    "NodeKind::Path { guide: true, .. }) {\n        return false;",
    "NodeKind::Path { guide: true, .. } | NodeKind::RulerGuide { .. }) {\n        return false;",
)
replace(
    document_info,
    "NodeKind::Path { guide: true, .. } => (objects, groups),",
    "NodeKind::Path { guide: true, .. } | NodeKind::RulerGuide { .. } => (objects, groups),",
)

replace(
    "crates/ui-egui/src/leo_core_bridge/ai_export.rs",
    "    doc.walk(|n|{\n        if !n.visible",
    "    doc.walk(|n|{\n        if n.is_ruler_guide() { return; }\n        if !n.visible",
)

replace(
    "crates/ui-egui/src/leo_core_bridge/ai_export_tests.rs",
    '    assert!(prepare(&framed,"far-rotated").is_ok());',
    '''    assert!(prepare(&framed,"far-rotated").is_ok());

    let mut with_guide = framed.clone();
    let guide = vectorcraft_doc::guides::Guide::new(true, 14200.0);
    with_guide.add_ruler_guide(&guide);
    assert_eq!(simple_geometry_witness(&with_guide), Some(witness));''',
)

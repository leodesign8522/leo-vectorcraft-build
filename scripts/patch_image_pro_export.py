#!/usr/bin/env python3
"""Integrate the native Leo Design Image Pro Export dialog into VectorCraft v0.17.0.

The dialog's source is kept as a separate module in this repo and uses the
upstream JPEG/PNG/ICC/batch artboard exporting implementation.
"""
from pathlib import Path
import sys

if len(sys.argv) != 3:
    raise SystemExit("Usage: patch_image_pro_export.py VECTORCRAFT_ROOT LEO_DIALOG_SOURCE")
root = Path(sys.argv[1])
source = Path(sys.argv[2])
if not source.is_file():
    raise SystemExit(f"Missing Leo Design dialog source: {source}")


def replace_once(path: Path, before: str, after: str, label: str):
    original = path.read_text(encoding="utf-8")
    count = original.count(before)
    if count != 1:
        raise SystemExit(f"FAIL {label}: expected one upstream anchor, got {count}")
    path.write_text(original.replace(before, after, 1), encoding="utf-8", newline="\n")


dialogs = root / "crates/ui-egui/src/dialogs/mod.rs"
replace_once(dialogs,
             "mod export_for_screens;\n",
             "mod export_for_screens;\nmod leo_image_export;\n",
             "Register Leo Design dialog module")
replace_once(dialogs,
             "pub use export_for_screens::open as open_export_for_screens;\n",
             "pub use export_for_screens::open as open_export_for_screens;\n"
             "pub use leo_image_export::open as open_leo_image_export;\n",
             "Expose Leo Design export dialog")
replace_once(dialogs,
             "    ExportForScreens: [\"exportForScreens\"] => export_for_screens::SPEC,\n",
             "    ExportForScreens: [\"exportForScreens\"] => export_for_screens::SPEC,\n"
             "    LeoImageExport: [leo_image_export::KIND] => leo_image_export::SPEC,\n",
             "Add dialog registration")

menus = root / "crates/ui-egui/src/menus.rs"
replace_once(menus,
             "        \"file.exportForScreens\" if p.as_object().is_none_or(|o| o.is_empty()) => {\n",
             "        \"file.leoImageProExport\" => {\n"
             "            crate::dialogs::open_leo_image_export(app);\n"
             "            Ok(Value::Null)\n"
             "        }\n"
             "        \"file.exportForScreens\" if p.as_object().is_none_or(|o| o.is_empty()) => {\n",
             "Wire Leo Design menu command")
replace_once(menus,
             "        | \"file.exportForScreens\"\n        | \"file.documentSetup\"\n",
             "        | \"file.exportForScreens\"\n        | \"file.leoImageProExport\"\n        | \"file.documentSetup\"\n",
             "Enable command only with active document")
replace_once(menus,
             "                         c(\"Export for Screens…\", \"file.exportForScreens\"),\n",
             "                         c(\"Leo Design · 图像专业导出…\", \"file.leoImageProExport\"),\n"
             "                         Sep,\n"
             "                         c(\"Export for Screens…\", \"file.exportForScreens\"),\n",
             "Add native export menu entry")

target = root / "crates/ui-egui/src/dialogs/leo_image_export.rs"
if target.exists():
    raise SystemExit("Refusing to overwrite an existing Leo Design image export dialog")
target.write_bytes(source.read_bytes())
print("SUCCESS: Leo Design Image Pro Export native menu and dialog installed.")

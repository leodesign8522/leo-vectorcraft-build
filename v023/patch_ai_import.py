#!/usr/bin/env python3
"""Editable-first AI entry and import provenance; no Adobe automation."""
from pathlib import Path
import sys
root = Path(sys.argv[1])
def replace(rel, old, new):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    assert s.count(old) == 1, (rel, s.count(old))
    p.write_text(s.replace(old, new, 1), encoding='utf-8', newline='\n')
replace('crates/ui-egui/src/leo_core_bridge/mod.rs', 'pub mod workspace_io;', 'pub mod workspace_io;\npub mod ai_import;')
replace('crates/ui-egui/src/menus/tree.rs', '                c("Open��", "file.open"),', '                c("Open��", "file.open"),\n                c("Leo Design �� AI��ԭʼ�༭�������ȣ���", "file.leoOpenAi"),')
replace('crates/ui-egui/src/menus/ui_commands.rs', 'pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[', 'pub const UI_COMMANDS: &[(&str, &str, &str, &str)] = &[\n    ("file.leoOpenAi", "Leo Design �� AI��ԭʼ�༭�������ȣ�", "", "{path?} open AI with editable text/layers and import report"),')
replace('crates/ui-egui/src/menus/run.rs', '        "file.open" => match s("path") {', '        "file.leoOpenAi" => crate::leo_core_bridge::ai_import::open(app, p),\n        "file.open" => match s("path") {')
replace('crates/engine/src/cmd/fileio/load.rs', 'let Loaded { mut doc, format, warnings, restored, converted, native_pdf_compatible, .. } = loaded.with_grid_prefs(&s.prefs);', 'let Loaded { mut doc, format, warnings, restored, converted, native_pdf_compatible, editing_data, .. } = loaded.with_grid_prefs(&s.prefs);')
replace('crates/engine/src/cmd/fileio/load.rs', '"warnings": warnings, "restored": restored }),', '"warnings": warnings, "restored": restored, "editingDataRead": editing_data }),')
print('SUCCESS editable-first AI import and provenance report')

p=root/'crates/engine/src/cmd/fileio/tests_aiimport.rs'
s=p.read_text(encoding="utf-8")
s+=r'''
#[test]
fn leo_ai_import_provenance_distinguishes_layers_from_page() {
    let file=ai::ai(&compressed(&ai::sample_data()),ai::page_pdf());
    let mut session=Session::new();
    let r=open(&mut session,"sample.ai",&file,json!({"editingData":"always"})).unwrap();
    assert_eq!(r["editingDataRead"],true);
    check_sample(&session);
    let r=open(&mut session,"sample.ai",&file,json!({"editingData":false})).unwrap();
    assert_eq!(r["editingDataRead"],false);
}
'''
p.write_text(s,encoding='utf-8',newline='\n')


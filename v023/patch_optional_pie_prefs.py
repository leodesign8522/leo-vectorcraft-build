#!/usr/bin/env python3
"""Retain dormant pie JSON in the no-pie build; never register UI/input hooks."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
state = root / 'crates/ui-egui/src/state.rs'
s = state.read_text(encoding='utf-8')
before = '    pub palette_query: String,\n    /// Screen mode:'
after = '''    pub palette_query: String,
    /// Opaque optional-add-on preferences, preserved without loading pie code.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub leo_pies: serde_json::Value,
    /// Screen mode:'''
assert s.count(before) == 1, 'unexpected UiState fields'
s = s.replace(before, after, 1)
before = '            palette_query: String::new(),\n            screen_mode: 0,'
after = '''            palette_query: String::new(),
            leo_pies: serde_json::Value::Null,
            screen_mode: 0,'''
assert s.count(before) == 1, 'unexpected UiState defaults'
state.write_text(s.replace(before, after, 1), encoding='utf-8')
lib = root / 'crates/ui-egui/src/lib.rs'
with lib.open('a', encoding='utf-8') as f:
    f.write('''
#[cfg(test)]
mod leo_optional_prefs_tests {
    #[test]
    fn dormant_pie_json_survives_core_roundtrip_with_other_preferences() {
        let saved = serde_json::json!([{
            "name":"旧菜单", "keyboard":"Shift+O", "futureField":{"keep":true},
            "slots":[{"command":"object.align", "params":{"horizontal":"left"}, "submenu":1}]
        }]);
        let mut ui = crate::state::UiState::default();
        ui.leo_pies = saved.clone();
        ui.palette_query = "other preference retained".into();
        let raw = serde_json::to_value(&ui).unwrap();
        let restored: crate::state::UiState = serde_json::from_value(raw).unwrap();
        assert_eq!(restored.leo_pies, saved);
        assert_eq!(restored.palette_query, ui.palette_query);
    }
    #[test]
    fn no_addon_does_not_create_pie_configuration() {
        let ui = crate::state::UiState::default();
        assert!(ui.leo_pies.is_null());
        let raw = serde_json::to_value(&ui).unwrap();
        let roundtrip: crate::state::UiState = serde_json::from_value(raw).unwrap();
        assert!(roundtrip.leo_pies.is_null());
    }
}
''')
print('Core only: preserved opaque preferences; no pie menu module or hooks')

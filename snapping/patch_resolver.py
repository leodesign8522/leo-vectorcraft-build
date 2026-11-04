#!/usr/bin/env python3
"""Install the independently owned point resolver after compatibility adapters."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
path = root / 'crates/tools/src/guides/targets.rs'
source = path.read_text(encoding='utf-8')
marker = '    fn snap_from_inner(&self, p: Point, from: Option<&Leave>, tol: f64) -> (Point, Vec<Overlay>) {'
if source.count(marker) != 1:
    raise SystemExit('Unexpected point resolver signature')
start = source.index(marker)
end = source.index('\n    /// Where a line from `at`', start)
source = source[:start] + marker + '\n        self.leo_snap_from(p, from, tol)\n    }\n' + source[end:]
field = 'pub struct Targets {\n'
assert source.count(field) == 1
source = source.replace(field, field + '    leo_point_lock: std::cell::Cell<Option<(Point, Kind)>>,\n', 1)
source = '#[path = "leo_snapping.rs"]\nmod leo_snapping;\n\n' + source
# Respect hidden guide feedback while continuing to compute snaps.
source = source.replace('        if let Some((end, projected, dir)) = preview {',
                        '        if self.style.lines && let Some((end, projected, dir)) = preview {')
path.write_text(source, encoding='utf-8', newline='\n')
module = Path(__file__).with_name('leo_snapping.rs').read_text(encoding='utf-8')
path.with_name('leo_snapping.rs').write_text(module, encoding='utf-8', newline='\n')
# Keep all custom snapping controls in their own preference category.
spec = root / 'crates/engine/src/cmd/prefscmds.rs'
s = spec.read_text(encoding='utf-8')
assert '"Smart Guides"' in s
spec.write_text(s.replace('"Smart Guides"', '"Leo Snapping"'), encoding='utf-8', newline='\n')
dialog = root / 'crates/ui-egui/src/prefs_dialog.rs'
s = dialog.read_text(encoding='utf-8')
dialog.write_text(s.replace('"Smart Guides"', '"Leo Snapping"'), encoding='utf-8', newline='\n')
print('PASS independent Leo Snapping resolver and preference category')

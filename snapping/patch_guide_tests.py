#!/usr/bin/env python3
"""Migrate native expectations for deliberately changed preview and colour semantics."""
from pathlib import Path
import sys

p = Path(sys.argv[1]) / 'crates/tools/src/guides/tests.rs'
s = p.read_text(encoding='utf-8')
def change(old, new):
    global s
    assert s.count(old) == 1, f'Unexpected guide test anchor: {old[:90]}'
    s = s.replace(old, new, 1)

channels = ['horizontal', 'vertical', 'spacing', 'construction', 'distance', 'distance_label', 'label', 'anchor', 'center', 'path']
change('let c = ToolContext { smart_guide_color: green, ..cx(&d, &s, &p) };',
       'let c = ToolContext { smart_guide_color: green, ' + ', '.join('smart_guide_' + c + '_color: green' for c in channels) + ', ..cx(&d, &s, &p) };')
change('''    // 150 along it (2.5 off), and 50 along it (shorter).
    for (pointer, on) in [(Point::new(421.5, 312.0), Point::new(420.0, 310.0)), (Point::new(341.5, 372.0), Point::new(340.0, 370.0))] {
        let (q, ov) = t.snap_point(pointer, 4.0);
        assert!((q - on).hypot() < 1e-9, "{q:?}");
        assert_eq!(lines(&ov), vec![(Point::new(300.0, 400.0), q)]);
    }''', '''    // Only the forward ray beyond the original endpoint is an extension target.
    let (q, ov) = t.snap_point(Point::new(421.5, 312.0), 4.0);
    assert!((q - Point::new(420.0, 310.0)).hypot() < 1e-9, "{q:?}");
    assert_eq!(lines(&ov), vec![(Point::new(380.0, 340.0), q)]);
    assert!(labels(&ov).contains(&"延长线吸附"));
    let pointer = Point::new(341.5, 372.0);
    let (q, ov) = t.snap_point(pointer, 4.0);
    assert_eq!(q, pointer, "shortening must not acquire a forward-extension lock");
    assert!(!labels(&ov).contains(&"延长线吸附"));''')
change('assert_eq!(lines(&ov), vec![(Point::new(100.0, 300.0), q)]);',
       'assert_eq!(lines(&ov), vec![(Point::new(300.0, 301.0), q)]);')
change('assert_eq!(lines(&ov).len(), 1, "{ov:?}");\n    // Nothing in reach:',
       'assert_eq!(lines(&ov).len(), 2, "constraint ray plus target alignment: {ov:?}");\n    // Nothing in reach:')
change('assert_eq!((q, ov.len()), (Point::new(300.0, 380.0), 0));',
       'assert_eq!((q, ov.len()), (Point::new(300.0, 380.0), 1));\n    assert_eq!(lines(&ov), vec![(Point::new(300.0, 300.0), q)]);')
p.write_text(s, encoding='utf-8', newline='\n')
print('PASS native guide tests retain geometry checks with independent colour/preview expectations')

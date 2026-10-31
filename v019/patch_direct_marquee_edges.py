#!/usr/bin/env python3
"""LC-017: v0.19 supplies segment marquee selection natively; do not patch it twice."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
s=(root/'crates/tools/src/direct.rs').read_text(encoding='utf-8')
for marker in ['fn crossed_segments(', 'let segs = crossed_segments(path, r);', '"select.segments"']:
    if marker not in s: raise SystemExit(f'FAIL native segment selection missing: {marker}')
print('SUCCESS LC-017: official v0.19 segment marquee retained; no legacy patch applied')

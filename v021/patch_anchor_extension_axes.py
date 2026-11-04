#!/usr/bin/env python3
"""LC-024 companion: gather all straight-line directions and keep tests/lints exact."""
from pathlib import Path
import sys
root = Path(sys.argv[1])

def change(rel, old, new, title):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    if s.count(old) != 1:
        raise SystemExit(f"FAIL LC-024 {title}: {s.count(old)}")
    p.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("PASS LC-024", title)

targets = "crates/tools/src/guides/targets.rs"
change(targets,
       "            if len.is_finite() && d.x.abs() > 1e-9 * len && d.y.abs() > 1e-9 * len {\n",
       "            if len.is_finite() && len > 1e-9 {\n",
       "collect horizontal, vertical and oblique source extensions")
old_comment = (
    "    /// The segment extension nearest " + chr(96) + "p" + chr(96) + ", within " + chr(96) + "tol" + chr(96) + " of it (and " + chr(96) + "p" + chr(96) + " farther than that from the\n"
    "    /// end it starts at): that end, its direction and where " + chr(96) + "p" + chr(96) + " lands on it. Where " + chr(96) + "p" + chr(96) + " is nearer\n"
    "    /// the horizontal or the vertical through that end there is none: lining up with it (\"align\")\n"
    "    /// is what straightens a segment that is nearly level or upright.\n"
)
change(targets, old_comment,
       "    /// The nearest forward ray of a straight segment ending at the dragged anchor.\n",
       "update extension documentation")
change(targets,
       "        for (bounds, segment) in self.segments.iter().filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q)) {\n",
       "        for (_, segment) in self.segments.iter().filter(|(bounds, _)| bounds.inflate(reach, reach).contains(q)) {\n",
       "keep actual-edge intersection loop warning-free")

change(targets,
       "            return match self.slide(at, dir, q, tol, false) {\n"
       "                Some((q, target)) => (q, std::iter::once(construction(q)).chain(self.lined_up(q, target)).collect()),\n"
       "                None => (q, vec![construction(q)]),\n"
       "            };",
       "            return match self.slide(at, dir, q, tol, false) {\n"
       "                Some((hit, target, kind)) => {\n"
       "                    let mut guides = vec![construction(hit)];\n"
       "                    if kind == \"intersect\" && self.nearest_point(hit, 1e-9).is_none() {\n"
       "                        guides.extend(self.label(hit, \"intersect\"));\n"
       "                    } else {\n"
       "                        guides.extend(self.lined_up(hit, target));\n"
       "                    }\n"
       "                    (hit, guides)\n"
       "                }\n"
       "                None => (q, vec![construction(q)]),\n"
       "            };",
       "update construction-guide slide caller for triple result")
change(targets,
       "        match self.slide(p, dir, p, tol, false) {\n"
       "            Some((q, from)) => (q, self.lined_up(q, from)),\n"
       "            None => (p, vec![]),\n"
       "        }",
       "        match self.slide(p, dir, p, tol, false) {\n"
       "            Some((q, from, kind)) => {\n"
       "                if kind == \"intersect\" && self.nearest_point(q, 1e-9).is_none() {\n"
       "                    (q, self.label(q, \"intersect\").into_iter().collect())\n"
       "                } else {\n"
       "                    (q, self.lined_up(q, from))\n"
       "                }\n"
       "            }\n"
       "            None => (p, vec![]),\n"
       "        }",
       "update snap-point-along slide caller for triple result")

direct_tests = "crates/tools/src/direct/tests.rs"
change(direct_tests,
       "shapes::line(Point::new(400.0, 300.0), Point::new(400.0, 350.0))",
       "shapes::line(Point::new(390.0, 300.0), Point::new(410.0, 350.0))",
       "exercise arbitrary extension against a slanted target edge")

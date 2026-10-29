#!/usr/bin/env python3
"""Leo Design v0.5.4: fix encoded CJK font names in Illustrator PDF clipboard.

Run after patch_illustrator_text_bridge.py against upstream v0.17.0.
A copied PDF can expose a CID encoding suffix like -GBpc-EUC-H after
the actual PostScript face name. The suffix is *not* part of the font.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_pdf_cjk_font_names.py VECTORCRAFT_ROOT")
p = Path(sys.argv[1]) / "crates/pdf/src/import_text.rs"
s = p.read_text(encoding="utf-8")

old = """        // A subset's six-letter tag.
        let name = match name.split_once('+') {
            Some((tag, rest)) if tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase()) => rest,
            _ => name,
        };
        // An installed face of that exact PostScript name: its own family and style.
        if let Some((family, style)) = vectorcraft_text::FontDb::global().by_postscript_name(name) {"""
new = """        // A subset's six-letter tag.
        let name = match name.split_once('+') {
            Some((tag, rest)) if tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase()) => rest,
            _ => name,
        };
        // Leo Design: the PDF importer sometimes receives an encoding appended
        // to a CJK face name, e.g. MiSans-Regular-GBpc-EUC-H. It is not part
        // of the installed font family or its weight/style. Do not strip an
        // arbitrary trailing -H: only recognized Adobe CJK encoding names.
        let cjk_encodings = [
            "-GBpc-EUC-H", "-GBpc-EUC-V",
            "-GBK-EUC-H", "-GBK-EUC-V",
            "-GBK2K-H", "-GBK2K-V",
            "-UniGB-UCS2-H", "-UniGB-UCS2-V",
            "-UniGB-UTF16-H", "-UniGB-UTF16-V",
            "-Identity-H", "-Identity-V",
        ];
        let decoded = cjk_encodings.iter().find_map(|suffix| name.strip_suffix(suffix));
        let name = decoded.unwrap_or(name);
        // Prefer a real installed PostScript face (e.g. MiSans-Regular)
        // over a lossy family/style guess.
        if let Some((family, style)) = vectorcraft_text::FontDb::global().by_postscript_name(name) {"""
if s.count(old) != 1:
    raise SystemExit(f"FAIL CJK PDF font anchor: {s.count(old)}")
s = s.replace(old,new,1)
old2 = """            None => (spaced(strip_ps(fam)).trim().to_string(), style, false),
"""
new2 = """            // For encoded PDF CID font names, retain the literal family on
            // failure instead of turning MiSans into Mi Sans. Missing fonts
            // must remain discoverable by the Find Font dialog.
            None if decoded.is_some() => (strip_ps(fam).trim().to_string(), style, false),
            None => (spaced(strip_ps(fam)).trim().to_string(), style, false),
"""
if s.count(old2)!=1:
    raise SystemExit(f"FAIL missing-family anchor: {s.count(old2)}")
s=s.replace(old2,new2,1)
# Add deterministic regression regardless of installed system fonts.
tag = "    #[test]\n    fn leo_pdf_cid_suffix_stays_out_of_the_family_name()"
test = """    #[test]
    fn leo_pdf_cid_suffix_stays_out_of_the_family_name() {
        // Empty family catalog, like a machine without the source font.
        let installed = Families(HashMap::new());
        let (family, style, found) =
            installed.resolve("MiSans-Regular-GBpc-EUC-H", None, false);
        assert_eq!(family, "MiSans");
        assert_eq!(style, "Regular");
        assert!(!found);
        let (family, style, found) =
            installed.resolve("ABCDEF+MiSans-Bold-UniGB-UCS2-H", None, false);
        assert_eq!((family, style, found), ("MiSans".to_string(), "Bold".to_string(), false));
        // A conventional Latin name must not lose a real suffix.
        assert_eq!(installed.resolve("SomeSans-ExtraBold", None, false).1, "Extra Bold");
    }

"""
needle="""    #[test]
    fn mixed_chinese_and_latin_names_resolve_to_correct_family() {"""
if s.count(needle)!=1 or tag in s:
    raise SystemExit("FAIL inserting CJK PDF regression test")
s=s.replace(needle,test+needle,1)
p.write_text(s,encoding="utf-8",newline="\n")
print("SUCCESS: Illustrator PDF CJK encoding suffix ignored; live font family/style preserved")

#!/usr/bin/env python3
"""Leo Design Illustrator Text Bridge v1.0 (preview), for VectorCraft v0.17.0.

Improve editable text fidelity when Illustrator supplies PDF/SVG clipboard data.
1. Keep Unicode CJK family names distinct during PDF font matching.
2. Resolve PostScript font family+style names in imported SVG text.
The existing copy/paste and import engines remain in place; no fonts are embedded.
"""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_illustrator_text_bridge.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])


def patch(path: str, before: str, after: str, label: str) -> None:
    file = root / path
    source = file.read_text(encoding="utf-8")
    count = source.count(before)
    if count != 1:
        raise SystemExit(f"FAIL {label}: expected exactly one upstream anchor, found {count}")
    file.write_text(source.replace(before, after, 1), encoding="utf-8", newline="\n")
    print(f"OK {label}")


patch(
    "crates/pdf/src/import_text.rs",
    """fn norm(s: &str) -> String {
    s.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_lowercase()).collect()
}""",
    """fn norm(s: &str) -> String {
    // Leo Design: Illustrator/PDF can supply localized CJK font family names.
    // ASCII-only filtering collapsed every Chinese-only family to the empty
    // string: the last installed CJK font then replaced unrelated families.
    // Preserve Unicode letters and digits while ignoring punctuation/spaces.
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}""",
    "PDF Unicode font family matching",
)

patch(
    "crates/svg/src/import/text.rs",
    """        let css = &self.css;
        let mut st = CharStyle::default();
        if let Some(f) = css.prop(n, "font-family") {
            let fam = f.split(',').next().unwrap_or("").trim().trim_matches(|c| c == '\\'' || c == '"').to_string();
            if !fam.is_empty() {
                st.font_family = fam;
            }
        }
        let size = self.font_size(n);
        st.size = size;
        let italic = css.prop(n, "font-style").is_some_and(|s| s == "italic" || s == "oblique");
        st.font_style = style_name(self.weight(n), italic);""",
    """        let css = &self.css;
        let mut st = CharStyle::default();
        // Leo Design: AI SVG text often supplies a PostScript face name
        // (e.g. SourceHanSansCN-Medium) instead of a font family. Keeping it
        // as the family and assigning Regular loses both family and weight.
        // Resolve against installed fonts while preserving a missing font's
        // original name for Find Font / later installation.
        let mut postscript_style = None;
        if let Some(f) = css.prop(n, "font-family") {
            let fam = f.split(',').next().unwrap_or("").trim().trim_matches(|c| c == '\\'' || c == '"').to_string();
            if !fam.is_empty() {
                if let Some((family, style)) = FontDb::global().by_postscript_name(&fam) {
                    st.font_family = family;
                    postscript_style = Some(style);
                } else {
                    st.font_family = fam;
                }
            }
        }
        let size = self.font_size(n);
        st.size = size;
        let italic = css.prop(n, "font-style").is_some_and(|s| s == "italic" || s == "oblique");
        // An explicit CSS weight/slant must override a style inferred from
        // the PostScript name. Otherwise retain that installed face's style.
        st.font_style = if css.prop(n, "font-weight").is_some() || css.prop(n, "font-style").is_some() {
            style_name(self.weight(n), italic)
        } else {
            postscript_style.unwrap_or_else(|| style_name(self.weight(n), italic))
        };""",
    "SVG Illustrator PostScript font + weight matching",
)

pdf = root / "crates/pdf/src/import_text.rs"
src = pdf.read_text(encoding="utf-8")
tag = "#[cfg(test)]\nmod leo_illustrator_text_bridge_tests"
if tag in src:
    raise SystemExit("FAIL PDF tests are already present")
src += """

// Leo Design regression: CJK family names in Illustrator PDF must not collapse
// to the same normalized key, while conventional Latin aliases still match.
#[cfg(test)]
mod leo_illustrator_text_bridge_tests {
    use super::*;

    #[test]
    fn cjk_families_have_distinct_unicode_keys() {
        assert_eq!(norm("思源黑体 CN"), "思源黑体cn");
        assert_eq!(norm("微软雅黑"), "微软雅黑");
        assert_ne!(norm("思源黑体"), norm("微软雅黑"));
        assert_eq!(norm("Source Han Sans CN"), norm("SourceHanSansCN"));
    }

    #[test]
    fn mixed_chinese_and_latin_names_resolve_to_correct_family() {
        let mut names = HashMap::new();
        for family in ["思源黑体", "微软雅黑", "Source Han Sans CN"] {
            names.insert(norm(family), family.to_string());
        }
        let installed = Families(names);
        assert_eq!(installed.resolve("思源黑体-Regular", None, false).0, "思源黑体");
        assert_eq!(installed.resolve("微软雅黑", None, false).0, "微软雅黑");
        assert_eq!(installed.resolve("SourceHanSansCN-Regular", None, false).0, "Source Han Sans CN");
    }
}
"""
pdf.write_text(src, encoding="utf-8", newline="\n")
print("OK PDF CJK font regression tests")
print("SUCCESS Leo Design Illustrator Text Bridge v1.0 (preview)")

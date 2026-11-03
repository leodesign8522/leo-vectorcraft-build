#!/usr/bin/env python3
"""Normalize CID encoding suffixes on SVG clipboard font aliases after text bridge."""
from pathlib import Path
import sys
root=Path(sys.argv[1])
p=root/'crates/svg/src/import/text.rs'
s=p.read_text(encoding="utf-8")
a='''            if !fam.is_empty() {
                if let Some((family, style)) = FontDb::global().by_postscript_name(&fam) {'''
b='''            let fam = leo_clipboard_font_alias(&fam).to_string();
            if !fam.is_empty() {
                if let Some((family, style)) = FontDb::global().by_postscript_name(&fam) {'''
assert s.count(a)==1,'SVG font lookup anchor changed'
s=s.replace(a,b,1)
a='''                } else {
                    st.font_family = fam;
                }
            }
        }
        let size = self.font_size(n);'''
b='''                } else if let Some((family, style)) = leo_clipboard_font_parts(&fam) {
                    st.font_family = FontDb::global().find_family(family).unwrap_or_else(|| family.to_owned());
                    postscript_style = Some(style.to_owned());
                } else {
                    st.font_family = FontDb::global().find_family(&fam).unwrap_or(fam);
                }
            }
        }
        let size = self.font_size(n);'''
assert s.count(a)==1,'SVG fallback anchor changed'
s=s.replace(a,b,1)
s+=r'''

// Registered CID encoding identifiers describe glyph encoding, not a font family.
fn leo_clipboard_font_alias(name: &str) -> &str {
    let name = match name.split_once('+') {
        Some((tag,rest)) if tag.len()==6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => name,
    };
    ["-GBpc-EUC-H", "-GBpc-EUC-V", "-GBK-EUC-H", "-GBK-EUC-V",
     "-GBK2K-H", "-GBK2K-V", "-UniGB-UCS2-H", "-UniGB-UCS2-V",
     "-UniGB-UTF16-H", "-UniGB-UTF16-V", "-Identity-H", "-Identity-V"]
        .iter().find_map(|suffix| name.strip_suffix(suffix)).unwrap_or(name)
}
fn leo_clipboard_font_parts(name: &str) -> Option<(&str, &'static str)> {
    let (family, style) = name.rsplit_once('-')?;
    if family.is_empty() { return None; }
    let style = match style {
        "Regular" | "Roman" => "Regular",
        "Light" => "Light", "ExtraLight" => "Extra Light", "Thin" => "Thin",
        "Medium" => "Medium", "Semibold" | "SemiBold" => "Semi Bold",
        "Bold" => "Bold", "ExtraBold" => "Extra Bold", "Black" => "Black",
        "Italic" => "Italic", "BoldItalic" => "Bold Italic",
        _ => return None,
    };
    Some((family,style))
}
#[cfg(test)]
mod leo_svg_clipboard_fonts {
    use super::*;
    #[test] fn cid_alias_and_style_are_separate() {
        for name in ["MiSans-Regular-GBpc-EUC-H", "ABCDEF+MiSans-Regular-GBpc-EUC-H"] {
            let clean=leo_clipboard_font_alias(name);
            assert_eq!(clean,"MiSans-Regular");
            assert_eq!(leo_clipboard_font_parts(clean),Some(("MiSans","Regular")));
        }
        assert_eq!(leo_clipboard_font_parts(leo_clipboard_font_alias("SourceHanSansCN-Bold-UniGB-UTF16-H")),Some(("SourceHanSansCN","Bold")));
        assert_eq!(leo_clipboard_font_alias("Some-Family-H"),"Some-Family-H");
        assert_eq!(leo_clipboard_font_parts("Some-Family-H"),None);
        assert_eq!(leo_clipboard_font_parts("Some-Family-Regular"),Some(("Some-Family","Regular")));
    }
    #[test] fn imported_svg_keeps_editable_text_and_cleans_font_name() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="100"><text x="10" y="30" font-family="MiSans-Regular-GBpc-EUC-H" font-size="20">装备演示区 Ground Equipment</text></svg>"#;
        let doc = crate::import(svg).unwrap();
        let mut styles = Vec::new();
        doc.walk(|node| {
            if let vectorcraft_doc::NodeKind::Text(text) = &node.kind {
                styles.push((text.first_style().font_family.clone(), text.first_style().font_style.clone()));
            }
        });
        assert!(!styles.is_empty(), "text must remain editable");
        for (family,style) in styles {
            assert_eq!(family,"MiSans");
            assert_eq!(style,"Regular");
        }
    }

}
'''
p.write_text(s,encoding='utf-8',newline='\n')
print('SUCCESS SVG clipboard CID font alias normalization, preserving explicit CSS weight/slant')


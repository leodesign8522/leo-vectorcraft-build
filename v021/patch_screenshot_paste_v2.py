#!/usr/bin/env python3
"""LC-023: preserve paste-time modifiers; decode all Windows bitmap candidates.

Apply after patch_screenshot_paste.py, against the pinned v0.21 baseline.
"""
from pathlib import Path
import sys

root = Path(sys.argv[1])

def change(rel, old, new):
    p = root / rel
    text = p.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"FAIL screenshot v2 anchor {rel}: {text.count(old)}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

keys = "crates/ui-egui/src/shortcuts/paste.rs"
change(keys, "    pressed: Vec<Key>,", "    pressed: Vec<Key>,\n    /// Modifiers when V went down, before Ctrl may be released.\n    leo_paste_held: Option<Modifiers>,")
change(keys, "                egui::Event::Key { key, pressed: true, .. } if PASTE_KEYS.contains(key) && !self.pressed.contains(key) => self.pressed.push(*key),", """                egui::Event::Key { key, pressed: true, modifiers, .. } if PASTE_KEYS.contains(key) && !self.pressed.contains(key) => {
                    self.pressed.push(*key);
                    self.leo_paste_held = Some(*modifiers);
                }""")
change(keys, """                    let shortcut = (*key == Key::Paste) || (*key == Key::V && modifiers.command);
                    if shortcut && !handled {
                        fire = Some(*modifiers);
                    }""", """                    let held = self.leo_paste_held.take().unwrap_or(*modifiers);
                    let shortcut = (*key == Key::Paste) || (*key == Key::V && held.command);
                    if shortcut && !handled {
                        fire = Some(held);
                    }""")

clip = "apps/vectorcraft/src/clipboard.rs"
change(clip, '        PNG => &["PNG"],', '        PNG => &["PNG", "image/png"],')
change(clip, "            BITMAP if read_registered(PNG, &mut data) => PNG,\n            BITMAP => {\n                return read_dibs(|id| {\n                    let mut dib = vec![];\n                    raw::get_vec(id, &mut dib).ok()?;\n                    Some(dib)\n                });\n            }", "            BITMAP => return read_bitmap_candidates(),")
change(clip, "    /// One flavour, while the clipboard is open. A device-independent bitmap is copied here, and", """    /// Snapshot every advertised picture while the clipboard is locked. A nonempty
    /// PNG/DIB is not proof that it will decode; try the other snapshots on failure.
    fn read_bitmap_candidates() -> Option<ClipboardRead> {
        let pngs: Vec<Vec<u8>> = formats_of(PNG).filter_map(|id| {
            let mut bytes = vec![];
            raw::get_vec(id, &mut bytes).ok()?;
            (!bytes.is_empty()).then_some(bytes)
        }).collect();
        let dibs: Vec<Vec<u8>> = DIBS.into_iter().filter_map(|id| {
            let mut bytes = vec![];
            raw::get_vec(id, &mut bytes).ok()?;
            (!bytes.is_empty()).then_some(bytes)
        }).collect();
        let gdi = raw::is_format_avail(formats::CF_BITMAP);
        if pngs.is_empty() && dibs.is_empty() && !gdi { return None; }
        let seq = raw::seq_num();
        Some(ClipboardRead::Later(Box::new(move || {
            super::leo_bitmap_candidates(pngs, dibs, || {
                // Never substitute an image copied after this paste began.
                let expected = seq?;
                if raw::seq_num() != Some(expected) { return None; }
                let mut clipboard = arboard::Clipboard::new().ok()?;
                let captured = clipboard.get_image().ok()?;
                if raw::seq_num() != Some(expected) { return None; }
                let (w, h) = (u32::try_from(captured.width).ok()?, u32::try_from(captured.height).ok()?);
                if w == 0 || h == 0 || w > super::MAX_SIDE || h > super::MAX_SIDE { return None; }
                let rgba = image::RgbaImage::from_raw(w, h, captured.bytes.into_owned())?;
                png_flavour(&rgba)
            })
        })))
    }

    /// One flavour, while the clipboard is open. A device-independent bitmap is copied here, and""")
change(clip, "#[cfg(windows)]\nmod win {", """/// All captured clipboard formats, decoded in preference order. A failed preferred
/// format cannot hide a valid secondary DIB or the OS image fallback. Keep failed
/// image bytes when nothing decodes, so the normal paste decoder reports an error
/// instead of silently pasting a stale internal clipboard.
#[cfg_attr(not(windows), allow(dead_code))]
fn leo_bitmap_candidates(
    pngs: Vec<Vec<u8>>, dibs: Vec<Vec<u8>>, fallback: impl FnOnce() -> Option<Flavour>,
) -> Option<Flavour> {
    for data in &pngs {
        let mut reader = image::ImageReader::with_format(Cursor::new(data), image::ImageFormat::Png);
        let mut limits = image::Limits::default();
        (limits.max_image_width, limits.max_image_height) = (Some(MAX_SIDE), Some(MAX_SIDE));
        reader.limits(limits);
        if reader.decode().is_ok() { return Some(Flavour { mime: PNG, data: data.clone() }); }
    }
    for dib in &dibs {
        if let Some(f) = dib_image(dib).and_then(|img| png_flavour(&img)) { return Some(f); }
    }
    fallback().or_else(|| {
        pngs.into_iter().next().map(|data| Flavour { mime: PNG, data })
            .or_else(|| dibs.into_iter().next().map(|dib| Flavour {
                mime: "image/bmp", data: dib_file(&dib).unwrap_or(dib),
            }))
    })
}

#[cfg(windows)]
mod win {""")

with (root / keys).open("a", encoding="utf-8") as f:
    f.write('''
#[cfg(test)]
mod leo_pixpin_key_tests {
    use super::*;
    fn key(pressed: bool, modifiers: Modifiers) -> egui::Event {
        egui::Event::Key { key: Key::V, physical_key: None, pressed, repeat: false, modifiers }
    }
    #[test]
    fn leo_pixpin_ctrl_released_before_v_keeps_paste_chord() {
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        let mut p = PasteChord::default();
        assert_eq!(p.textless_paste(&[key(true, ctrl)]), None);
        assert_eq!(p.textless_paste(&[key(false, Modifiers::NONE)]), Some(ctrl));
        assert_eq!(p.textless_paste(&[]), None);
    }
    #[test]
    fn leo_pixpin_paste_image_event_and_focus_loss_do_not_duplicate() {
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        let mut p = PasteChord::default();
        p.textless_paste(&[key(true, ctrl), egui::Event::PasteImage(std::sync::Arc::new(egui::ColorImage::filled([1,1], egui::Color32::WHITE)))]);
        assert_eq!(p.textless_paste(&[key(false, Modifiers::NONE)]), None);
        p.textless_paste(&[key(true, ctrl), egui::Event::WindowFocused(false)]);
        assert_eq!(p.textless_paste(&[key(false, Modifiers::NONE)]), None);
    }
}
''')

with (root / clip).open("a", encoding="utf-8") as f:
    f.write('''
#[cfg(test)]
mod leo_pixpin_bitmap_tests {
    use super::*;
    fn png() -> Vec<u8> {
        png_flavour(&image::RgbaImage::from_pixel(2, 1, image::Rgba([8, 16, 32, 255]))).unwrap().data
    }
    #[test]
    fn leo_pixpin_invalid_png_uses_second_png_then_fallback() {
        let valid = png();
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec(), valid.clone()], vec![], || panic!("fallback not needed")).unwrap();
        assert_eq!(result.data, valid);
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec()], vec![], || Some(Flavour { mime: PNG, data: valid.clone() })).unwrap();
        assert_eq!(result.data, valid);
    }
    #[test]
    fn leo_pixpin_invalid_png_uses_valid_dib() {
        let img = image::RgbImage::from_pixel(2, 1, image::Rgb([8, 16, 32]));
        let mut bmp = vec![];
        img.write_to(&mut Cursor::new(&mut bmp), image::ImageFormat::Bmp).unwrap();
        let dib = bmp[14..].to_vec();
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec()], vec![dib], || None).unwrap();
        let decoded = decode_png(&result.data).unwrap();
        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.get_pixel(0,0).0, [8,16,32,255]);
    }
    #[test]
    fn leo_pixpin_bad_data_reaches_decoder_instead_of_stale_clipboard() {
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec()], vec![], || None).unwrap();
        assert!(decode_png(&result.data).is_none());
        assert!(leo_bitmap_candidates(vec![], vec![], || None).is_none());
        assert_eq!(windows_formats(PNG), &["PNG", "image/png"]);
    }
}
''')
print("PASS LC-023 screenshot candidate decoding and paste-time modifier capture")

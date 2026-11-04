#!/usr/bin/env python3
"""LC-025: native CF_BITMAP fallback and resilient Windows Ctrl+V event capture."""
from pathlib import Path
import sys

if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_screenshot_paste_v3.py VECTORCRAFT_ROOT")
root = Path(sys.argv[1])

def change(rel, old, new, title):
    p = root / rel
    s = p.read_text(encoding="utf-8")
    n = s.count(old)
    if n != 1:
        raise SystemExit(f"FAIL LC-025 {title}: {rel}, source anchor count={n}")
    p.write_text(s.replace(old, new, 1), encoding="utf-8", newline="\n")
    print("OK LC-025", title)

keys = "crates/ui-egui/src/shortcuts/paste.rs"
change(keys,
       "    leo_paste_held: Option<Modifiers>,",
       "    leo_paste_held: Option<Modifiers>,\n"
       "    /// Last modifier snapshot, including frames where egui swallows the V key-down.\n"
       "    leo_last_modifiers: Modifiers,\n"
       "    /// Modifier state captured when Ctrl/Cmd is released before the V key-up.\n"
       "    leo_released_modifiers: Option<Modifiers>,",
       "remember modifier state even when the paste key-down is swallowed")
old_method = """    pub(crate) fn textless_paste(&mut self, events: &[egui::Event]) -> Option<Modifiers> {
        let mut fire = None;
        for e in events {
            match e {
                egui::Event::Paste(_) | egui::Event::PasteImage(_) => self.pasted = true,
                // A release that comes while the window is away is never seen.
                egui::Event::WindowFocused(false) => *self = Self::default(),
                egui::Event::Key { key, pressed: true, modifiers, .. } if PASTE_KEYS.contains(key) && !self.pressed.contains(key) => {
                    self.pressed.push(*key);
                    self.leo_paste_held = Some(*modifiers);
                }
                egui::Event::Key { key, pressed: false, modifiers, .. } if PASTE_KEYS.contains(key) => {
                    // Screenshot tools often put *only* a Windows CF_DIB/CF_BITMAP
                    // onto the clipboard. With some input backends egui delivers
                    // both a V press and release but no Paste or PasteImage event.
                    // The old code silently dropped that legitimate Ctrl+V.
                    if let Some(i) = self.pressed.iter().position(|k| k == key) {
                        self.pressed.swap_remove(i);
                    }
                    let handled = std::mem::take(&mut self.pasted);
                    let held = self.leo_paste_held.take().unwrap_or(*modifiers);
                    let shortcut = (*key == Key::Paste) || (*key == Key::V && held.command);
                    if shortcut && !handled {
                        fire = Some(held);
                    }
                }
                _ => {}
            }
        }
        fire
    }"""

new_method = """    pub(crate) fn textless_paste(&mut self, events: &[egui::Event]) -> Option<Modifiers> {
        let pending_at_frame_start = self.leo_released_modifiers.is_some();
        let mut modifier_released_this_frame = false;
        let mut v_released = false;
        let mut fire = None;
        for e in events {
            match e {
                egui::Event::Paste(_) | egui::Event::PasteImage(_) => self.pasted = true,
                // A release that comes while the window is away is never seen.
                egui::Event::WindowFocused(false) => *self = Self::default(),
                egui::Event::ModifiersChanged(modifiers) => {
                    let was_command = self.leo_last_modifiers.command || self.leo_last_modifiers.ctrl;
                    let is_command = modifiers.command || modifiers.ctrl;
                    if was_command && !is_command {
                        self.leo_released_modifiers = Some(self.leo_last_modifiers);
                        modifier_released_this_frame = true;
                    }
                    self.leo_last_modifiers = *modifiers;
                }
                egui::Event::Key { key, pressed: true, modifiers, .. } if PASTE_KEYS.contains(key) && !self.pressed.contains(key) => {
                    self.pressed.push(*key);
                    self.leo_paste_held = Some(*modifiers);
                    self.leo_last_modifiers = *modifiers;
                }
                egui::Event::Key { key, pressed: false, modifiers, .. } if PASTE_KEYS.contains(key) => {
                    v_released = true;
                    if let Some(i) = self.pressed.iter().position(|k| k == key) {
                        self.pressed.swap_remove(i);
                    }
                    // Some Windows backends swallow Ctrl+V's key-down when the clipboard
                    // contains an image. Preserve the modifier transition from the previous
                    // frame so releasing Ctrl before V cannot turn the paste into a no-op.
                    let keydown = self.leo_paste_held.take();
                    let released = self.leo_released_modifiers.take();
                    let previous = self.leo_last_modifiers;
                    let previous_if_released = ((previous.command || previous.ctrl)
                        && !(modifiers.command || modifiers.ctrl)).then_some(previous);
                    let held = keydown.or(released).or(previous_if_released).unwrap_or(*modifiers);
                    let handled = std::mem::take(&mut self.pasted);
                    let shortcut = (*key == Key::Paste) || (*key == Key::V && (held.command || held.ctrl));
                    if shortcut && !handled {
                        fire = Some(held);
                    }
                    self.leo_last_modifiers = *modifiers;
                }
                egui::Event::Key { modifiers, .. } => self.leo_last_modifiers = *modifiers,
                _ => {}
            }
        }
        // Keep a just-released Ctrl/Cmd for one frame only: enough for the following V key-up,
        // but not long enough to make a later, unrelated V key-up paste.
        if pending_at_frame_start && !v_released && !modifier_released_this_frame {
            self.leo_released_modifiers = None;
        }
        fire
    }"""
change(keys, old_method, new_method, "recover Ctrl+V when V key-down is swallowed or Ctrl is released first")

clip = "apps/vectorcraft/src/clipboard.rs"
old_reader = """    fn read_bitmap_candidates() -> Option<ClipboardRead> {
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
    }"""
new_reader = """    fn read_bitmap_candidates() -> Option<ClipboardRead> {
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
        // CF_BITMAP is a GDI handle, not an HGLOBAL DIB. clipboard-win's get_bitmap
        // uses GetDIBits while the clipboard is open and snapshots it as a real BMP.
        let gdi = if raw::is_format_avail(formats::CF_BITMAP) {
            let mut bmp = vec![];
            raw::get_bitmap(&mut bmp).ok().and_then(|_| (!bmp.is_empty()).then_some(bmp))
        } else {
            None
        };
        if pngs.is_empty() && dibs.is_empty() && gdi.is_none() { return None; }
        Some(ClipboardRead::Later(Box::new(move || {
            super::leo_bitmap_candidates(pngs, dibs, gdi)
        })))
    }"""
change(clip, old_reader, new_reader, "snapshot CF_BITMAP natively instead of relying on arboard to reopen it")
old_helper = """fn leo_bitmap_candidates(
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
}"""
new_helper = """fn leo_bitmap_candidates(
    pngs: Vec<Vec<u8>>, dibs: Vec<Vec<u8>>, gdi: Option<Vec<u8>>,
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
    if let Some(bmp) = gdi.as_deref() {
        let mut reader = image::ImageReader::with_format(Cursor::new(bmp), image::ImageFormat::Bmp);
        let mut limits = image::Limits::default();
        (limits.max_image_width, limits.max_image_height) = (Some(MAX_SIDE), Some(MAX_SIDE));
        reader.limits(limits);
        if let Ok(decoded) = reader.decode() {
            if let Some(f) = png_flavour(&decoded.to_rgba8()) { return Some(f); }
        }
    }
    // Do not silently paste an old internal clipboard if every external image format is bad.
    pngs.into_iter().next().map(|data| Flavour { mime: PNG, data })
        .or_else(|| dibs.into_iter().next().map(|dib| Flavour {
            mime: "image/bmp", data: dib_file(&dib).unwrap_or(dib),
        }))
        .or_else(|| gdi.map(|data| Flavour { mime: "image/bmp", data }))
}"""
change(clip, old_helper, new_helper, "decode PNG, DIB and native GDI bitmap candidates in order")

old_test = """    #[test]
    fn leo_pixpin_invalid_png_uses_second_png_then_fallback() {
        let valid = png();
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec(), valid.clone()], vec![], || panic!("fallback not needed")).unwrap();
        assert_eq!(result.data, valid);
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec()], vec![], || Some(Flavour { mime: PNG, data: valid.clone() })).unwrap();
        assert_eq!(result.data, valid);
    }"""
new_test = """    #[test]
    fn leo_pixpin_invalid_png_uses_second_png() {
        let valid = png();
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec(), valid.clone()], vec![], None).unwrap();
        assert_eq!(result.data, valid);
    }
    #[test]
    fn leo_pixpin_invalid_png_uses_native_cf_bitmap_fallback() {
        let rgb = image::RgbImage::from_pixel(2, 1, image::Rgb([8, 16, 32]));
        let mut bmp = vec![];
        rgb.write_to(&mut Cursor::new(&mut bmp), image::ImageFormat::Bmp).unwrap();
        let result = leo_bitmap_candidates(vec![b"invalid".to_vec()], vec![], Some(bmp)).unwrap();
        let decoded = decode_png(&result.data).unwrap();
        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.get_pixel(0, 0).0, [8, 16, 32, 255]);
    }"""
change(clip, old_test, new_test, "test invalid PNG recovery through native CF_BITMAP")
change(clip,
       "leo_bitmap_candidates(vec![b\"invalid\".to_vec()], vec![dib], || None)",
       "leo_bitmap_candidates(vec![b\"invalid\".to_vec()], vec![dib], None)",
       "update DIB fallback regression for native bitmap snapshot")
change(clip,
       "leo_bitmap_candidates(vec![b\"invalid\".to_vec()], vec![], || None)",
       "leo_bitmap_candidates(vec![b\"invalid\".to_vec()], vec![], None)",
       "update invalid clipboard regression for native bitmap snapshot")

with (root / keys).open("a", encoding="utf-8") as f:
    f.write("""
#[cfg(test)]
mod leo_pixpin_modifier_fallback_tests {
    use super::*;

    fn v_release(modifiers: Modifiers) -> egui::Event {
        egui::Event::Key { key: Key::V, physical_key: None, pressed: false, repeat: false, modifiers }
    }

    #[test]
    fn release_only_v_after_ctrl_release_still_triggers_paste() {
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        let mut chord = PasteChord::default();
        assert_eq!(chord.textless_paste(&[egui::Event::ModifiersChanged(ctrl)]), None);
        assert_eq!(chord.textless_paste(&[egui::Event::ModifiersChanged(Modifiers::NONE), v_release(Modifiers::NONE)]), Some(ctrl));
    }

    #[test]
    fn release_only_v_uses_last_ctrl_snapshot_if_modifier_event_was_swallowed() {
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        let mut chord = PasteChord::default();
        chord.textless_paste(&[egui::Event::ModifiersChanged(ctrl)]);
        assert_eq!(chord.textless_paste(&[v_release(Modifiers::NONE)]), Some(ctrl));
    }
}
""")

print("PASS LC-025 native CF_BITMAP and Ctrl+V fallback patch")

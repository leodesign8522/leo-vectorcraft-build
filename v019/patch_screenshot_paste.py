#!/usr/bin/env python3
"""Leo Design Screenshot Paste v1.0 for VectorCraft v0.19.0.

Fix missed Windows Ctrl+V shortcuts for image-only clipboards and add an
OS bitmap fallback for screenshot tools that publish CF_BITMAP alone.
Existing PNG / CF_DIBV5 / CF_DIB readers and editable raster import stay native.
"""
from pathlib import Path
import sys
if len(sys.argv) != 2:
    raise SystemExit("Usage: patch_screenshot_paste.py VECTORCRAFT_ROOT")
root=Path(sys.argv[1])

def change(path, old, new, description):
    p=root/path
    s=p.read_text(encoding="utf-8")
    n=s.count(old)
    if n!=1:
        raise SystemExit(f"FAIL {description}: {path}, source anchor count={n}")
    p.write_text(s.replace(old,new,1),encoding="utf-8",newline="\n")
    print("OK",description)

change(
    "crates/ui-egui/src/shortcuts/paste.rs",
    '''                egui::Event::Paste(_) => self.pasted = true,
                // A release that comes while the window is away is never seen.''',
    '''                egui::Event::Paste(_) | egui::Event::PasteImage(_) => self.pasted = true,
                // A release that comes while the window is away is never seen.''',
    "Recognize text and image paste events to prevent duplicate pastes",
)
change(
    "crates/ui-egui/src/shortcuts/paste.rs",
    '''                egui::Event::Key { key, pressed: false, modifiers, .. } if PASTE_KEYS.contains(key) => {
                    match self.pressed.iter().position(|k| k == key) {
                        Some(i) => {
                            self.pressed.swap_remove(i);
                        }
                        // Unless a Paste event came (the guard takes it either way).
                        None if !std::mem::take(&mut self.pasted) => fire = Some(*modifiers),
                        None => {}
                    }
                }''',
    '''                egui::Event::Key { key, pressed: false, modifiers, .. } if PASTE_KEYS.contains(key) => {
                    // Screenshot tools often put *only* a Windows CF_DIB/CF_BITMAP
                    // onto the clipboard. With some input backends egui delivers
                    // both a V press and release but no Paste or PasteImage event.
                    // The old code silently dropped that legitimate Ctrl+V.
                    if let Some(i) = self.pressed.iter().position(|k| k == key) {
                        self.pressed.swap_remove(i);
                    }
                    let handled = std::mem::take(&mut self.pasted);
                    let shortcut = (*key == Key::Paste) || (*key == Key::V && modifiers.command);
                    if shortcut && !handled {
                        fire = Some(*modifiers);
                    }
                }''',
    "Handle Ctrl+V released without an egui paste event, including screenshots",
)
change(
    "apps/vectorcraft/src/clipboard.rs",
    '''                    BITMAP => DIBS.into_iter().chain(formats_of(PNG)).any(raw::is_format_avail),''',
    '''                    // Some screenshot programs publish a GDI bitmap only,
                    // without either DIB variant or a registered PNG stream.
                    BITMAP => DIBS.into_iter().chain(formats_of(PNG)).any(raw::is_format_avail)
                        || raw::is_format_avail(formats::CF_BITMAP),''',
    "Detect GDI-only screenshot clipboard formats",
)
change(
    "apps/vectorcraft/src/clipboard.rs",
    '''            copied_file(files, mimes).or_else(|| {
                let _open = clipboard_win::Clipboard::new_attempts(10).ok()?;
                mimes.iter().find_map(|m| read_one(m))
            })
        }

        fn has(&mut self, mimes: &[&'static str]) -> bool {''',
    '''            let native = copied_file(files, mimes).or_else(|| {
                let _open = clipboard_win::Clipboard::new_attempts(10).ok()?;
                mimes.iter().find_map(|m| read_one(m))
            });
            if native.is_some() || !mimes.contains(&BITMAP) {
                return native;
            }
            // A handful of Windows capture tools only offer CF_BITMAP, which
            // cannot be read by raw::get_vec (it is a GDI handle, not HGLOBAL).
            // arboard uses the Windows bitmap APIs and can turn it into BGRA.
            // Crucially the previous clipboard guard has been dropped.
            let mut fallback = arboard::Clipboard::new().ok()?;
            let captured = fallback.get_image().ok()?;
            let w = u32::try_from(captured.width).ok()?;
            let h = u32::try_from(captured.height).ok()?;
            if w == 0 || h == 0 || w > crate::clipboard::MAX_SIDE || h > crate::clipboard::MAX_SIDE {
                return None;
            }
            let rgba = image::RgbaImage::from_raw(w, h, captured.bytes.into_owned())?;
            png_flavour(&rgba)
        }

        fn has(&mut self, mimes: &[&'static str]) -> bool {''',
    "Fallback to Windows GDI bitmap decoding for external screenshot tools",
)
p=root/"crates/ui-egui/src/shortcuts/paste.rs"
s=p.read_text(encoding="utf-8")
s+='''
// Leo Design screenshot-paste keyboard regression tests. A screenshot-only
// clipboard can arrive without egui PasteImage or Paste text events.
#[cfg(test)]
mod leo_screenshot_paste_tests {
    use super::*;

    fn key(key: Key, pressed: bool, modifiers: Modifiers) -> egui::Event {
        egui::Event::Key { key, physical_key: None, pressed, repeat: false, modifiers }
    }

    #[test]
    fn ctrl_v_image_clipboard_still_pastes_when_key_press_is_delivered() {
        let mut chord = PasteChord::default();
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        assert_eq!(chord.textless_paste(&[key(Key::V, true, ctrl)]), None);
        assert_eq!(chord.textless_paste(&[key(Key::V, false, ctrl)]), Some(ctrl));
        // No duplicate after release.
        assert_eq!(chord.textless_paste(&[]), None);
    }

    #[test]
    fn text_paste_event_blocks_the_image_only_fallback() {
        let mut chord = PasteChord::default();
        let ctrl = Modifiers { ctrl: true, command: true, ..Default::default() };
        chord.textless_paste(&[key(Key::V, true, ctrl), egui::Event::Paste("text".into())]);
        assert_eq!(chord.textless_paste(&[key(Key::V, false, ctrl)]), None);
    }

    #[test]
    fn plain_v_never_triggers_image_paste() {
        let mut chord = PasteChord::default();
        assert_eq!(chord.textless_paste(&[key(Key::V, true, Modifiers::NONE)]), None);
        assert_eq!(chord.textless_paste(&[key(Key::V, false, Modifiers::NONE)]), None);
    }
}
'''
p.write_text(s,encoding="utf-8",newline="\n")
print("OK keyboard shortcut regression tests")
print("SUCCESS Leo Design Screenshot Paste v1.0")

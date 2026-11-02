//! Leo Design ICO export (VectorCraft 0.18).
//! Export vector art directly at each Windows icon size using VectorCraft's
//! raster renderer, with alpha-preserving, centered square PNG directory entries.
//! A Windows .ico is an ICONDIR/ICONDIRENTRY table plus PNG payloads, not a
//! renamed PNG. PNG payloads are supported by modern Windows (Vista onward).
use serde_json::Value;
use vectorcraft_doc::Document;
use vectorcraft_render::encode::{RasterExportOptions, RasterFormat};
use vectorcraft_render::Renderer;

use super::{ArtboardPick, Encoded, *};

pub(super) const DEFAULT_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

fn selected_sizes(p: &Value) -> Result<Vec<u32>> {
    let values = match p.get("icoSizes") {
        None | Some(Value::Null) => return Ok(DEFAULT_SIZES.to_vec()),
        Some(Value::Array(a)) if !a.is_empty() && a.len() <= 7 => a,
        _ => return Err(bad("document.export", "icoSizes must be a nonempty array of 1–7 icon sizes")),
    };
    let mut sizes = Vec::new();
    for item in values {
        let size = item.as_u64().and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| bad("document.export", "ICO sizes must be integers"))?;
        if !DEFAULT_SIZES.contains(&size) {
            return Err(bad("document.export", format!("unsupported ICO size {size}, choose 16,24,32,48,64,128,256")));
        }
        if !sizes.contains(&size) {
            sizes.push(size);
        }
    }
    sizes.sort_unstable();
    Ok(sizes)
}

fn pack_png_entries(layers: &[(u32, Vec<u8>)]) -> std::result::Result<Vec<u8>, String> {
    if layers.is_empty() || layers.len() > 7 {
        return Err("ICO must contain between 1 and 7 sizes".into());
    }
    let table = 6usize + 16usize * layers.len();
    let total = layers.iter().try_fold(table, |n, (_, bytes)| n.checked_add(bytes.len()))
        .ok_or("ICO is too large")?;
    if total > u32::MAX as usize {
        return Err("ICO exceeds 4GB Windows icon limit".into());
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // icon
    out.extend_from_slice(&(layers.len() as u16).to_le_bytes());
    let mut offset = table as u32;
    for (size, png) in layers {
        if !DEFAULT_SIZES.contains(size) || !png.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("ICO entry has an invalid PNG image or unsupported size".into());
        }
        out.extend_from_slice(&[if *size == 256 { 0 } else { *size as u8 }]);
        out.extend_from_slice(&[if *size == 256 { 0 } else { *size as u8 }]);
        out.extend_from_slice(&[0, 0]); // 0 colors (true color), reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // RGBA bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in layers {
        out.extend_from_slice(png);
    }
    Ok(out)
}

fn encode_one(doc: &Document, board: usize, sizes: &[u32], antialias: vectorcraft_render::AntiAlias) -> Result<Vec<u8>> {
    let rect = doc.artboards.get(board).ok_or_else(|| bad("document.export", "invalid artboard for ICO"))?.rect;
    let longest = rect.width().max(rect.height());
    if !longest.is_finite() || longest <= 0.0 {
        return Err(bad("document.export", "ICO artboard needs positive width and height"));
    }
    let mut renderer = Renderer::new();
    let mut layers = Vec::with_capacity(sizes.len());
    for side in sizes {
        vectorcraft_doc::cancel::check()?;
        let mut opts = RasterExportOptions {
            ppi: *side as f64 * 72.0 / longest,
            background: None,
            anti_alias: antialias,
            ..Default::default()
        };
        // ICO image data is always RGBA/sRGB; do not inject CMYK profile bytes.
        opts.jpeg.embed_icc = false;
        let rendered = renderer.export_region(doc, rect, RasterFormat::Png, &opts)
            .map_err(EngineError::Other)?;
        let image = image::load_from_memory_with_format(&rendered, image::ImageFormat::Png)
            .map_err(|e| bad("document.export", format!("ICO PNG decode failed: {e}")))?.to_rgba8();
        // Contain non-square artboards on transparent canvas (never distort).
        let tw = ((rect.width() / longest * f64::from(*side)).round() as u32).clamp(1, *side);
        let th = ((rect.height() / longest * f64::from(*side)).round() as u32).clamp(1, *side);
        let centered = if image.width() != tw || image.height() != th {
            image::imageops::resize(&image, tw, th, image::imageops::FilterType::Lanczos3)
        } else {
            image
        };
        let mut square = image::RgbaImage::new(*side, *side);
        image::imageops::overlay(&mut square, &centered,
            i64::from((*side - tw) / 2), i64::from((*side - th) / 2));
        let png = vectorcraft_render::encode::png::encode(
            square.as_raw(), *side, *side,
            &vectorcraft_render::encode::png::PngOptions { ppi: None, interlaced: false })
            .map_err(EngineError::Other)?;
        layers.push((*side, png));
    }
    pack_png_entries(&layers).map_err(|e| bad("document.export", e))
}

pub(super) fn encode(doc: &Document, p: &Value, use_artboards: Option<bool>) -> Result<Encoded> {
    let sizes = selected_sizes(p)?;
    let anti = super::anti_alias(p.get("antiAlias").and_then(Value::as_str).unwrap_or("art"))
        .map_err(|e| bad("document.export", e))?;
    let n = doc.artboards.len();
    let pick: ArtboardPick = serde_json::from_value(p.clone())
        .map_err(|e| bad("document.export", format!("ICO artboards: {e}")))?;
    let boards = match use_artboards {
        Some(true) => pick.resolve(n).map_err(|e| bad("document.export", e))?
            .unwrap_or_else(|| (0..n).collect()),
        _ => vec![pick.one(n).map_err(|e| bad("document.export", e))?],
    };
    if boards.is_empty() { return Err(bad("document.export", "no artboard for ICO")); }
    let mut enc = Encoded::default();
    for (i, board) in boards.iter().enumerate() {
        vectorcraft_doc::cancel::progress(i as u64, boards.len() as u64, "Creating Windows icon");
        let bytes = encode_one(doc, *board, &sizes, anti)?;
        enc.files.push((Some(*board), bytes));
    }
    Ok(enc)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ico_container_has_all_sizes_and_valid_offsets() {
        let png = vectorcraft_render::encode::png::encode(
            &[255, 0, 0, 255], 1, 1, &Default::default()).unwrap();
        let bytes = pack_png_entries(&[(16, png.clone()), (256, png.clone())]).unwrap();
        assert_eq!(&bytes[..6], &[0,0,1,0,2,0]);
        assert_eq!(bytes[6],16);
        assert_eq!(bytes[22],0); // width 0 encodes 256
        let a = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
        let b = u32::from_le_bytes(bytes[34..38].try_into().unwrap()) as usize;
        assert_eq!(&bytes[a..a+8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&bytes[b..b+8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(b,a+png.len());
    }
    #[test]
    fn icon_sizes_validate_and_deduplicate() {
        assert_eq!(selected_sizes(&serde_json::json!({"icoSizes":[256,16,16,32]})).unwrap(), vec![16,32,256]);
        assert!(selected_sizes(&serde_json::json!({"icoSizes":[512]})).is_err());
        assert!(selected_sizes(&serde_json::json!({"icoSizes":[]})).is_err());
    }
    #[test]
    fn non_square_artboard_renders_all_icon_sizes_with_transparent_margins() {
        let mut s = crate::Session::new();
        s.execute("file.new", &serde_json::json!({"width": 100, "height": 50})).unwrap();
        s.execute("paint.setStroke", &serde_json::json!({"none": true})).unwrap();
        s.execute("shape.rectangle", &serde_json::json!({"x": 0, "y": 0, "width": 100, "height": 50})).unwrap();
        let enc = encode(&s.doc().unwrap().doc, &serde_json::json!({"icoSizes": [16, 32, 256]}), Some(true)).unwrap();
        assert_eq!(enc.files.len(), 1);
        let bytes = &enc.files[0].1;
        assert_eq!(u16::from_le_bytes(bytes[4..6].try_into().unwrap()), 3);
        for (i, side) in [16u32, 32, 256].into_iter().enumerate() {
            let entry = 6 + i * 16;
            let len = u32::from_le_bytes(bytes[entry+8..entry+12].try_into().unwrap()) as usize;
            let offset = u32::from_le_bytes(bytes[entry+12..entry+16].try_into().unwrap()) as usize;
            let image = image::load_from_memory(&bytes[offset..offset+len]).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (side, side));
            assert_eq!(image.get_pixel(side/2, 0).0[3], 0, "square padding must stay transparent");
            assert!(image.get_pixel(side/2, side/2).0[3] > 0, "vector art must be rendered");
        }
    }

}

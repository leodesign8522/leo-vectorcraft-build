//! Keep object-transform artwork and selection overlays on the same document revision.
use super::{CacheKey, Frame, VectorcraftApp, upload};
use vectorcraft_geom::Affine;

const FRAME_BUDGET_MS: f64 = 8.0;
const DRAFT_PIXELS: f64 = 262_144.0;

pub(super) fn active(app: &VectorcraftApp, key: &CacheKey) -> bool {
    key.interacting && key.pixel.is_none() && app.canvas.worker.is_some()
        && app.session.prefs.real_time_drawing && app.session.pointer_held()
        && (app.session.tool_transforming()
            || matches!(app.session.tool_id(), "scale" | "rotate" | "reflect" | "shear" | "freeTransform" | "directSelection" | "groupSelection"))
}

/// Light drawings retain full resolution. Heavy previews reduce pixels until release;
/// the image is still placed through the original canvas extent, without moving its art.
pub(super) fn dimensions(w: u32, h: u32, last_ms: f64) -> (u32, u32) {
    if !last_ms.is_finite() || last_ms <= FRAME_BUDGET_MS { return (w, h); }
    let area = f64::from(w.max(1)) * f64::from(h.max(1));
    let factor = (FRAME_BUDGET_MS / last_ms).sqrt().min((DRAFT_PIXELS / area).sqrt()).min(1.0);
    let dim = |n: u32| (f64::from(n) * factor).floor().max(1.0) as u32;
    (dim(w), dim(h))
}

pub(super) fn draw(app: &mut VectorcraftApp, ui: &egui::Ui, frame: &Frame<'_>) -> bool {
    if !active(app, frame.key) { return false; }
    let (w, h) = frame.size;
    let full_ms = app.canvas.last_ms;
    let (pw, ph) = dimensions(w, h, full_ms);
    let view = Affine::scale_non_uniform(f64::from(pw) / f64::from(w.max(1)), f64::from(ph) / f64::from(h.max(1))) * frame.view;
    let start = crate::now_ms();
    // Render the current immutable preview, rather than combining an older worker
    // bitmap with the current document's bounding box and path overlays.
    let img = frame.effects.view(|| app.canvas.renderer.render(frame.doc, pw, ph, view, &frame.opts));
    let moved = app.canvas.renderer.stats.moved > 0;
    upload(app, ui.ctx(), img, false);
    frame.effects.first_view_done();
    app.canvas.key = Some(frame.key.clone());
    app.canvas.shown = Some((frame.doc.clone(), frame.opts.clone()));
    app.canvas.from_worker = false;
    app.canvas.reused = None;
    app.canvas.moved = moved;
    app.canvas.live_draft = (pw, ph) != (w, h);
    app.perf.render_ms = crate::now_ms() - start;
    // Preserve the last full-frame cost during a reduced preview. Otherwise its
    // cheap frame would immediately switch the next drag step back to full size.
    app.canvas.last_ms = if app.canvas.live_draft { full_ms } else { app.perf.render_ms };
    true
}

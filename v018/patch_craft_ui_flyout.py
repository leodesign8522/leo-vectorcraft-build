#!/usr/bin/env python3
"""Leo Design Core Bridge: enable native draggable icon flyouts in pinned craft-ui.

This patches the exact craft-libs git checkout pinned by official VectorCraft
v0.18.0. No upstream source is rewritten in the VectorCraft repository.
An iconic flyout stays docked, visually anchored to the icon strip, but gets
left/right/bottom resize handles like a floating frame. Each panel size is
kept under a deterministic egui context id. The host captures its changes.
"""
from pathlib import Path
import os
import sys

base = Path(os.environ.get("CARGO_HOME") or (Path.home()/".cargo"))/"git"/"checkouts"
candidates = sorted(base.glob("craft-libs-*/**/crates/ui/src/docking/render/iconic.rs"))
if len(candidates) != 1:
    raise SystemExit(f"FAIL expected exactly one pinned craft-libs iconic.rs, found {len(candidates)}: {candidates}")
file = candidates[0]
text = file.read_text(encoding="utf-8")
def edit(old,new,name):
    global text
    n=text.count(old)
    if n!=1:raise SystemExit(f"FAIL {name}, match count={n}")
    text=text.replace(old,new,1)
    print("OK",name)

edit(
    '''        let bounds = self.workspace.intersect(self.float_bounds);
        let width = request.width.max(self.style.min_pane).min(bounds.width());
        let max = self''',
    '''        let bounds = self.workspace.intersect(self.float_bounds);
        // Leo Design: per-panel flyout size, shared with the host's persisted
        // workspace settings. The ID includes the active panel; changing icons
        // does not force every panel to adopt the previously selected size.
        let size_key = self.id.with(("leo-icon-size", active.clone()));
        let size: Option<egui::Vec2> = ctx.data(|d| d.get_temp(size_key));
        let width = size.map_or(request.width, |s| s.x)
            .max(self.style.min_pane + 40.0).min(bounds.width());
        let max = self''',
    "read per-panel flyout size from egui context"
)
edit(
    '''        let height = self
            .fitted_height(&active, self.style.tab_height, max)
            .unwrap_or(max)
            .max(self.style.tab_height + self.style.min_pane)
            .min(bounds.height());''',
    '''        let height = size.map(|s| s.y).unwrap_or_else(|| {
            self.fitted_height(&active, self.style.tab_height, max).unwrap_or(max)
        })
            .max(self.style.tab_height + self.style.min_pane)
            .min(bounds.height());''',
    "respect manually sized flyout height instead of always fitting content"
)
edit(
    '''        let response = FloatingFrame::new(id, rect).bounds(bounds).resizable(false).show(ctx, &style, |ui, rect| {
            self.group(ui, &group.panels, group.active, false, rect, GroupKind::Dock);
        });
        self.iconic.rect = Some(response.rect);''',
    '''        // Leo Design: native border handles work without converting the
        // flyout into an independent floating window. A panel can be resized
        // from its visible left, right and bottom edges in place.
        let response = FloatingFrame::new(id, rect).bounds(bounds)
            .resizable(true).resize_label("调整停靠面板宽高")
            .show(ctx, &style, |ui, rect| {
                self.group(ui, &group.panels, group.active, false, rect, GroupKind::Dock);
            });
        for handle in &response.handles {
            self.register_interaction(ctx, handle.id);
        }
        if let Some(resized) = response.resized {
            let next = vec2(resized.width().clamp(220.0, bounds.width()),
                resized.height().clamp(self.style.tab_height + self.style.min_pane, bounds.height()));
            ctx.data_mut(|d| d.insert_temp(size_key, next));
        }
        self.iconic.rect = Some(response.rect);''',
    "give iconic flyouts direct, draggable resize handles"
)
file.write_text(text,encoding="utf-8",newline="\n")
print("SUCCESS: Leo Design resizable pinned craft-ui icon flyouts",file)

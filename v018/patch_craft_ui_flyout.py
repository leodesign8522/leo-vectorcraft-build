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
    '''        // Leo Design: an iconic flyout is anchored to its icon rail. The
        // standard FloatingFrame edge handles are only 4 screen points wide
        // and very hard to hit; instead give the LEFT and BOTTOM edges large,
        // visible, accessible drag targets entirely inside the flyout.
        // No float conversion or new OS window is required.
        let response = FloatingFrame::new(id, rect).bounds(bounds)
            .resizable(false).show(ctx, &style, |ui, panel_rect| {
                self.group(ui, &group.panels, group.active, false, panel_rect, GroupKind::Dock);
                let min_width = (self.style.min_pane + 40.0).max(220.0);
                // 16 pt wide inner strip, with a center grip painted when
                // hovered; the flyout is attached to the icons on its right.
                let left_grip = Rect::from_min_max(
                    pos2(panel_rect.left(), panel_rect.top() + 20.0),
                    pos2((panel_rect.left() + 16.0).min(panel_rect.right()), panel_rect.bottom() - 20.0));
                let resize_width = ui.interact(left_grip, id.with("leo-resize-width"), Sense::drag())
                    .on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
                    .on_hover_text("Leo Design：拖动左边缘调整面板宽度");
                resize_width.widget_info(|| egui::WidgetInfo::labeled(
                    egui::Role::Button, ui.is_enabled(), "调整停靠面板宽度"));
                let c = if resize_width.hovered() || resize_width.dragged() {
                    style.accent
                } else {
                    style.inactive_text.gamma_multiply(0.65)
                };
                let mid = left_grip.center();
                for offset in [-5.0, 0.0, 5.0] {
                    ui.painter().vline(mid.x + offset, (mid.y - 13.0)..=(mid.y + 13.0),
                        Stroke::new(1.0, c));
                }
                // The bottom grip occupies 16 pt INSIDE the panel. Leave
                // corners to the horizontal grip and avoid the scrollbar.
                let bottom_grip = Rect::from_min_max(
                    pos2(panel_rect.left() + 20.0, (panel_rect.bottom() - 16.0).max(panel_rect.top())),
                    pos2(panel_rect.right() - 20.0, panel_rect.bottom()));
                let resize_height = ui.interact(bottom_grip, id.with("leo-resize-height"), Sense::drag())
                    .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                    .on_hover_text("Leo Design：拖动下边缘调整面板高度");
                resize_height.widget_info(|| egui::WidgetInfo::labeled(
                    egui::Role::Button, ui.is_enabled(), "调整停靠面板高度"));
                let c = if resize_height.hovered() || resize_height.dragged() {
                    style.accent
                } else {
                    style.inactive_text.gamma_multiply(0.65)
                };
                let mid = bottom_grip.center();
                for offset in [-5.0, 0.0, 5.0] {
                    ui.painter().hline((mid.x - 13.0)..=(mid.x + 13.0), mid.y + offset,
                        Stroke::new(1.0, c));
                }
                let delta = ui.input(|i| i.pointer.delta());
                if (resize_width.dragged() && delta.x != 0.0)
                    || (resize_height.dragged() && delta.y != 0.0)
                {
                    let mut next = ctx.data(|d| d.get_temp::<egui::Vec2>(size_key))
                        .unwrap_or(vec2(width, height));
                    if resize_width.dragged() {
                        next.x = (next.x - delta.x).clamp(min_width, bounds.width());
                    }
                    if resize_height.dragged() {
                        next.y = (next.y + delta.y)
                            .clamp(self.style.tab_height + self.style.min_pane, bounds.height());
                    }
                    ctx.data_mut(|d| d.insert_temp(size_key, next));
                    ctx.request_repaint();
                }
            });
        self.iconic.rect = Some(response.rect);''',
    "give iconic flyouts accessible 16pt inside-edge resize grips"
)
file.write_text(text,encoding="utf-8",newline="\n")
print("SUCCESS: Leo Design resizable pinned craft-ui icon flyouts",file)

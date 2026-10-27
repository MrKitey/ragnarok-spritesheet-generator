use crate::app::SpriteEditorApp;
use eframe::egui;

impl SpriteEditorApp {
    pub(crate) fn draw_preview(&mut self, ui: &mut egui::Ui) {
        let Some(rendered) = self.current_render() else {
            ui.centered_and_justified(|ui| {
                ui.label(sprite_editor::strings::PREVIEW_EMPTY);
            });
            return;
        };

        let (adx, ady) = self.current_anchor_offset();
        let key = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}",
            self.base.as_ref().map(|b| b.name.as_str()).unwrap_or(""),
            self.attach
                .as_ref()
                .map(|a| a.name.as_str())
                .unwrap_or(""),
            self.weapon
                .as_ref()
                .map(|w| w.name.as_str())
                .unwrap_or(""),
            self.headgear
                .as_ref()
                .map(|h| h.name.as_str())
                .unwrap_or(""),
            self.action_idx,
            self.frame_idx,
            adx,
            ady
        );

        if self.texture_key != key || self.texture.is_none() {
            let color = egui::ColorImage::from_rgba_unmultiplied(
                [rendered.width as usize, rendered.height as usize],
                &rendered.pixels,
            );
            self.texture = Some(ui.ctx().load_texture(
                "preview",
                color,
                egui::TextureOptions::NEAREST,
            ));
            self.texture_key = key;
        }

        let tex = self.texture.as_ref().unwrap();
        let size = egui::vec2(
            rendered.width as f32 * self.zoom,
            rendered.height as f32 * self.zoom,
        );

        let (rect, resp) = ui.allocate_exact_size(
            ui.available_size().max(size + egui::vec2(40.0, 40.0)),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 44, 52));

        let start = rect.center() - size * 0.5;
        let img_rect = egui::Rect::from_min_size(start, size);
        if self.show_checker {
            let cell = 8.0 * self.zoom;
            paint_checker(&painter, img_rect, cell);
        }

        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        if self.show_origin {
            let ox = img_rect.min.x + rendered.origin_x * self.zoom;
            let oy = img_rect.min.y + rendered.origin_y * self.zoom;
            let c = egui::pos2(ox, oy);
            painter.line_segment(
                [c + egui::vec2(-6.0, 0.0), c + egui::vec2(6.0, 0.0)],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
            );
            painter.line_segment(
                [c + egui::vec2(0.0, -6.0), c + egui::vec2(0.0, 6.0)],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
            );
        }

        let body_screen = rendered.body_anchor.map(|(ax, ay)| {
            egui::pos2(
                img_rect.min.x + ax * self.zoom,
                img_rect.min.y + ay * self.zoom,
            )
        });

        if self.show_anchors {
            for (ax, ay) in &rendered.anchors {
                let p = egui::pos2(
                    img_rect.min.x + *ax * self.zoom,
                    img_rect.min.y + *ay * self.zoom,
                );
                painter.circle_stroke(p, 4.0, egui::Stroke::new(1.5, egui::Color32::RED));
                painter.line_segment(
                    [p + egui::vec2(-8.0, 0.0), p + egui::vec2(8.0, 0.0)],
                    egui::Stroke::new(1.0, egui::Color32::RED),
                );
                painter.line_segment(
                    [p + egui::vec2(0.0, -8.0), p + egui::vec2(0.0, 8.0)],
                    egui::Stroke::new(1.0, egui::Color32::RED),
                );
            }
            if let Some(p) = body_screen {
                painter.circle_filled(p, 3.0, egui::Color32::YELLOW);
            }
        }

        // Drag body anchor (yellow handle when anchors visible, or always hit-test).
        let hit_r = 12.0;
        let near_handle = body_screen
            .zip(resp.hover_pos())
            .is_some_and(|(p, h)| (h - p).length() <= hit_r);

        if near_handle {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        if resp.drag_started() && near_handle {
            self.dragging_anchor = true;
            self.show_anchors = true;
        }
        if resp.drag_stopped() {
            self.dragging_anchor = false;
        }
        if self.dragging_anchor && resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            let d = resp.drag_delta() / self.zoom;
            let (ox, oy) = self.current_anchor_offset();
            self.set_anchor_offset(
                ox + d.x.round() as i32,
                oy + d.y.round() as i32,
            );
        }
    }
}

fn paint_checker(painter: &egui::Painter, rect: egui::Rect, cell: f32) {
    let c1 = egui::Color32::from_rgb(55, 58, 65);
    let c2 = egui::Color32::from_rgb(70, 74, 82);
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for y in 0..rows {
        for x in 0..cols {
            let color = if (x + y) % 2 == 0 { c1 } else { c2 };
            let min = rect.min + egui::vec2(x as f32 * cell, y as f32 * cell);
            let max = (min + egui::vec2(cell, cell)).min(rect.max);
            painter.rect_filled(egui::Rect::from_min_max(min, max), 0.0, color);
        }
    }
}

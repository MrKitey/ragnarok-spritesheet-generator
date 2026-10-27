use crate::app::SpriteEditorApp;
use eframe::egui;

impl SpriteEditorApp {
    pub(crate) fn draw_busy_overlay(&self, ctx: &egui::Context) {
        let label = self
            .busy
            .as_ref()
            .map(|j| j.label.as_str())
            .unwrap_or("");
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("busy_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ctx, |ui| {
                let (rect, _resp) =
                    ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(160));

                let spinner_size = 42.0;
                let center = rect.center();
                let spinner_rect = egui::Rect::from_center_size(
                    center - egui::vec2(0.0, 18.0),
                    egui::vec2(spinner_size, spinner_size),
                );
                egui::Spinner::new().paint_at(ui, spinner_rect);

                ui.painter().text(
                    center + egui::vec2(0.0, 28.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
            });
    }
}

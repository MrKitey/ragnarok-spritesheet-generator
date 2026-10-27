use crate::app::{ClassKind, PaletteAction, SpriteEditorApp};
use eframe::egui;

impl SpriteEditorApp {
    pub(crate) fn draw_palette_panel(&mut self, ui: &mut egui::Ui, kind: ClassKind) {
        let list = match kind {
            ClassKind::Body => &self.body_palettes,
            ClassKind::Head => &self.head_palettes,
            ClassKind::Weapon | ClassKind::Headgear => return,
        };

        let mut action: Option<PaletteAction> = None;
        let down = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if down {
            action = Some(PaletteAction::Navigate(1));
        } else if up {
            action = Some(PaletteAction::Navigate(-1));
        }

        let row_height = ui.spacing().interact_size.y;

        let default_selected = list.selected.is_none();
        if ui
            .add(egui::Button::selectable(default_selected, "Default"))
            .clicked()
        {
            action = Some(PaletteAction::Reset);
        }

        if list.entries.is_empty() {
            if self.palette_index_rx.is_some() {
                ui.label("Index palettes…");
            } else {
                ui.label("Aucune palette.");
            }
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &list.entries {
                        let selected = list.selected.as_ref() == Some(&entry.path);
                        let clicked = ui
                            .allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), row_height),
                                egui::Layout::left_to_right(egui::Align::Center)
                                    .with_main_align(egui::Align::LEFT),
                                |ui| {
                                    ui.add(
                                        egui::Button::selectable(selected, &entry.label)
                                            .truncate()
                                            .min_size(egui::vec2(
                                                ui.available_width(),
                                                row_height,
                                            )),
                                    )
                                },
                            )
                            .inner
                            .clicked();
                        if clicked {
                            action = Some(PaletteAction::Apply(entry.path.clone()));
                        }
                    }
                });
        }

        match action {
            Some(PaletteAction::Reset) => self.reset_palette(kind),
            Some(PaletteAction::Apply(path)) => self.apply_palette(kind, &path),
            Some(PaletteAction::Navigate(delta)) => self.navigate_palettes(kind, delta),
            None => {}
        }
    }
}

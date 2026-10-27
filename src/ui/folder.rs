use crate::app::{FolderAction, SpriteEditorApp};
use eframe::egui;
use sprite_editor::strings;

impl SpriteEditorApp {
    pub(crate) fn draw_folder_panel(&mut self, ui: &mut egui::Ui) {
        if self.folder.is_none() {
            ui.label(strings::FOLDER_NONE);
            return;
        }

        let mut action: Option<FolderAction> = None;
        {
            let folder = self.folder.as_ref().unwrap();
            ui.label(folder.root.display().to_string());
            ui.separator();

            if folder.rows.is_empty() {
                ui.label(strings::FOLDER_EMPTY);
                return;
            }

            let row_height = ui.spacing().interact_size.y;
            let total = folder.rows.len();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, total, |ui, range| {
                    for index in range {
                        let row = &folder.rows[index];
                        let selected = folder.selected.as_ref() == Some(&row.path);
                        let mark = if row.is_dir {
                            if folder.expanded.contains(&row.path) {
                                "- "
                            } else {
                                "+ "
                            }
                        } else {
                            "  "
                        };
                        let indent = "  ".repeat(row.depth);
                        let label = format!("{indent}{mark}{}", row.name);
                        let width = ui.available_width();
                        let clicked = ui
                            .allocate_ui_with_layout(
                                egui::vec2(width, row_height),
                                egui::Layout::left_to_right(egui::Align::Center)
                                    .with_main_align(egui::Align::LEFT),
                                |ui| {
                                    ui.add(
                                        egui::Button::selectable(selected, label)
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
                            action = Some(if row.is_dir {
                                FolderAction::Toggle(row.path.clone())
                            } else {
                                FolderAction::Open(row.path.clone())
                            });
                        }
                    }
                });
        }

        match action {
            Some(FolderAction::Toggle(path)) => self.toggle_dir(&path),
            Some(FolderAction::Open(path)) => self.load_from_folder(&path),
            None => {}
        }
    }
}

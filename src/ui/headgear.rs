use crate::app::{ClassAction, ClassKind, SpriteEditorApp};
use eframe::egui;
use sprite_editor::scan::{build_headgear_rows, HeadgearRow};

impl SpriteEditorApp {
    pub(crate) fn draw_headgear_panel(&mut self, ui: &mut egui::Ui) {
        if self.headgear_rx.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Index headgears…");
            });
            return;
        }
        let Some(browser) = self.headgear_classes.as_ref() else {
            ui.label("Aucun headgear.");
            return;
        };
        if browser.entries.is_empty() {
            ui.label("Aucun headgear.");
            return;
        }

        let rows = build_headgear_rows(browser);
        let mut action: Option<ClassAction> = None;
        let down = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if down {
            action = Some(ClassAction::Navigate(1));
        } else if up {
            action = Some(ClassAction::Navigate(-1));
        }

        let row_height = ui.spacing().interact_size.y;
        let total = rows.len();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total, |ui, range| {
                for index in range {
                    let row = &rows[index];
                    match row {
                        HeadgearRow::Class {
                            stem,
                            label,
                            expanded,
                        } => {
                            let mark = if *expanded { "- " } else { "+ " };
                            let clicked = ui
                                .add(
                                    egui::Button::selectable(
                                        false,
                                        format!("{mark}{label}"),
                                    )
                                    .truncate()
                                    .min_size(egui::vec2(ui.available_width(), row_height)),
                                )
                                .clicked();
                            if clicked {
                                action = Some(ClassAction::ToggleClass(stem.clone()));
                            }
                        }
                        HeadgearRow::Gender {
                            stem,
                            is_male,
                            multi,
                            expanded,
                            selected,
                            path,
                        } => {
                            let gender_label = if *is_male { "Male" } else { "Female" };
                            let g_mark = if *multi {
                                if *expanded { "- " } else { "+ " }
                            } else {
                                "  "
                            };
                            let clicked = ui
                                .add(
                                    egui::Button::selectable(
                                        *selected,
                                        format!("  {g_mark}{gender_label}"),
                                    )
                                    .truncate()
                                    .min_size(egui::vec2(ui.available_width(), row_height)),
                                )
                                .clicked();
                            if clicked {
                                if *multi {
                                    action = Some(ClassAction::ToggleGender((
                                        stem.clone(),
                                        *is_male,
                                    )));
                                } else if let Some(p) = path {
                                    action = Some(ClassAction::Open(p.clone()));
                                }
                            }
                        }
                        HeadgearRow::Variant {
                            path,
                            label,
                            selected,
                        } => {
                            let clicked = ui
                                .add(
                                    egui::Button::selectable(
                                        *selected,
                                        format!("    {label}"),
                                    )
                                    .truncate()
                                    .min_size(egui::vec2(ui.available_width(), row_height)),
                                )
                                .clicked();
                            if clicked {
                                action = Some(ClassAction::Open(path.clone()));
                            }
                        }
                    }
                }
            });

        match action {
            Some(ClassAction::ToggleClass(stem)) => {
                if let Some(b) = self.headgear_classes.as_mut() {
                    if b.expanded.contains(&stem) {
                        b.expanded.remove(&stem);
                    } else {
                        b.expanded.insert(stem);
                    }
                }
            }
            Some(ClassAction::ToggleGender(key)) => {
                if let Some(b) = self.headgear_classes.as_mut() {
                    if b.expanded_gender.contains(&key) {
                        b.expanded_gender.remove(&key);
                    } else {
                        b.expanded_gender.insert(key);
                    }
                }
            }
            Some(ClassAction::Open(path)) => self.load_headgear_sprite(&path),
            Some(ClassAction::Navigate(delta)) => {
                self.navigate_class_sprites(ClassKind::Headgear, delta)
            }
            None => {}
        }
    }
}

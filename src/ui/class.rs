use crate::app::{ClassAction, ClassKind, SpriteEditorApp};
use eframe::egui;
use sprite_editor::names::Gender;
use sprite_editor::strings;

impl SpriteEditorApp {
    pub(crate) fn draw_class_panel(&mut self, ui: &mut egui::Ui, kind: ClassKind) {
        let Some(browser) = self.class_browser(kind) else {
            return;
        };

        if kind == ClassKind::Weapon && self.body_sel.is_none() {
            ui.label(strings::CLASS_SELECT_BODY);
            return;
        }

        if browser.entries.is_empty() {
            ui.label(strings::CLASS_NO_SPRITE);
            return;
        }

        let weapon_filter = (kind == ClassKind::Weapon).then(|| {
            self.body_sel
                .as_ref()
                .map(|s| (s.stem.clone(), s.gender))
                .expect("body_sel checked")
        });

        if let Some((stem, gender)) = &weapon_filter {
            let has = browser.entries.iter().any(|e| {
                if &e.stem != stem {
                    return false;
                }
                match gender {
                    Gender::Male => !e.male.is_empty(),
                    Gender::Female => !e.female.is_empty(),
                }
            });
            if !has {
                ui.label(strings::CLASS_NO_WEAPON);
                return;
            }
        }

        let mut action: Option<ClassAction> = None;
        let down = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if down {
            action = Some(ClassAction::Navigate(1));
        } else if up {
            action = Some(ClassAction::Navigate(-1));
        }

        let row_height = ui.spacing().interact_size.y;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &browser.entries {
                    if let Some((stem, _)) = &weapon_filter {
                        if &entry.stem != stem {
                            continue;
                        }
                    }

                    let force_weapon = weapon_filter.is_some();
                    let expanded = browser.expanded.contains(&entry.stem) || force_weapon;
                    let mark = if expanded { "- " } else { "+ " };
                    let width = ui.available_width();
                    let clicked = ui
                        .allocate_ui_with_layout(
                            egui::vec2(width, row_height),
                            egui::Layout::left_to_right(egui::Align::Center)
                                .with_main_align(egui::Align::LEFT),
                            |ui| {
                                ui.add(
                                    egui::Button::selectable(false, format!("{mark}{}", entry.label))
                                        .truncate()
                                        .min_size(egui::vec2(ui.available_width(), row_height)),
                                )
                            },
                        )
                        .inner
                        .clicked();
                    if clicked && !force_weapon {
                        action = Some(ClassAction::ToggleClass(entry.stem.clone()));
                    }

                    if !expanded {
                        continue;
                    }

                    for (is_male, variants) in [(true, &entry.male), (false, &entry.female)] {
                        if variants.is_empty() {
                            continue;
                        }
                        if let Some((_, gender)) = &weapon_filter {
                            let want_male = *gender == Gender::Male;
                            if is_male != want_male {
                                continue;
                            }
                            // Body filter: list weapon ids directly.
                            for v in variants {
                                let selected = browser.selected.as_ref() == Some(&v.path);
                                let label = format!("  {}", v.label);
                                let clicked = ui
                                    .allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), row_height),
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
                                    action = Some(ClassAction::Open(v.path.clone()));
                                }
                            }
                            continue;
                        }
                        let gender_label = if is_male {
                            strings::LABEL_MALE
                        } else {
                            strings::LABEL_FEMALE
                        };
                        let gkey = (entry.stem.clone(), is_male);
                        let multi = variants.len() > 1;
                        let g_expanded = browser.expanded_gender.contains(&gkey);
                        let g_mark = if multi {
                            if g_expanded { "- " } else { "+ " }
                        } else {
                            "  "
                        };
                        let selected = !multi
                            && variants
                                .first()
                                .is_some_and(|v| browser.selected.as_ref() == Some(&v.path));
                        let label = format!("  {g_mark}{gender_label}");
                        let clicked = ui
                            .allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), row_height),
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
                            if multi {
                                action = Some(ClassAction::ToggleGender(gkey));
                            } else if let Some(v) = variants.first() {
                                action = Some(ClassAction::Open(v.path.clone()));
                            }
                        }

                        if multi && g_expanded {
                            for v in variants {
                                let selected = browser.selected.as_ref() == Some(&v.path);
                                let label = format!("    {}", v.label);
                                let clicked = ui
                                    .allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), row_height),
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
                                    action = Some(ClassAction::Open(v.path.clone()));
                                }
                            }
                        }
                    }
                }
            });

        match action {
            Some(ClassAction::ToggleClass(stem)) => {
                if let Some(b) = self.class_browser_mut(kind) {
                    if b.expanded.contains(&stem) {
                        b.expanded.remove(&stem);
                    } else {
                        b.expanded.insert(stem);
                    }
                }
            }
            Some(ClassAction::ToggleGender(key)) => {
                if let Some(b) = self.class_browser_mut(kind) {
                    if b.expanded_gender.contains(&key) {
                        b.expanded_gender.remove(&key);
                    } else {
                        b.expanded_gender.insert(key);
                    }
                }
            }
            // Clicking the current selection again deselects it.
            Some(ClassAction::Open(path)) if self.is_class_selected(kind, &path) => {
                self.unload_class_sprite(kind)
            }
            Some(ClassAction::Open(path)) => self.open_class_sprite(kind, &path),
            Some(ClassAction::Navigate(delta)) => self.navigate_class_sprites(kind, delta),
            None => {}
        }
    }
}

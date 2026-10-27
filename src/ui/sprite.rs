use crate::app::SpriteEditorApp;
use eframe::egui;
use sprite_editor::render::action_frame;
use sprite_editor::strings;
use std::time::Instant;

impl SpriteEditorApp {
    pub(crate) fn draw_sprite_side(&mut self, ui: &mut egui::Ui) {
        let mut export = false;
        let mut reset_anchor = false;
        let mut new_offset: Option<(i32, i32)> = None;
        if let Some(base) = &self.base {
            ui.heading(&base.name);
            ui.label(strings::spr_version(base.spr.version));
            ui.label(strings::act_version(base.act.version));
            ui.label(strings::images_counts(
                base.spr.palette_count,
                base.spr.rgba_count,
            ));
            ui.label(strings::actions_count(base.act.actions.len()));
            ui.separator();

            let n_actions = base.act.actions.len();
            if n_actions > 0 {
                ui.add(
                    egui::Slider::new(&mut self.action_idx, 0..=n_actions.saturating_sub(1))
                        .text(strings::LABEL_ACTION),
                );
                let n_frames = base
                    .act
                    .actions
                    .get(self.action_idx)
                    .map(|a| a.frames.len())
                    .unwrap_or(0);
                if n_frames > 0 {
                    if self.frame_idx >= n_frames {
                        self.frame_idx = 0;
                    }
                    ui.add(
                        egui::Slider::new(&mut self.frame_idx, 0..=n_frames.saturating_sub(1))
                            .text(strings::LABEL_FRAME),
                    );
                    let ms = base.act.actions[self.action_idx].frame_ms;
                    ui.label(strings::interval_ms(ms));
                }
            }

            ui.horizontal(|ui| {
                if ui
                    .button(if self.playing {
                        strings::BTN_PAUSE
                    } else {
                        strings::BTN_PLAY
                    })
                    .clicked()
                {
                    self.playing = !self.playing;
                    self.last_tick = Instant::now();
                }
                if ui.button("<<").clicked() {
                    self.frame_idx = 0;
                    self.texture_key.clear();
                }
            });

            if ui.button(strings::BTN_EXPORT_SHEET).clicked() {
                export = true;
            }

            if let Some(frame) = action_frame(&base.act, self.action_idx, self.frame_idx) {
                ui.separator();
                ui.label(strings::layers_count(frame.layers.len()));
                ui.label(strings::anchors_count(frame.anchors.len()));
                for (i, a) in frame.anchors.iter().enumerate() {
                    ui.monospace(format!("#{i}: ({}, {}) attr={}", a.x, a.y, a.attr));
                }
                if frame.event_id >= 0 {
                    let name = base
                        .act
                        .events
                        .get(frame.event_id as usize)
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    ui.label(strings::event_label(frame.event_id, name));
                }
            }

            ui.separator();
            ui.label(strings::anchor_offset_label(self.action_idx));
            let (mut dx, mut dy) = self
                .anchor_offsets
                .get(&self.action_idx)
                .copied()
                .unwrap_or((0, 0));
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("X");
                changed |= ui.add(egui::DragValue::new(&mut dx).speed(1.0)).changed();
                ui.label("Y");
                changed |= ui.add(egui::DragValue::new(&mut dy).speed(1.0)).changed();
                if ui.button(strings::BTN_RESET).clicked() {
                    reset_anchor = true;
                }
            });
            if changed {
                new_offset = Some((dx, dy));
            }

            if let Some(attach) = &self.attach {
                ui.separator();
                ui.label(strings::attached_head(&attach.name));
            }
            if let Some(weapon) = &self.weapon {
                ui.label(strings::attached_weapon(&weapon.name));
            }
            if let Some(headgear) = &self.headgear {
                ui.label(strings::attached_headgear(&headgear.name));
            }
        } else {
            ui.label(strings::SPRITE_NONE);
        }

        if reset_anchor {
            let action = self.action_idx;
            self.reset_anchor_offset();
            self.status = strings::status_anchor_origin(action);
        } else if let Some((dx, dy)) = new_offset {
            let action = self.action_idx;
            self.set_anchor_offset(dx, dy);
            self.show_anchors = true;
            self.status = strings::status_anchor_offset(action, dx, dy);
        }

        if export {
            self.export_spritesheet();
        }
    }
}

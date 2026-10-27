use crate::app::SpriteEditorApp;
use eframe::egui;
use sprite_editor::render::action_frame;
use std::time::Instant;

impl SpriteEditorApp {
    pub(crate) fn draw_sprite_side(&mut self, ui: &mut egui::Ui) {
        let mut export = false;
        let mut reset_anchor = false;
        let mut new_offset: Option<(i32, i32)> = None;
        if let Some(base) = &self.base {
            ui.heading(&base.name);
            ui.label(format!("SPR v{:.1}", base.spr.version));
            ui.label(format!("ACT v{:.1}", base.act.version));
            ui.label(format!(
                "Images: {} pal + {} rgba",
                base.spr.palette_count, base.spr.rgba_count
            ));
            ui.label(format!("Actions: {}", base.act.actions.len()));
            ui.separator();

            let n_actions = base.act.actions.len();
            if n_actions > 0 {
                ui.add(
                    egui::Slider::new(&mut self.action_idx, 0..=n_actions.saturating_sub(1))
                        .text("Action"),
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
                            .text("Frame"),
                    );
                    let ms = base.act.actions[self.action_idx].frame_ms;
                    ui.label(format!("Interval: {ms:.0} ms"));
                }
            }

            ui.horizontal(|ui| {
                if ui
                    .button(if self.playing { "Pause" } else { "Play" })
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

            if ui.button("Exporter spritesheet…").clicked() {
                export = true;
            }

            if let Some(frame) = action_frame(&base.act, self.action_idx, self.frame_idx) {
                ui.separator();
                ui.label(format!("Layers: {}", frame.layers.len()));
                ui.label(format!("Ancres: {}", frame.anchors.len()));
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
                    ui.label(format!("Event: {} ({name})", frame.event_id));
                }
            }

            ui.separator();
            ui.label(format!("Offset ancre (action {})", self.action_idx));
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
                if ui.button("Reset").clicked() {
                    reset_anchor = true;
                }
            });
            if changed {
                new_offset = Some((dx, dy));
            }

            if let Some(attach) = &self.attach {
                ui.separator();
                ui.label(format!("Head: {}", attach.name));
            }
            if let Some(weapon) = &self.weapon {
                ui.label(format!("Weapon: {}", weapon.name));
            }
            if let Some(headgear) = &self.headgear {
                ui.label(format!("Headgear: {}", headgear.name));
            }
        } else {
            ui.label("Aucun sprite charge.");
        }

        if reset_anchor {
            let action = self.action_idx;
            self.reset_anchor_offset();
            self.status = format!("Ancre action {action}: origine");
        } else if let Some((dx, dy)) = new_offset {
            let action = self.action_idx;
            self.set_anchor_offset(dx, dy);
            self.show_anchors = true;
            self.status = format!("Ancre action {action}: ({dx}, {dy})");
        }

        if export {
            self.export_spritesheet();
        }
    }
}

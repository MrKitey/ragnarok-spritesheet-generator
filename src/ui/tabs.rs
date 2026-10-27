use crate::app::{ClassKind, SideTab, SpriteEditorApp};
use eframe::egui;
use sprite_editor::strings;

impl SpriteEditorApp {
    pub(crate) fn draw_side_tabs(&mut self, ui: &mut egui::Ui) {
        self.ensure_side_tab();

        let tabs: &[(SideTab, &str)] = &[
            (SideTab::Folder, strings::TAB_FOLDER),
            (SideTab::Body, strings::TAB_BODY),
            (SideTab::Head, strings::TAB_HEAD),
            (SideTab::Weapons, strings::TAB_WEAPONS),
            (SideTab::Headgears, strings::TAB_HEADGEARS),
            (SideTab::BodyPal, strings::TAB_BODY_PAL),
            (SideTab::HeadPal, strings::TAB_HEAD_PAL),
            (SideTab::Sprite, strings::TAB_SPRITE),
        ];

        ui.horizontal_wrapped(|ui| {
            for &(tab, label) in tabs {
                if !self.tab_available(tab) {
                    continue;
                }
                ui.selectable_value(&mut self.side_tab, tab, label);
            }
        });
        ui.separator();

        match self.side_tab {
            SideTab::Folder => self.draw_folder_panel(ui),
            SideTab::Body => self.draw_class_panel(ui, ClassKind::Body),
            SideTab::Head => self.draw_class_panel(ui, ClassKind::Head),
            SideTab::Weapons => self.draw_class_panel(ui, ClassKind::Weapon),
            SideTab::Headgears => self.draw_headgear_panel(ui),
            SideTab::BodyPal => self.draw_palette_panel(ui, ClassKind::Body),
            SideTab::HeadPal => self.draw_palette_panel(ui, ClassKind::Head),
            SideTab::Sprite => self.draw_sprite_side(ui),
        }
    }
}

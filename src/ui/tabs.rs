use crate::app::{ClassKind, SideTab, SpriteEditorApp};
use eframe::egui;

impl SpriteEditorApp {
    pub(crate) fn draw_side_tabs(&mut self, ui: &mut egui::Ui) {
        self.ensure_side_tab();

        let tabs: &[(SideTab, &str)] = &[
            (SideTab::Folder, "Dossier"),
            (SideTab::Body, "Body"),
            (SideTab::Head, "Head"),
            (SideTab::Weapons, "Weapons"),
            (SideTab::Headgears, "Headgears"),
            (SideTab::BodyPal, "Body Pal"),
            (SideTab::HeadPal, "Head Pal"),
            (SideTab::Sprite, "Sprite"),
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

mod sidebar;
use egui;

use crate::my_structs::*;


impl MyApp {
    pub(super) fn show_left(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("标签");
        });
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_side_bar(ui);

            // 版本号水印
            ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                ui.label(egui::RichText::new(crate::PROGRAM_VERSION).weak());
            });
        });
    }
}

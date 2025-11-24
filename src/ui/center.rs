mod links;

use egui;

use crate::my_structs::*;

impl MyApp {
    pub(super) fn show_center(&mut self, ui: &mut egui::Ui) {

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.edit_mode, "编辑模式");
            });
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_popup(ui);

            self.show_links(ui);
        });
    }
}

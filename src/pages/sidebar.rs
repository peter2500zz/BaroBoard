use egui;

use crate::my_structs::*;


impl MyApp {
    pub fn side_bar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            for tag in &self.tags {
                let is_selected = self.current_tag.as_ref().unwrap_or(&"".to_string()) == tag;

                if ui.selectable_label(
                    is_selected,
                    tag.clone()
                ).clicked() {
                    if is_selected {
                        self.current_tag = None;
                    } else {
                        self.current_tag = Some(tag.clone());
                    }
                }
            }
        });
    }
}

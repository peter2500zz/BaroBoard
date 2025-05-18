use eframe::egui;

use crate::my_structs::*;


impl MyApp {
    pub fn side_bar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            for (i, _page) in self.pages.iter().enumerate() {
                if ui.button( &_page.title).clicked() {
                    self.current_page_index = i;
                    println!("现在是第 {} 页", i);
                }
            }
        });
    }
}
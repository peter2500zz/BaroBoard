use egui;

use crate::my_structs::*;


impl MyApp {
    pub fn side_bar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            for i in 0..3 {
                if ui.button( i.to_string()).clicked() {
                    println!("现在是第 {} 页", i);
                }
            }
        });
    }
}

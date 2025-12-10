mod center;
mod left;
mod top;
pub mod popups;

use egui;

use crate::my_structs::*;



impl MyApp {
    pub fn show_ui(&mut self, ui: &mut egui::Ui)  {        
        // 添加面板的顺序非常重要，影响最终的布局
        egui::TopBottomPanel::top("title")
            .resizable(false)
            .min_height(32.0)
            .show_inside(ui, |ui| self.show_top(ui));

        egui::SidePanel::left("side_bar")
            .resizable(false)
            .default_width(150.0)
            // .width_range(80.0..=200.0)
            .show_inside(ui, |ui| self.show_left(ui));

        egui::CentralPanel::default()
            .show_inside(ui, |ui| self.show_center(ui));
    }

}

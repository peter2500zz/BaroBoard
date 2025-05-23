use egui;
use std::collections::HashSet;

use crate::my_structs::*;


pub struct LinkDelete {
    pub delete_called: bool,
    index_of_the_link: usize,
}

impl LinkDelete {
    pub fn new() -> Self {
        Self {
            delete_called: false,
            index_of_the_link: 0
        }
    }



}


impl MyApp {
        // pub 
    
    // pub fn show_delete_linkx(&mut self, ui: &mut egui::Ui) {
    //     // 删除快捷方式弹窗
    //     egui::Window::new("你确定要删除这个快捷方式吗？")
    //     .title_bar(false)
    //     .collapsible(false)
    //     .resizable(false)
    //     .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    //     .fade_in(true)
    //     .fade_out(true)
    //     .open(&mut self.link_popups.link_delete.delete_called.clone())

    //     .show(ui.ctx(), |ui| {
    //         ui.vertical_centered(|ui| {
    //             ui.heading("你确定要删除这个快捷方式吗？");
    //             ui.label(format!(
    //                 "“{}”将会永久消失！（真的很久！）", 
    //                 self
    //                 // 这里不能unwarp的原因是
    //                 // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
    //                 .program_links.get(self.link_popups.link_delete.index_of_the_link).cloned().unwrap_or_default()//(ProgramLink { name: "已删除".to_string(), ..Default::default()})
    //                 .name
    //             ));
                
    //             ui.separator();
                
    //             ui.with_layout(egui::Layout {
    //                 cross_align: egui::Align::RIGHT,
    //                 ..Default::default()
    //             }, |ui| {
    //                 ui.horizontal(|ui| {
    //                     if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
    //                     .clicked() && self.link_popups.link_delete.delete_called {

    //                         self.link_popups.link_delete.delete_called = false;
    //                     }
    //                     if ui.button("取消").clicked() {
    //                         self.link_popups.link_delete.delete_called = false;
    //                     }
    //                 });
    //             });
    //         });
    //     });
    // }
}

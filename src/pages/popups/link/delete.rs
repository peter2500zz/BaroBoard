use eframe::egui;

use crate::my_structs::*;


pub struct LinkDelete {
    pub delete_called: bool,
    pub page_to_delete: usize,
    index_of_the_link: usize,
}

impl LinkDelete {
    pub fn new() -> Self {
        Self {
            delete_called: false, 
            page_to_delete: 0, 
            index_of_the_link: 0
        }
    }


    pub fn delete_link(&mut self, page_index: usize, link_index: usize) {
        self.page_to_delete = page_index;
        self.index_of_the_link = link_index;
        self.delete_called = true;
    }
}


impl MyApp {
    pub fn show_delete_link(&mut self, ui: &mut egui::Ui) {
        // 删除快捷方式弹窗
        egui::Window::new("你确定要删除这个快捷方式吗？")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut self.link_popups.link_delete.delete_called.clone())

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个快捷方式吗？");
                ui.label(format!(
                    "“{}”将会永久消失！（真的很久！）", 
                    self
                    .pages.get(self.link_popups.link_delete.page_to_delete).cloned().unwrap_or_default()
                    .program_links.get(self.link_popups.link_delete.index_of_the_link).cloned().unwrap_or_default()
                    .name
                ));
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() && self.link_popups.link_delete.delete_called {
                            let program_links = &mut self.pages[self.link_popups.link_delete.page_to_delete].program_links;

                            self.icon_will_clean.push(program_links[self.link_popups.link_delete.index_of_the_link].icon_path.clone());
                            program_links.remove(self.link_popups.link_delete.index_of_the_link);
                            println!("删除成功: {:?}", program_links);
                            match self.link_popups.link_save.save_conf(self.pages.clone()) {
                                Ok(_) => println!("保存成功"),
                                Err(e) => {
                                    println!("保存失败: {}", e);
                                    self.link_popups.link_save.error_called = true;
                                },
                            };
                            
                            self.link_popups.link_delete.delete_called = false;
                        }
                        if ui.button("取消").clicked() {
                            self.link_popups.link_delete.delete_called = false;
                        }
                    });
                });
            });
        });
    }
}

use egui;
use std::collections::HashSet;

use crate::my_structs::*;


pub struct Info {
    pub called: bool,
    pub title: String,
    pub content: String,
}

impl Info {
    pub fn new() -> Self {
        Self {
            called: false,
            title: "".to_string(),
            content: "".to_string(),
        }
    }
}


impl MyApp {
    pub fn show_info(&mut self, ui: &mut egui::Ui) {
        
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
                    // 这里不能unwarp的原因是
                    // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
                    .program_links.get(self.link_popups.link_delete.index_of_the_link).cloned().unwrap_or_default()//(ProgramLink { name: "已删除".to_string(), ..Default::default()})
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
                            let program_links = &mut self.program_links;

                            if let Some(icon_path) = self.cached_icon.get_mut(&program_links[self.link_popups.link_delete.index_of_the_link].icon_path) {
                                icon_path.remove(&program_links[self.link_popups.link_delete.index_of_the_link].uuid);
                            } else {
                                // 如果不行则强制清空
                                self.cached_icon.insert(program_links[self.link_popups.link_delete.index_of_the_link].icon_path.clone(), HashSet::new());
                            }
                            self.icon_will_clean.push(program_links[self.link_popups.link_delete.index_of_the_link].icon_path.clone());

                            program_links.remove(self.link_popups.link_delete.index_of_the_link);
                            println!("删除成功: {:?}", program_links);
                            match self.link_popups.link_save.save_conf(self.program_links.clone()) {
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
    fn delete_link(&mut self, link_index: usize) {
        let program_links = &mut self.program_links;

        if let Some(icon_path) = self.cached_icon.get_mut(&program_links[link_index].icon_path) {
            icon_path.remove(&program_links[link_index].uuid);
        } else {
            // 如果不行则强制清空
            self.cached_icon.insert(program_links[link_index].icon_path.clone(), HashSet::new());
        }
        self.icon_will_clean.push(program_links[link_index].icon_path.clone());

        program_links.remove(link_index);
        println!("删除成功: {:?}", program_links);
        match self.link_popups.link_save.save_conf(self.program_links.clone()) {
            Ok(_) => println!("保存成功"),
            Err(e) => {
                println!("保存失败: {}", e);
                self.link_popups.link_save.error_called = true;
            },
        };
    }
}

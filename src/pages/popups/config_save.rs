use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};
use std::collections::HashSet;

use crate::my_structs::*;


#[derive(Serialize, Deserialize)]
pub struct LinkConfigSchema {
    pub version: String,
    pub pages: Vec<Page>
}

pub struct ConfigSave {
    pub error_called: bool,
    pub delete_called: bool,

    page_to_save: usize,
    index_of_the_link: usize,
}

impl ConfigSave {
    pub fn new() -> Self {
        Self {
            // title: "你确定要删除这个快捷方式吗？".to_string(),
            // message: "“{}”将会永久消失！（真的很久！）".to_string(),
            error_called: false,
            delete_called: false,
            page_to_save: 0,
            index_of_the_link: 0,
        }
    }

    pub fn delete_link(&mut self, page_index: usize, link_index: usize) {
        self.page_to_save = page_index;
        self.index_of_the_link = link_index;
        self.delete_called = true;
    }

    pub fn load_conf(path: &str) -> Result<LinkConfigSchema, std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;


        let links_config: LinkConfigSchema = serde_json::from_str(&buffer)?;

        Ok(links_config)
    }

    pub fn save_conf(&self, pages: Vec<Page>) -> Result<(), std::io::Error> {
        self.save_conf_to_path(pages, ".links.json")
    }

    pub fn save_conf_to_path(&self, pages: Vec<Page>, path: &str) -> Result<(), std::io::Error> {
        let links_config = LinkConfigSchema {
            version: "1.0".to_string(),
            pages: pages,
        };

        let serialized = serde_json::to_string_pretty(&links_config)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
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
        .open(&mut self.config_save.delete_called.clone())

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个快捷方式吗？");
                ui.label(format!("“{}”将会永久消失！（真的很久！）", self.pages[self.config_save.page_to_save].program_links[self.config_save.index_of_the_link].name));
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            let program_links = &mut self.pages[self.config_save.page_to_save].program_links;
                            self.cached_icon
                                .entry(program_links[self.config_save.index_of_the_link].icon_path.clone())
                                .or_insert_with(HashSet::new)
                                .remove(&program_links[self.config_save.index_of_the_link].uuid);

                            self.icon_will_clean.push(program_links[self.config_save.index_of_the_link].icon_path.clone());
                            program_links.remove(self.config_save.index_of_the_link);
                            println!("删除成功: {:?}", program_links);
                            match self.config_save.save_conf(self.pages.clone()) {
                                Ok(_) => println!("保存成功"),
                                Err(e) => {
                                    println!("保存失败: {}", e);
                                    self.config_save.error_called = true;
                                },
                            };
                            self.config_save.delete_called = false;
                        }
                        if ui.button("取消").clicked() {
                            self.config_save.delete_called = false;
                        }
                    });
                });
            });
        });
    }

    pub fn show_config_save_error(&mut self, ui: &mut egui::Ui) {
        // 配置文件弹窗
        egui::Window::new("无法写入配置文件！")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut self.called)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label("你可以尝试删除配置文件并再次保存");
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("好的").clicked() {
                            self.config_save.error_called = false;
                        };
                        if ui.button("重试").clicked() {
                            match self.config_save.save_conf(self.pages.clone()) {
                                Ok(_) => {
                                    println!("保存成功");
                                    self.config_save.error_called = false;
                                },
                                Err(e) => {
                                    println!("保存失败: {}", e);
                                    self.config_save.error_called = true;
                                },
                            };
                        }
                    });
                });
            });
        });
    }
}
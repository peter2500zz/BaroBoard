use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};

use crate::my_structs::*;


#[derive(Serialize, Deserialize)]
pub struct LinkConfigSchema {
    pub version: String,
    pub pages: Vec<Page>
}


pub struct LinkSave {
    pub error_called: bool,
}

impl LinkSave {
    pub fn new() -> Self {
        Self {
            // title: "你确定要删除这个快捷方式吗？".to_string(),
            // message: "“{}”将会永久消失！（真的很久！）".to_string(),
            error_called: false,
        }
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
                            self.link_popups.link_save.error_called = false;
                        };
                        if ui.button("重试").clicked() {
                            match self.link_popups.link_save.save_conf(self.pages.clone()) {
                                Ok(_) => {
                                    println!("保存成功");
                                    self.link_popups.link_save.error_called = false;
                                },
                                Err(e) => {
                                    println!("保存失败: {}", e);
                                    self.link_popups.link_save.error_called = true;
                                },
                            };
                        }
                    });
                });
            });
        });
    }
}

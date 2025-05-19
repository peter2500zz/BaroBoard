use eframe::egui;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::pages::popups::link::{LinkPopups, save::LinkSave};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramLink {
    pub name: String,
    pub icon_path: String,
    pub run_command: String,
    pub uuid: String,
}

impl Default for ProgramLink {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            icon_path: "".to_string(),
            run_command: "".to_string(),
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

impl ProgramLink {
    pub fn new(name: String, icon_path: String, run_command: String) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
            ..Default::default()
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    pub program_links: Vec<ProgramLink>,
    pub title: String,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            program_links: Vec::new(),
            title: "新页面".to_string(),
        }
    }
}

impl Page {
    pub fn new(title: String, programms: Vec<ProgramLink>) -> Self {
        Self {
            program_links: programms,
            title: title,
        }
    }
}


pub struct LinkPosition {
    pub page_index: usize,
    pub link_index: usize,
}

impl LinkPosition {
    pub fn new(page_index: usize, link_index: usize) -> Self {
        Self {
            page_index: page_index,
            link_index: link_index,
        }
    }
}


pub struct MyApp {
    pub pages: Vec<Page>,
    pub current_page_index: usize,
    pub title: String,
    pub search_text: String,
    pub sorted_program_links: Vec<ProgramLink>,
    
    // 设置相关
    pub link_popups: LinkPopups,
    
    // 需要清理的图标
    pub icon_will_clean: Vec<String>,
    // 缓存图标
    pub cached_icon: HashMap<String, HashSet<String>>,
    // 编辑模式
    pub edit_mode: bool,
    // 被唤起
    pub called: bool,
}

impl MyApp {
    pub fn new() -> Self {
        let pages = match LinkSave::load_conf(".links.json") {
            Ok(links_config) => {
                links_config.pages
            },
            Err(e) => {
                println!("{}", e);
                vec![Page::new(
                    "示例页面".to_string(), 
                    Vec::new()
                )]
            },
        };

        Self {
            pages,
            current_page_index: 0,
            title: "Debug: 右键此条目".to_string(),
            search_text: "".to_string(),
            sorted_program_links: Vec::new(),
            link_popups: LinkPopups::new(),
            cached_icon: HashMap::new(),
            icon_will_clean: Vec::new(),
            called: true,
            edit_mode: false,
        }
    }


    fn clean_unused_icon(&mut self, ui: &mut egui::Ui) {
        for icon_path in self.icon_will_clean.iter() {
            if self.cached_icon.get(icon_path).map_or(true, |set| set.is_empty()) {
                println!("释放图片资源 {}", icon_path);
                ui.ctx().forget_image(&format!("file://{}", icon_path));
                self.cached_icon.remove(icon_path);
            } else {
                println!("图片仍在被使用，将不会释放 {}", icon_path);
            }
        }
        self.icon_will_clean.clear();
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_ui(ui);
            self.clean_unused_icon(ui);
        });
    }
}

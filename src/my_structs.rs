use eframe::egui;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;


#[derive(Serialize, Deserialize)]
pub struct ProgramLink {
    pub name: String,
    pub icon_path: String,
    pub run_command: String,
    pub uuid: String,
}

impl ProgramLink {
    pub fn new(name: String, icon_path: String, run_command: String) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    pub program_links: Vec<ProgramLink>,
    pub title: String,
}


impl Page {
    pub fn new(title: String, programms: Vec<ProgramLink>) -> Self {

        Self {
            program_links: programms,
            title: title,
        }
    }
}



pub struct MyApp {
    pub pages: Vec<Page>,
    pub current_page_index: usize,
    pub title: String,
    pub search_text: String,
    pub setting_open: bool,
    pub cached_icon: HashMap<String, HashSet<String>>,
    // 设置窗口的UI closure
    pub current_setting_page: usize,
    pub current_setting_link: usize,
    pub temp_name: String,
    pub temp_icon_path: String,
    pub temp_run_command: String,
}

impl MyApp {
    pub fn new() -> Self {
        let pages = match Self::load_conf(".links.json") {
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
            title: "默认页面".to_string(),
            search_text: "".to_string(),
            setting_open: false,
            current_setting_page: 0,
            current_setting_link: 0,
            temp_name: "".to_string(),
            temp_icon_path: "".to_string(),
            temp_run_command: "".to_string(),
            cached_icon: HashMap::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("注册页面");
            self.main_ui(ui);
        });
    }
}

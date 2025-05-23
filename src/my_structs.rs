use egui;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use crate::pages::popups::Popups;
use crate::window::{self, event::UserEvent};

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


#[derive(Debug)]
pub struct LinkPosition {
    pub link_index: usize,
}

impl LinkPosition {
    pub fn new(link_index: usize) -> Self {
        Self {
            link_index: link_index,
        }
    }
}


pub const DOUBLE_ALT_COOLDOWN: u64 = 500;
pub struct MyApp {
    pub proxy: winit::event_loop::EventLoopProxy<UserEvent>,

    pub program_links: Vec<ProgramLink>,
    pub current_tag: Option<String>,
    pub title: String,
    pub search_text: String,
    pub sorted_program_links: Vec<ProgramLink>,
    
    // 设置相关
    pub popups: Popups,
    
    // 需要清理的图标
    pub icon_will_clean: Vec<String>,
    // 缓存图标
    pub cached_icon: HashMap<String, HashSet<String>>,
    // 编辑模式
    pub edit_mode: bool,
    // 被唤起
    pub called: Arc<Mutex<bool>>,
}

impl MyApp {
    pub fn new(
        called: Arc<Mutex<bool>>,
        proxy: winit::event_loop::EventLoopProxy<UserEvent>
    ) -> Self {
        let program_links = match crate::pages::popups::link::save::load_conf(".links.json") {
            Ok(links_config) => {
                links_config.program_links
            },
            Err(e) => {
                println!("{}", e);
                Vec::new()
            },
        };

        Self {
            proxy: proxy,

            program_links: program_links,
            current_tag: None,
            title: "BaroBoard 工具箱".to_string(),
            search_text: "".to_string(),
            sorted_program_links: Vec::new(),
            popups: Popups::new(),
            cached_icon: HashMap::new(),
            icon_will_clean: Vec::new(),
            called: called,
            edit_mode: false,
        }
    }


    pub fn clean_unused_icon(&mut self, ctx: &egui::Context) {
        for icon_path in self.icon_will_clean.iter() {
            if self.cached_icon.get(icon_path).map_or(true, |set| set.is_empty()) {
                println!("释放图片资源 {}", icon_path);
                ctx.forget_image(&format!("file://{}", icon_path));
                // ctx.forget_all_images();
                self.cached_icon.remove(icon_path);
            } else {
                println!("图片仍在被使用，将不会释放 {}", icon_path);
            }
        }
        self.icon_will_clean.clear();
    }

    // pub fn show_window(&self) {
    //     self.proxy
    //         .send_event(UserEvent::ShowWindow)
    //         .unwrap();
    // }

    pub fn hide_window(&self) {
        self.proxy
            .send_event(UserEvent::HideWindow)
            .unwrap();
    }
}

impl window::App for MyApp {
    fn update(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide_window();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_ui(ctx, ui);
            self.clean_unused_icon(ctx);
        });
    }
}

impl Drop for MyApp {
    fn drop(&mut self) {
        println!("MyApp 被销毁");
    }
}

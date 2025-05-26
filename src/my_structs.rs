use egui;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::process::Command;

use crate::pages::popups::Popups;
use crate::window::{self, event::UserEvent};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramLink {
    pub name: Vec<String>,
    pub icon_path: String,
    pub run_command: String,
    pub arguments: Vec<String>,
    pub tags: HashSet<String>,
    pub uuid: String,
}

impl Default for ProgramLink {
    fn default() -> Self {
        Self {
            name: Vec::new(),
            icon_path: "".to_string(),
            run_command: "".to_string(),
            arguments: Vec::new(),
            tags: HashSet::new(),
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

impl ProgramLink {
    pub fn new(name: Vec<String>, icon_path: String, run_command: String, argument: Vec<String>, tags: HashSet<String>) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
            arguments: argument,
            tags: tags,
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

pub struct MyApp {
    pub proxy: winit::event_loop::EventLoopProxy<UserEvent>,

    pub program_links: Vec<ProgramLink>,
    pub tags: HashSet<String>,
    pub current_tag: Option<String>,
    pub title: String,
    pub search_text: String,
    pub sorted_program_links: Vec<ProgramLink>,

    // 停止保存模式
    pub wont_save: bool,
    
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
        let mut popup = Popups::new();

        let links_config = crate::pages::popups::link::save::load_conf(crate::CONFIG_FILE_NAME);

        let (program_links, tags) =  match links_config {
            Ok(links_config) => {
                let version = links_config.get("version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                
                if version < crate::CONFIG_FILE_VERSION {
                    proxy.send_event(crate::event::UserEvent::ShowWindow).unwrap();
                    popup.config_file_too_old();
                    (Vec::new(), HashSet::new())
                } else {
                    // 尝试反序列化为正确的结构体
                    match serde_json::from_value::<crate::pages::popups::link::save::LinkConfigSchema>(links_config) {
                        Ok(config) => (config.program_links, config.tags),
                        Err(_) => {
                            proxy.send_event(crate::event::UserEvent::ShowWindow).unwrap();
                            popup.config_file_format_error();
                            (Vec::new(), HashSet::new())
                        }
                    }
                }
            },
            Err(e) => {
                println!("{}", e);
                // 检查文件是否存在
                if std::path::Path::new(crate::CONFIG_FILE_NAME).exists() {
                    proxy.send_event(crate::event::UserEvent::ShowWindow).unwrap();
                    popup.config_file_format_error();
                }
                (Vec::new(), HashSet::new())
            },
        };

        Self {  
            proxy: proxy,

            program_links: program_links,
            tags: tags,
            current_tag: None,
            title: "BaroBoard 工具箱".to_string(),
            search_text: "".to_string(),
            sorted_program_links: Vec::new(),
            popups: popup,
            cached_icon: HashMap::new(),
            icon_will_clean: Vec::new(),
            called: called,
            edit_mode: false,
            wont_save: false,
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

    pub fn run_program(&self, program_link: ProgramLink) {
        // 解析命令字符串，分离程序名和参数
        let command = program_link.run_command;
        let args = program_link.arguments;
        
        if command.is_empty() {
            println!("{} 运行失败: 命令为空", program_link.name.get(0).unwrap_or(&"".to_string()));
            return;
        }

        match Command::new(command).args(args).spawn() {
            Ok(_) => println!("{} 运行成功", program_link.name.get(0).unwrap_or(&"".to_string())),
            Err(e) => {
                println!("{} 运行失败: {}", program_link.name.get(0).unwrap_or(&"".to_string()), e);
            },
        }
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

pub fn sort_by_tag(program_links: Vec<ProgramLink>, tag: String) -> Vec<ProgramLink> {
    program_links
    .iter()
    .filter(|link| link.tags.contains(&tag))
    .cloned()
    .collect()
}

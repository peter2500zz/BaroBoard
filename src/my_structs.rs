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

    // 高级内容
    pub is_admin: bool,
    pub is_new_window: bool,

     // 自动生成
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

            is_admin: false,
            is_new_window: true,

            uuid: Uuid::new_v4().to_string(),
        }
    }
}

impl ProgramLink {
    pub fn new(name: Vec<String>, icon_path: String, run_command: String, argument: Vec<String>, tags: HashSet<String>, is_admin: bool, is_new_window: bool) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
            arguments: argument,
            tags: tags,
            is_admin: is_admin,
            is_new_window: is_new_window,
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

#[derive(Debug)]
pub struct MyApp {
    // 与窗口通信的代理
    pub proxy: winit::event_loop::EventLoopProxy<UserEvent>,

    // 程序链接
    pub program_links: Vec<ProgramLink>,
    // 标签
    pub tags: HashSet<String>,
    // 当前标签
    pub current_tag: Option<String>,
    // 标题
    pub title: String,
    // 搜索文本
    pub search_text: String,
    // 排序后的程序链接
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
    // 是否有悬浮文件
    pub is_hover_file: Option<String>,

    // 被唤起
    pub called: Arc<Mutex<bool>>,
}

impl MyApp {
    pub fn new(
        called: Arc<Mutex<bool>>,
        proxy: winit::event_loop::EventLoopProxy<UserEvent>
    ) -> Self {
        let mut wont_save = false;

        // 创建.baro文件夹
        if !std::path::Path::new(crate::CONFIG_SAVE_PATH).exists() {
            match std::fs::create_dir_all(crate::CONFIG_SAVE_PATH) {
                Ok(_) => println!("创建.baro文件夹成功"),
                Err(e) => {
                    println!("创建.baro文件夹失败: {}", e);
                    wont_save = true;
                },
            }
        }

        let mut popup = Popups::new();

        let links_config = crate::pages::popups::link::save::load_conf(format!("{}/{}", crate::CONFIG_SAVE_PATH, crate::CONFIG_FILE_NAME).as_str());

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
                if std::path::Path::new(format!("{}/{}", crate::CONFIG_SAVE_PATH, crate::CONFIG_FILE_NAME).as_str()).exists() {
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
            is_hover_file: None,
            wont_save: wont_save,
        }
    }

    pub fn clean_unused_icon(&mut self, ctx: &egui::Context) {
        for icon_path in self.icon_will_clean.iter() {
            if self.cached_icon.get(icon_path).map_or(true, |set| set.is_empty()) {
                println!("释放图片资源 {}", icon_path);
                ctx.forget_image(&format!("file://{}", icon_path));
                // ctx.forget_all_images();
                self.cached_icon.remove(icon_path);

                #[cfg(target_os = "windows")]
                {
                    let icon_path = format!("{}/cache/exe_icon/{:x}.png", crate::CONFIG_SAVE_PATH, md5::compute(icon_path.as_bytes()));
                    match std::fs::remove_file(icon_path.clone()) {
                        Ok(_) => println!("删除缓存图片资源 {} 成功", icon_path),
                        Err(e) => println!("删除缓存图片资源 {} 失败: {}", icon_path, e),
                    }
                }
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
        let is_admin = program_link.is_admin;
        let is_new_window = program_link.is_new_window;

        let program_name = program_link.name.get(0);
        
        if command.is_empty() {
            println!("{} 运行失败: 命令为空", program_name.unwrap_or(&"".to_string()));
            return;
        }

        #[cfg(target_os = "windows")]
        {
            // 根据不同的运行模式选择不同的执行方式
            let result = match (is_admin, is_new_window) {
                // 管理员权限 + 新窗口
                (true, true) => {
                    let mut ps_command = format!(
                        "Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Normal",
                        command.replace("'", "''")
                    );
                    if !args.is_empty() {
                        let args_str = args.join(" ");
                        ps_command.push_str(&format!(" -ArgumentList '{}'", args_str.replace("'", "''")));
                    }
                    
                    Command::new("powershell")
                        .args(["-Command", &ps_command])
                        .spawn()
                },
                // 仅管理员权限
                (true, false) => {
                    let mut ps_command = format!(
                        "Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Hidden",
                        command.replace("'", "''")
                    );
                    if !args.is_empty() {
                        let args_str = args.join(" ");
                        ps_command.push_str(&format!(" -ArgumentList '{}'", args_str.replace("'", "''")));
                    }
                    
                    Command::new("powershell")
                        .args(["-Command", &ps_command])
                        .spawn()
                },
                // 仅新窗口
                (false, true) => {
                    let mut cmd_args = vec!["/c", "start", "cmd", "/c"];
                    cmd_args.push(&command);
                    cmd_args.extend(args.iter().map(|s| s.as_str()));
                    
                    Command::new("cmd")
                        .args(cmd_args)
                        .spawn()
                },
                // 普通运行
                (false, false) => {
                    Command::new(command)
                        .args(args)
                        .spawn()
                }
            };

            match result {
                Ok(_) => println!("{} 运行成功", program_name.unwrap_or(&"".to_string())),
                Err(e) => {
                    println!("{} 运行失败: {}", program_name.unwrap_or(&"".to_string()), e);
                },
            }
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

    
    fn file_hover_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.is_hover_file.is_some() {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
            );

            ui.painter().text(
                screen_rect.center(),
                egui::Align2::CENTER_CENTER,
                "松开鼠标为此文件添加快捷方式",
                egui::FontId::proportional(24.0),
                egui::Color32::WHITE,
            );
        }
    }

    fn create_link_by_hover_file(&mut self, path: String) {
        // 如果是个目录或者文件不存在
        if std::path::Path::new(&path).is_dir() || !std::path::Path::new(&path).exists() {
            return;
        }

        // 如果是个exe文件（仅限windows）
        #[cfg(target_os = "windows")]
        if path.ends_with(".exe") {
            match self.save_exe_icon(path.clone()) {
                Ok(_) => println!("保存图标成功"),
                Err(e) => println!("保存图标失败: {}", e),
            }

            self.program_links.push(ProgramLink::new(
                vec![std::path::Path::new(&path).file_name().unwrap().to_str().unwrap().to_string()],
                path.clone(),
                path.clone(),
                Vec::new(),
                HashSet::new(),
                false,
                true,
            ));
        }
    }
}

impl window::App for MyApp {
    fn init(&mut self, ctx: &egui::Context) {
        println!("初始化成功");

        #[cfg(target_os = "windows")]
        {
            for program_link in self.program_links.iter() {
                if program_link.icon_path.ends_with(".exe") {
                    match self.save_exe_icon(program_link.icon_path.clone()) {
                        Ok(_) => println!("保存图标成功"),
                        Err(e) => println!("保存图标失败: {}", e),
                    }
                }
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide_window();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 顺序是重要的
            self.main_ui(ctx, ui);
            self.clean_unused_icon(ctx);
            self.file_hover_ui(ctx, ui);
        });
    }

    fn on_file_hovered(&mut self, path: String) {
        self.is_hover_file = Some(path);
    }

    // 文件悬浮取消
    fn on_file_hover_cancelled(&mut self) {
        self.is_hover_file = None;
    }

    // 文件释放
    fn on_file_dropped(&mut self, path: String) {
        self.is_hover_file = None;
        self.create_link_by_hover_file(path);
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

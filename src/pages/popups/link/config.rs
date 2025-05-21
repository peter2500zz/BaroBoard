use egui;
use std::collections::HashSet;
use rfd;

use crate::my_structs::*;


pub struct LinkConfig {
    pub called: bool,
    should_close: bool,
    is_new_link: bool,
    
    page_to_save: usize,
    index_of_the_link: usize,

    // 临时变量们
    pub name: String,
    pub icon_path: Option<String>,
    pub run_command: String,
}

impl LinkConfig {
    pub fn new() -> Self {
        Self {
            called: false,
            should_close: false,
            is_new_link: false,
            page_to_save: 0,
            index_of_the_link: 0,
            name: "".to_string(),
            icon_path: None,
            run_command: "".to_string(),
        }
    }


    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.called = true;
        self.is_new_link = false;
        self.page_to_save = position.page_index;
        self.index_of_the_link = position.link_index;

        self.name = link.name.clone();
        self.icon_path = Some(link.icon_path.clone());
        self.run_command = link.run_command.clone();
    }

    
    pub fn config_new_link(&mut self, position: LinkPosition) {
        self.called = true;
        self.is_new_link = true;
        self.page_to_save = position.page_index;

        self.name = "".to_string();
        self.icon_path = None;
        self.run_command = "".to_string();
    }
}



impl MyApp {
    pub  fn show_setting_window(&mut self, ui: &mut egui::Ui) {
        if self.link_popups.link_config.should_close {
            self.link_popups.link_config.called = false;
            self.link_popups.link_config.should_close = false;
        }

        // 设置页面
        egui::Window::new(if self.link_popups.link_config.is_new_link {
            "创建快捷方式"
        } else {
            "配置快捷方式"
        })
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        
        .fade_in(true)
        .fade_out(true)
        .open(&mut self.link_popups.link_config.called)

        .show(ui.ctx(), |ui| {
            
            if ui.add_sized(
                egui::vec2(96.0, 96.0),
                egui::ImageButton::new(format!("file://{}", &self.link_popups.link_config.icon_path.clone().unwrap_or("你还没有添加任何图片！".to_string())))
            ).clicked() {
                
                if let Some(path) = rfd::FileDialog::new()
                .add_filter("图片", &["png", "svg"])  //, "gif"])
                .pick_file() {
                    // 如果之前设置页面有图片，则尝试删除缓存
                    if let Some(icon_path) = self.link_popups.link_config.icon_path.clone() {
                        self.icon_will_clean.push(icon_path);
                    }
                    self.link_popups.link_config.icon_path = Some(path.display().to_string());
                }
            }
            
            ui.label(&self.link_popups.link_config.icon_path.clone().unwrap_or("↑ 你至少需要一张图片！".to_string()));

            ui.horizontal(|ui| {
                ui.label("名称");
                ui.add(egui::TextEdit::singleline(&mut self.link_popups.link_config.name).hint_text("e.g. 记事本"));
                
            });

            ui.horizontal(|ui| {
                ui.label("命令");
                ui.add(
                    egui::TextEdit::singleline(&mut self.link_popups.link_config.run_command).hint_text("e.g. C:\\Windows\\System32\\notepad.exe")
                )
                .context_menu(|ui| {
                    if ui.button("选择一个程序").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("任意文件", &["*"])
                            .pick_file() {
                                // 如果之前设置页面有图片，则尝试删除缓存
                                self.link_popups.link_config.run_command = path.display().to_string();
                            }
                        ui.close_menu();
                    }
                })
                ;
            });
            
            ui.label(
                egui::RichText::new("tip: 右键命令输入框可以打开路径选择器")
                    .weak()
            );

            ui.separator();
            // 保存与取消按钮
            ui.with_layout(egui::Layout {
                cross_align: egui::Align::RIGHT,
                ..Default::default()
            }, |ui| {ui.horizontal(|ui| {
                if self.link_popups.link_config.is_new_link {
                    ui.horizontal(|ui| {
                        if self.link_popups.link_config.icon_path.is_none() {
                            ui.disable();
                        }

                        let response = ui.button("创建");
                        let clicked = response.clicked();
                        if self.link_popups.link_config.icon_path.is_none() {
                            response.on_hover_text_at_pointer("请先添加图片");
                        }
                        
                        if clicked {
                            // 创建不需要清除之前的图片缓存
                            self.pages[self.link_popups.link_config.page_to_save].program_links.push(
                                ProgramLink::new(
                                    self.link_popups.link_config.name.clone(),
                                    self.link_popups.link_config.icon_path.clone().unwrap_or("".to_string()),
                                    self.link_popups.link_config.run_command.clone(),
                                )
                            );
                            
                            match self.link_popups.link_save.save_conf(self.pages.clone()) {
                                Ok(_) => println!("保存成功"),
                                Err(e) => {
                                    println!("保存失败: {}", e);
                                    self.link_popups.link_save.error_called = true;
                                },
                            };

                            self.link_popups.link_config.is_new_link = false;
                            self.link_popups.link_config.should_close = true;
                        }
                    });
                    
                } else {
                    if ui.button("保存").clicked() {
                        let current_link = &mut self.pages[self.link_popups.link_config.page_to_save].program_links[self.link_popups.link_config.index_of_the_link];
                        // 尝试移除之前的缓存标记
                        if let Some(icon_path) = self.cached_icon.get_mut(&current_link.icon_path) {
                            icon_path.remove(&current_link.uuid);
                        } else {
                            // 如果不行则强制清空
                            self.cached_icon.insert(current_link.icon_path.clone(), HashSet::new());
                        }

                        // 如果缓存为空，则删除缓存
                        self.icon_will_clean.push(current_link.icon_path.clone());

                        current_link.name = self.link_popups.link_config.name.clone();
                        current_link.icon_path = self.link_popups.link_config.icon_path.clone().unwrap_or("".to_string());
                        current_link.run_command = self.link_popups.link_config.run_command.clone();

                        match self.link_popups.link_save.save_conf(self.pages.clone()) {
                            Ok(_) => println!("保存成功"),
                            Err(e) => {
                                println!("保存失败: {}", e);
                                self.link_popups.link_save.error_called = true;
                            },
                        };
                        
                        self.link_popups.link_config.should_close = true;
                    }
                }

                if ui.button("取消").clicked() {
                    // 如果此图片没有被其他程序使用，则删除缓存
                    if let Some(icon_path) = self.link_popups.link_config.icon_path.clone() {
                        self.icon_will_clean.push(icon_path);
                    }
                    self.link_popups.link_config.should_close = true;
                }
            })});
        });
    }
}


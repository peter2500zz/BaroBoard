use egui;
use std::collections::HashSet;
use rfd;

use crate::my_structs::*;


pub struct LinkConfig {
    is_new_link: bool,
    
    index_of_the_link: usize,

    // 临时变量们
    pub name: String,
    pub icon_path: Option<String>,
    pub run_command: String,
    pub tags: HashSet<String>,
}

impl LinkConfig {
    pub fn new() -> Self {
        Self {
            is_new_link: false,
            index_of_the_link: 0,
            name: "".to_string(),
            icon_path: None,
            run_command: "".to_string(),
            tags: HashSet::new(),
        }
    }


    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.is_new_link = false;
        self.index_of_the_link = position.link_index;

        self.name = link.name.clone().join("/");
        self.icon_path = Some(link.icon_path.clone());
        self.run_command = link.run_command.clone();
        self.tags = HashSet::from_iter(link.tags.clone());
    }

    
    pub fn config_new_link(&mut self) {
        self.is_new_link = true;

        self.name = "".to_string();
        self.icon_path = None;
        self.run_command = "".to_string();
        self.tags = HashSet::new();
    }
}



impl MyApp {
    pub fn show_link_config(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_save = false;
        let mut should_close = false;

        // 设置页面
        egui::Window::new(if self.popups.link_config.is_new_link {
            "创建快捷方式"
        } else {
            "配置快捷方式"
        })
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {

            if ui.add_sized(
                egui::vec2(96.0, 96.0),
                egui::ImageButton::new(format!("file://{}", &self.popups.link_config.icon_path.clone().unwrap_or("你还没有添加任何图片！".to_string())))
            ).clicked() {
                
                if let Some(path) = rfd::FileDialog::new()
                .add_filter("图片", &["png", "svg"])  //, "gif"])
                .pick_file() {
                    // 如果之前设置页面有图片，则尝试删除缓存
                    if let Some(icon_path) = self.popups.link_config.icon_path.clone() {
                        self.icon_will_clean.push(icon_path);
                    }
                    self.popups.link_config.icon_path = Some(path.display().to_string());
                }
            }

            ui.label(&self.popups.link_config.icon_path.clone().unwrap_or("↑ 你至少需要一张图片！".to_string()));

            ui.horizontal(|ui| {
                ui.label("名称");
                ui.add(egui::TextEdit::singleline(&mut self.popups.link_config.name).hint_text("e.g. 记事本/notepad"));
                
            });

            ui.horizontal(|ui| {
                ui.label("命令");
                ui.add(
                    egui::TextEdit::singleline(&mut self.popups.link_config.run_command).hint_text("e.g. C:\\Windows\\System32\\notepad.exe")
                )
                .context_menu(|ui| {
                    if ui.button("选择一个程序").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("任意文件", &["*"])
                            .pick_file() {
                                self.popups.link_config.run_command = path.display().to_string();
                            }
                        ui.close_menu();
                    }
                })
                ;
            });

            ui.label(
                egui::RichText::new("tip: 名称可以使用 / 来创建别名，也可以只输入一个名称。右键命令输入框可以打开路径选择器")
                    .weak()
            );


            egui::ComboBox::from_label("选择标签")
                .selected_text(if self.popups.link_config.tags.is_empty() {
                    "无标签".to_string()
                } else {
                    let tag_counts = self.popups.link_config.tags
                        .iter()
                        .filter(|&tag| self.tags.contains(tag))
                        .collect::<Vec<_>>()
                        .len();
                    if tag_counts == 0 {
                        "无标签".to_string()
                    } else {
                        format!("{} 个标签", tag_counts)
                    }
                })
                .truncate()
                .show_ui(ui, |ui| {
                    if self.tags.is_empty() {
                        ui.label(egui::RichText::new("你还没有任何标签").weak());
                    }

                    for tag in &self.tags {
                        let is_select = self.popups.link_config.tags.contains(tag);
                        let mut selected = is_select.clone();

                        ui.checkbox(
                            &mut selected,
                            tag.clone()
                        );
                        
                        if selected {
                            if !is_select {
                                self.popups.link_config.tags.insert(tag.clone());
                            }
                        } else {
                            self.popups.link_config.tags.remove(tag);
                        }
                    }
                });


            ui.separator();
            // 保存与取消按钮
            ui.with_layout(egui::Layout {
                cross_align: egui::Align::RIGHT,
                ..Default::default()
            }, |ui| {ui.horizontal(|ui| {
                if self.popups.link_config.is_new_link {
                    ui.horizontal(|ui| {
                        if self.popups.link_config.icon_path.is_none() {
                            ui.disable();
                        }

                        let response = ui.button("创建");
                        let clicked = response.clicked();
                        if self.popups.link_config.icon_path.is_none() {
                            response.on_hover_text_at_pointer("请先添加图片");
                        }
                        
                        if clicked {
                            // 创建不需要清除之前的图片缓存
                            self.program_links.push(
                                ProgramLink::new(
                                    self.popups.link_config.name.clone().split("/").map(|s| s.to_string()).collect(),
                                    self.popups.link_config.icon_path.clone().unwrap_or("".to_string()),
                                    self.popups.link_config.run_command.clone(),
                                    self.popups.link_config.tags.clone().into_iter().collect()
                                )
                            );
                            
                            should_save = true;
                            should_close = true;
                        }
                    });
                    
                } else {
                    if ui.button("保存").clicked() {
                        let current_link = &mut self.program_links[self.popups.link_config.index_of_the_link];
                        // 尝试移除之前的缓存标记
                        // 我的意思是原本的快捷方式图片而不是设置中的预览
                        if let Some(icon_path) = self.cached_icon.get_mut(&current_link.icon_path) {
                            icon_path.remove(&current_link.uuid);
                        } else {
                            // 如果不行则强制清空
                            self.cached_icon.insert(current_link.icon_path.clone(), HashSet::new());
                        }

                        // 如果缓存为空，则删除缓存
                        self.icon_will_clean.push(current_link.icon_path.clone());

                        current_link.name = self.popups.link_config.name.clone().split("/").map(|s| s.to_string()).collect();
                        current_link.icon_path = self.popups.link_config.icon_path.clone().unwrap_or("".to_string());
                        current_link.run_command = self.popups.link_config.run_command.clone();
                        current_link.tags = self.popups.link_config.tags.clone().into_iter().collect();

                        should_save = true;
                        should_close = true;
                    }
                }

                if ui.button("取消").clicked() {
                    // 如果此图片没有被其他程序使用，则删除缓存
                    
                    should_close = true;
                }
            })});
        });


        if (!show && !should_close && self.popups.called) || should_close {
            // 只有在窗口还是打开状态时才执行清理
            if self.popups.called {
                println!("*你* 关闭了对吧？");
                // 用户关闭
                if let Some(icon_path) = self.popups.link_config.icon_path.clone() {
                    if !should_save {
                        self.icon_will_clean.push(icon_path);
                    }
                }
                
                if should_save {
                    self.popups.save_conf(self.program_links.clone(), self.tags.clone());
                }

                self.popups.called = false;
            }
        }
    }
}


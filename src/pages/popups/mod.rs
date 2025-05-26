pub mod link;

use std::collections::HashSet;

use link::save;
use crate::my_structs::*;

pub struct LinkToDelete {
    index_of_the_link: usize,
}

impl LinkToDelete {
    pub fn new() -> Self {
        Self {
            index_of_the_link: 0
        }
    }
}

#[derive(Clone)]
pub enum PopupType {
    LinkConfig,
    LinkDelete,
    CannotSave,
    TagDelete,
    TagNew,

    // 配置文件错误
    ConfigTooOld,
    ConfigFormatError,
    ConfigNotAJson,
}

pub struct Popups {
    pub called: bool,

    popup_type: Option<PopupType>,

    // 临时变量
    link_config: link::config::LinkConfig,
    link_to_delete: LinkToDelete,
    tag_to_delete: String,
    tag_new: String,
    // pub info: info::Info,
}

impl Popups {
    pub fn new() -> Self {
        Self {
            called: false,
            popup_type: None,
            link_config: link::config::LinkConfig::new(),
            link_to_delete: LinkToDelete::new(),
            tag_to_delete: "".to_string(),
            tag_new: "".to_string(),
            // info: info::Info::new(),
        }
    }

    pub fn cannot_save(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::CannotSave);
    }

    pub fn delete_link(&mut self, position: LinkPosition) {
        println!("index {:?}", position);
        self.called = true;
        self.popup_type = Some(PopupType::LinkDelete);
        self.link_to_delete.index_of_the_link = position.link_index;
    }

    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_existing_link(position, link);
    }

    pub fn config_new_link(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_new_link();
    }

    pub fn delete_tag(&mut self, tag: String) {
        self.called = true;
        self.popup_type = Some(PopupType::TagDelete);
        self.tag_to_delete = tag;
    }

    pub fn new_tag(&mut self) {
        self.called = true;
        self.tag_new = "".to_string();
        self.popup_type = Some(PopupType::TagNew);
    }

    pub fn config_file_too_old(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::ConfigTooOld);
    }

    pub fn config_file_format_error(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::ConfigFormatError);
    }

    fn config_not_a_json(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::ConfigNotAJson);
    }
}


impl MyApp {
    pub fn show_popup(&mut self, ui: &mut egui::Ui) {
        // if self.popups.called {
            if let Some(popup_type) = self.popups.popup_type.clone() {
                match popup_type {
                    PopupType::LinkConfig => self.show_link_config(ui),
                    PopupType::LinkDelete => self.show_delete_link(ui),
                    PopupType::TagDelete => self.show_delete_tag(ui),
                    PopupType::TagNew => self.show_new_tag(ui),
                    PopupType::ConfigTooOld => self.show_config_file_too_old(ui),
                    PopupType::ConfigFormatError => self.show_config_file_format_error(ui),
                    PopupType::ConfigNotAJson => self.show_config_not_a_json(ui),
                    // PopupType::Info => self.popups.info.show(ui),
                    _ => {}
                }
            }
        // }
    }

    fn show_config_not_a_json(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;

        egui::Window::new("此配置文件不是一个有效的JSON文件")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("此配置文件不是一个有效的JSON文件");
                ui.label(
                    "请检查文件结构是否正确。"
                );
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("好的").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });

        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;
        }
    }

    fn show_config_file_format_error(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_auto_fix = false;

        egui::Window::new("无法读取配置文件")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("嘿！我无法读取你的配置文件！");
                ui.separator();
                ui.label(
                    "这可能是由于配置文件的版本过旧，或是配置文件被损坏。"
                );
                ui.label(
                    "你可以尝试自动修复程序，否则为了安全起见，程序将不会自动保存你的任何操作，直到你有了一份正确的配置文件。"
                );
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("尝试修复").color(egui::Color32::RED))
                        .clicked() {
                            should_auto_fix = true;
                            should_close = true;
                        }
                        if ui.button("在无自动保存的情况下继续").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });

        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            if should_auto_fix {
                self.config_auto_fix();
            }

            self.popups.called = false;
        }
    }

    fn show_config_file_too_old(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_force_read = false;

        // 删除快捷方式弹窗
        egui::Window::new("配置文件版本过旧")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("配置文件版本过旧");
                ui.label(
                    "仍然尝试读取？"
                );
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("是").color(egui::Color32::RED))
                        .clicked() {
                            should_force_read = true;
                            should_close = true;
                        }
                        if ui.button("否").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            
            self.popups.called = false;

            if should_force_read {
                println!("强制读取");
                self.force_read_config();
            }
        }
    }

    fn show_new_tag(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("创建一个新的标签")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("创建一个新的标签");

                ui.add(egui::TextEdit::singleline(&mut self.popups.tag_new).hint_text("请输入标签名称"));
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {

                        if self.popups.tag_new.is_empty() {
                            ui.disable();
                        }

                        if ui.button(egui::RichText::new("创建"))
                        .clicked() {
                            self.tags.insert(self.popups.tag_new.clone());

                            println!("创建成功: {:?}", self.popups.tag_new);

                            should_save = true;
                            should_close = true;
                        }
                        });

                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }

    fn show_delete_tag(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("你确定要删除这个标签吗？")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个标签吗？");
                ui.label(format!(
                    "所有快捷方式的 “{}” 标签将会被删除", 
                    self
                    // 这里不能unwarp的原因是
                    // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
                    .popups.tag_to_delete
                ));
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            self.tags.remove(&self.popups.tag_to_delete);

                            println!("删除成功: {:?}", self.popups.tag_to_delete);

                            should_save = true;
                            should_close = true;
                        }
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }

    fn show_delete_link(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("你确定要删除这个快捷方式吗？")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            let current_index = self.popups.link_to_delete.index_of_the_link;

            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个快捷方式吗？");
                ui.label(format!(
                    "“{}”将会永久消失！（真的很久！）", 
                    self
                    // 这里不能unwarp的原因是
                    // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
                    .program_links.get(current_index).unwrap_or(&ProgramLink::default())
                    //(ProgramLink { name: "已删除".to_string(), ..Default::default()})
                    .name.get(0).unwrap_or(&"已删除".to_string())
                ));
                
                ui.separator();
                
                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            let program_links = &mut self.program_links;

                            if let Some(icon_path) = self.cached_icon.get_mut(&program_links[current_index].icon_path) {
                                icon_path.remove(&program_links[current_index].uuid);
                            } else {
                                // 如果不行则强制清空
                                self.cached_icon.insert(program_links[current_index].icon_path.clone(), HashSet::new());
                            }
                            self.icon_will_clean.push(program_links[current_index].icon_path.clone());

                            let name = program_links[current_index].name.clone();
                            program_links.remove(current_index);
                            println!("删除成功: {:?}", name);

                            should_save = true;
                            should_close = true;
                        }
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }
}


impl MyApp {
    fn config_auto_fix(&mut self) {
        let config_file = crate::pages::popups::link::save::load_conf(crate::CONFIG_FILE_NAME);

        match config_file {
            Ok(links_config) => {
                // 开始尝试修复
                let mut new_links_config = crate::pages::popups::link::save::LinkConfigSchema::default();

                // 尝试获取版本号
                if let Some(version) = links_config.get("version") {
                    if let Some(version_int) = version.as_u64() {
                        new_links_config.version = version_int as u32;
                    }
                }

                // 尝试获取tags
                if let Some(tags) = links_config.get("tags") {
                    if let Some(tags_list) = tags.as_array() {
                        for tag in tags_list {
                            if let Some(tag_str) = tag.as_str() {
                                new_links_config.tags.insert(tag_str.to_string());
                            }
                        }
                    } else if let Some(tag) = tags.as_str() {
                        new_links_config.tags.insert(tag.to_string());
                    }
                }

                // 尝试获取program_links
                if let Some(program_links) = links_config.get("program_links") {
                    if let Some(program_links_list) = program_links.as_array() {
                        for program_link in program_links_list {
                            let mut new_program_link = ProgramLink::default();

                            // 尝试获取name
                            if let Some(name) = program_link.get("name") {
                                if let Some(name_list) = name.as_array() {
                                    for name_item in name_list {
                                        if let Some(name_str) = name_item.as_str() {
                                            new_program_link.name.push(name_str.to_string());
                                        }
                                    }
                                } else if let Some(name_str) = name.as_str() {
                                    new_program_link.name.push(name_str.to_string());
                                }
                            }

                            // 尝试获取icon_path
                            if let Some(icon_path) = program_link.get("icon_path") {
                                if let Some(icon_path_str) = icon_path.as_str() {
                                    new_program_link.icon_path = icon_path_str.to_string();
                                }
                            }

                            // 尝试获取run_command
                            if let Some(run_command) = program_link.get("run_command") {
                                if let Some(run_command_str) = run_command.as_str() {
                                    new_program_link.run_command = run_command_str.to_string();
                                }
                            }

                            // 尝试获取argument
                            if let Some(arguments) = program_link.get("arguments") {
                                if let Some(arguments_list) = arguments.as_array() {
                                    for argument_item in arguments_list {
                                        if let Some(argument_item_str) = argument_item.as_str() {
                                            new_program_link.arguments.push(argument_item_str.to_string());
                                        }
                                    }
                                } else if let Some(argument_str) = arguments.as_str() {
                                    new_program_link.arguments.push(argument_str.to_string());
                                }
                            }

                            // 尝试获取tags
                            if let Some(tags) = program_link.get("tags") {
                                if let Some(tags_list) = tags.as_array() {
                                    for tag in tags_list {
                                        if let Some(tag_str) = tag.as_str() {
                                            new_program_link.tags.insert(tag_str.to_string());
                                        }
                                    }
                                } else if let Some(tag) = tags.as_str() {
                                    new_program_link.tags.insert(tag.to_string());
                                }
                            }
                            
                            // 尝试获取uuid
                            if let Some(uuid) = program_link.get("uuid") {
                                if let Some(uuid_str) = uuid.as_str() {
                                    new_program_link.uuid = uuid_str.to_string();
                                }
                            }

                            new_links_config.program_links.push(new_program_link);
                        }
                    }
                }

                self.program_links = new_links_config.program_links;
                self.tags = new_links_config.tags;

                self.save_conf();
            }
            Err(_) => {
                self.popups.config_not_a_json();
            }
        }
    }

    fn force_read_config(&mut self) {
        let (program_links, tags) = match serde_json::from_value::<crate::pages::popups::link::save::LinkConfigSchema>(crate::pages::popups::link::save::load_conf(crate::CONFIG_FILE_NAME).unwrap()) {
            Ok(links_config) => (links_config.program_links, links_config.tags),
            Err(e) => {
                println!("读取配置文件失败: {}", e);
                self.popups.config_file_format_error();
                (Vec::new(), HashSet::new())
            }
        };

        self.program_links = program_links;
        self.tags = tags;
    }

    pub fn save_conf(&mut self) {
        if !self.wont_save {
            match save::save_conf(
                self.program_links.clone().into_iter().map(|mut link| {
                    link.tags = link.tags.clone().into_iter().filter(|tag| self.tags.contains(tag)).collect();
                    link
                }).collect()
                , 
                self.tags.clone()
            ) {
                Ok(_) => println!("保存成功"),
                Err(e) => {
                    println!("保存失败: {}", e);
                    self.popups.cannot_save();
                },
            }
        } else {
            println!("停止保存模式");
        }
    }
}

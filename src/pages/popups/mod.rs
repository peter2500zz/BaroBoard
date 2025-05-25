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

    pub fn save_conf(&mut self, program_links: Vec<ProgramLink>, tags: HashSet<String>) {
        match save::save_conf(
            program_links.into_iter().map(|mut link| {
                link.tags = link.tags.clone().into_iter().filter(|tag| tags.contains(tag)).collect();
                link
            }).collect()
            , 
            tags
        ) {
            Ok(_) => println!("保存成功"),
            Err(e) => {
                println!("保存失败: {}", e);
                self.called = true;
                self.popup_type = Some(PopupType::CannotSave);
            },
        }
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
                    // PopupType::Info => self.popups.info.show(ui),
                    _ => {}
                }
            }
        // }
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
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            println!("*你* 关闭了对吧？");
            // 用户关闭
            if should_force_read {
                println!("强制读取");
                self.force_read_config();
            }

            self.popups.called = false;
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
            if should_save {
                self.popups.save_conf(self.program_links.clone(), self.tags.clone());
            }

            self.popups.called = false;
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
            if should_save {
                self.popups.save_conf(self.program_links.clone(), self.tags.clone());
            }

            self.popups.called = false;
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
            if should_save {
                self.popups.save_conf(self.program_links.clone(), self.tags.clone());
            }

            self.popups.called = false;
        }
    }
}


impl MyApp {
    fn force_read_config(&mut self) {
        let (program_links, tags) = match serde_json::from_value::<crate::pages::popups::link::save::LinkConfigSchema>(crate::pages::popups::link::save::load_conf(".links.json").unwrap()) {
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
}

mod config_link;
mod config_not_a_json;
mod config_file_format_error;
mod config_file_too_old;
mod new_here;
mod new_tag;
mod delete_tag;
mod delete_link;

use std::collections::HashSet;

use log::debug;
use crate::my_structs::*;
use crate::utils::save;

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub enum PopupType {
    LinkConfig,
    LinkDelete,
    CannotSave,
    TagDelete,
    TagNew,

    // 新用户
    NewHere,

    // 配置文件错误
    ConfigTooOld,
    ConfigFormatError,
    ConfigNotAJson,
}

#[derive(Debug)]
pub struct Popups {
    pub called: bool,

    popup_type: Option<PopupType>,

    // 临时变量
    link_config: config_link::LinkConfig,
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
            link_config: config_link::LinkConfig::new(),
            link_to_delete: LinkToDelete::new(),
            tag_to_delete: "".to_string(),
            tag_new: "".to_string(),
            // info: info::Info::new(),
        }
    }

    pub fn cannot_save(&mut self) {
        debug!("请求无法保存弹窗");
        self.called = true;
        self.popup_type = Some(PopupType::CannotSave);
    }

    pub fn delete_link(&mut self, position: LinkPosition) {
        debug!("请求删除快捷方式弹窗，位置: {:?}", position);
        self.called = true;
        self.popup_type = Some(PopupType::LinkDelete);
        self.link_to_delete.index_of_the_link = position.link_index;
    }

    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        debug!("请求配置快捷方式弹窗，位置: {:?}", position);
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_existing_link(position, link);
    }

    pub fn config_new_link(&mut self) {
        debug!("请求配置新快捷方式弹窗");
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_new_link();
    }

    pub fn delete_tag(&mut self, tag: String) {
        debug!("请求删除标签弹窗，标签: {}", tag);
        self.called = true;
        self.popup_type = Some(PopupType::TagDelete);
        self.tag_to_delete = tag;
    }

    pub fn new_tag(&mut self) {
        debug!("请求创建新标签弹窗");
        self.called = true;
        self.tag_new = "".to_string();
        self.popup_type = Some(PopupType::TagNew);
    }

    pub fn new_here(&mut self) {
        debug!("这家伙第一次用哦");
        self.called = true;
        self.popup_type = Some(PopupType::NewHere);
    }

    pub fn config_file_too_old(&mut self) {
        debug!("请求配置文件过旧弹窗");
        self.called = true;
        self.popup_type = Some(PopupType::ConfigTooOld);
    }

    pub fn config_file_format_error(&mut self) {
        debug!("请求配置文件格式错误弹窗");
        self.called = true;
        self.popup_type = Some(PopupType::ConfigFormatError);
    }

    fn config_not_a_json(&mut self) {
        debug!("请求配置文件不是JSON弹窗");
        self.called = true;
        self.popup_type = Some(PopupType::ConfigNotAJson);
    }
}


impl MyApp {
    pub fn show_popup(&mut self, ui: &mut egui::Ui) {
        if let Some(popup_type) = self.popups.popup_type.clone() {
            match popup_type {
                PopupType::LinkConfig => self.show_config_link(ui),
                PopupType::LinkDelete => self.show_delete_link(ui),
                PopupType::TagDelete => self.show_delete_tag(ui),
                PopupType::TagNew => self.show_new_tag(ui),
                PopupType::NewHere => self.show_new_here(ui),
                PopupType::ConfigTooOld => self.show_config_file_too_old(ui),
                PopupType::ConfigFormatError => self.show_config_file_format_error(ui),
                PopupType::ConfigNotAJson => self.show_config_not_a_json(ui),
                PopupType::CannotSave => todo!(),
            }
        }
    }
}


impl MyApp {
    fn config_auto_fix(&mut self) {
        let config_file = save::load_conf(
            &format!("{}/{}", crate::CONFIG_SAVE_PATH, crate::CONFIG_FILE_NAME)
        );

        match config_file {
            Ok(links_config) => {
                // 开始尝试修复
                let mut new_links_config = save::LinkConfigSchema::default();

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

                            // 尝试获取working_directory
                            if let Some(working_directory) = program_link.get("working_directory") {
                                if let Some(working_directory_str) = working_directory.as_str() {
                                    new_program_link.working_directory = working_directory_str.to_string();
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

                            // 尝试获取is_admin
                            if let Some(is_admin) = program_link.get("is_admin") {
                                if let Some(is_admin_bool) = is_admin.as_bool() {
                                    new_program_link.is_admin = is_admin_bool;
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
        let (program_links, tags) = match serde_json::from_value::<save::LinkConfigSchema>(
            save::load_conf(&format!("{}/{}", crate::CONFIG_SAVE_PATH, crate::CONFIG_FILE_NAME)).unwrap()
        ) {
            Ok(links_config) => (links_config.program_links, links_config.tags),
            Err(e) => {
                debug!("读取配置文件失败: {}", e);
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
                Ok(_) => debug!("保存成功"),
                Err(e) => {
                    debug!("保存失败: {}", e);
                    self.popups.cannot_save();
                },
            }
        } else {
            debug!("停止保存模式");
        }
    }
}

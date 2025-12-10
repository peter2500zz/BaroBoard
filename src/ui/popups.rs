pub mod config_link;
pub mod config_not_a_json;
pub mod config_file_format_error;
pub mod config_file_too_old;
pub mod new_here;
pub mod new_tag;
pub mod delete_tag;
pub mod delete_link;

use std::fmt::Debug;

use std::collections::{HashSet, VecDeque};

use log::debug;
use crate::my_structs::*;
use crate::ui::popups::config_file_format_error::ConfigFormatError;
use crate::ui::popups::config_not_a_json::ConfigNotAJson;
use crate::utils::save;


pub trait Popup: Debug {
    /// 当返回值为 true 时，弹窗会关闭
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool;

    /// debug 模式下打印的内容
    fn close_desc(&self) -> String {
        "弹窗关闭".to_string()
    }

    /// 弹窗每一刻会执行的内容
    fn on_update(&mut self) -> Option<Box<dyn FnOnce(&mut MyApp)>>  {
        None
    }

    /// 弹窗关闭时会执行的内容
    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>>  {
        None
    }
}


#[derive(Debug, Default)]
pub struct PopupMgr {
    showing: bool,

    popup: Option<Box<dyn Popup>>,
    popup_queue: VecDeque<Box<dyn Popup>>,
}

impl PopupMgr {
    /// 显示一个弹窗，如果当前已经有一个弹窗就不会显示并返回 false
    pub fn show(&mut self, popup: Box<dyn Popup>) -> bool {
        if self.popup.is_some() {
            debug!("因为已经有一个弹窗了，{:?} 将不会被显示", popup);

            return false;
        }

        self.popup = Some(popup);
        self.showing = true;

        true
    }

    /// 将一个弹窗加入队列，之后按队列依次显示弹窗
    pub fn queue(&mut self, popup: Box<dyn Popup>) {
        self.popup_queue.push_back(popup);
    }

    pub fn can_show(&self) -> bool {
        !self.showing
    }
}

impl MyApp {
    pub fn show_popup(&mut self, ui: &mut egui::Ui) {
        let popupmgr = &mut self.popupgmr;

        // 确认是否需要显示弹窗
        if let Some(popup) = &mut popupmgr.popup {
            // 保留上一次的显示状态
            let showed = popupmgr.showing;
            // 因为这里可能改变 show
            let need_clean = popup.show(ui, &mut popupmgr.showing);
            // 如果上次还在显示，这次不显示了，关闭
            if showed && !popupmgr.showing {
                debug!("{}", popup.close_desc());

                if let Some(on_close) = popup.on_close() {
                    on_close(self)
                }

                self.popupgmr.showing = false;
            } else {
                if let Some(on_update) = popup.on_update() {
                    on_update(self)
                }
            }
            // 直到弹窗认为自己需要被清理，清理弹窗
            if need_clean {
                debug!("弹窗已被清理");
                self.popupgmr.popup = None
            }
        } else if !self.popupgmr.popup_queue.is_empty() {
            self.popupgmr.popup = self.popupgmr.popup_queue.pop_front();
            self.popupgmr.showing = true;
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
                self.popupgmr.queue(ConfigNotAJson::new());
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
                self.popupgmr.queue(ConfigFormatError::new());
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
                    todo!();
                },
            }
        } else {
            debug!("停止保存模式");
        }
    }
}

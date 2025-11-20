use egui::{self, TextBuffer};
use std::collections::HashSet;
use rfd;
use std::path::Path;
use log::debug;

use crate::{my_structs::*, texture_mgr::save_icon};

/// 表示参数在列表中的索引位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArgumentIndex(usize);

/// 快捷方式配置
#[derive(Debug)]
pub struct LinkConfig {
    is_new_link: bool,
    
    index_of_the_link: usize,

    // 临时变量们
    pub name: String,
    pub icon_path: Option<String>,
    pub file_path: String,
    pub working_directory: String,
    pub arguments: Vec<String>,
    pub tags: HashSet<String>,
    pub is_admin: bool,

    // 子窗口配置
    show_args_config: bool,
    args_scroll_to_bottom: bool,
    show_advanced_config: bool,
}

impl LinkConfig {
    pub fn new() -> Self {
        Self {
            is_new_link: false,
            index_of_the_link: 0,
            name: "".to_string(),
            icon_path: None,
            file_path: "".to_string(),
            working_directory: "".to_string(),
            arguments: Vec::new(),
            tags: HashSet::new(),
            is_admin: false,

            show_args_config: false,
            args_scroll_to_bottom: false,
            show_advanced_config: false,
        }
    }


    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.is_new_link = false;
        self.index_of_the_link = position.link_index;

        self.name = link.name.clone().join("/");
        self.icon_path = Some(link.icon_path.clone());
        self.file_path = link.run_command.clone();
        self.working_directory = link.working_directory.clone();
        self.arguments = link.arguments.clone();
        self.tags = HashSet::from_iter(link.tags.clone());
        self.is_admin = link.is_admin;
    }

    
    pub fn config_new_link(&mut self) {
        *self = Self::new();
        self.is_new_link = true;
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
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        // .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {

            if ui.add_sized(
                egui::vec2(96.0, 96.0),
                egui::ImageButton::new(format!("file://{}", &self.popups.link_config.icon_path.clone().unwrap_or("你还没有添加任何图片！".to_string())))
            ).clicked() {
                let can_display = vec!["png", "svg"];

                if let Some(path) = rfd::FileDialog::new()
                .pick_file() {
                    // 如果之前设置页面有图片，则尝试删除缓存
                    if let Some(icon_path) = self.popups.link_config.icon_path.clone() {
                        self.texture_mgr.schedule_forget(icon_path);
                    }

                    let mut icon_path = path.display().to_string();

                    if !can_display.contains(&path.extension().unwrap().to_string_lossy().as_str()) {
                        icon_path = match save_icon(&icon_path) {
                            Ok(icon_path) => icon_path,
                            Err(e) => {
                                debug!("保存图标失败: {}", e);
                                "读取exe图标失败".to_string()
                            }
                        };
                    }

                    self.popups.link_config.icon_path = Some(icon_path);
                }
            }

            ui.label(&self.popups.link_config.icon_path.clone().unwrap_or("↑ 你至少需要一张图片！".to_string()));

            ui.horizontal(|ui| {
                ui.label("名称");
                ui.add(egui::TextEdit::singleline(&mut self.popups.link_config.name).hint_text("e.g. 记事本/notepad"));
                
            });

            ui.horizontal(|ui| {
                ui.label("路径");
                ui.add(
                    egui::TextEdit::singleline(&mut self.popups.link_config.file_path).hint_text("e.g. C:\\Windows\\System32\\notepad.exe")
                )
                .context_menu(|ui| {
                    if ui.button("选择一个文件").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("任意文件", &["*"])
                            .pick_file() {
                                self.popups.link_config.file_path = path.display().to_string();
                            }
                        ui.close_menu();
                    }
                })
                ;
            });

            let arguments_config_window = egui::Window::new("参数配置")
            .collapsible(false)
            .resizable(false)
            .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
            .open(&mut self.popups.link_config.show_args_config)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let mut has_empty_argument = false;

                egui::ScrollArea::vertical()
                .max_height(256.)
                .show(ui, |ui| {
                // ui.vertical(|ui| {
                    let mut index_should_remove: Option<usize> = None;
                    let mut drag_from = None;
                    let mut drag_to = None;

                    for (index, _) in self.popups.link_config.arguments.clone().iter().enumerate() {
                        let response = ui.horizontal(|ui| {
                            // 只让标签部分可拖拽
                            let drag_response = ui.dnd_drag_source(
                                egui::Id::new(format!("arg_{}", index)), 
                                ArgumentIndex(index), 
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("☰ 参数 {}", index + 1)));
                                    });
                                }
                            ).response;

                            // 输入框和按钮在拖拽区域外
                            if self.popups.link_config.arguments[index].is_empty() {
                                has_empty_argument = true;
                            }

                            ui.add(
                                egui::TextEdit::singleline(&mut self.popups.link_config.arguments[index])
                                .hint_text("e.g. --name=John")
                            );
                            if ui.button("➖").clicked() {
                                index_should_remove = Some(index);
                            }

                            drag_response
                        }).inner;

                        // 检查是否有拖拽悬停在当前项目上
                        if let (Some(pointer), Some(_)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            response.dnd_hover_payload::<ArgumentIndex>(),
                        ) {
                            // 获取当前项目的矩形区域
                            let rect = response.rect;

                            // 创建线条样式
                            let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 255));

                            // 根据鼠标位置确定插入位置（上方或下方）
                            if pointer.y < rect.center().y {
                                // 在上方绘制水平线
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                            } else {
                                // 在下方绘制水平线
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                            }

                            // 检查是否释放了拖拽
                            if let Some(dragged_index) = response.dnd_release_payload::<ArgumentIndex>() {
                                // 记录拖拽源和目标
                                drag_from = Some(dragged_index.0);

                                // 根据鼠标位置确定是插入到上方还是下方
                                let target_index = if pointer.y < rect.center().y {
                                    index
                                } else {
                                    index + 1
                                };

                                drag_to = Some(target_index);
                            }
                        }
                    }

                    // 处理拖拽重排
                    if let (Some(from_idx), Some(to_idx)) = (drag_from, drag_to) {
                        if from_idx != to_idx {
                            // 先移除源项目
                            let argument = self.popups.link_config.arguments.remove(from_idx);

                            // 调整目标索引（如果源在目标之前）
                            let adjusted_to_idx = if from_idx < to_idx {
                                to_idx - 1
                            } else {
                                to_idx
                            };

                            // 插入到目标位置
                            self.popups.link_config.arguments.insert(adjusted_to_idx, argument);
                        }
                    }

                    if let Some(index) = index_should_remove {
                        self.popups.link_config.arguments.remove(index);
                    }

                    if self.popups.link_config.args_scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.popups.link_config.args_scroll_to_bottom = false;
                    }
                });

                ui.horizontal(|ui| {
                    if self.popups.link_config.arguments.is_empty() {
                        ui.label(egui::RichText::new(
                            "这个快捷方式还没有任何参数"
                        ).weak());
                    } else if has_empty_argument {
                        ui.label(egui::RichText::new(
                            "⚠ 你似乎有一些空参数，如果是刻意为之，请无视此警告"
                        ).color(egui::Color32::LIGHT_RED));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("➕").clicked() {
                            self.popups.link_config.arguments.push("".to_string());
                            self.popups.link_config.args_scroll_to_bottom = true;
                        }
                    });
                });
            });

            ui.label(
                egui::RichText::new("tip: 名称可以使用 / 来创建别名，也可以只输入一个名称。右键路径输入框可以打开路径选择器")
                    .weak()
            );

            let advanced_config_window = egui::Window::new("高级选项")
            .collapsible(false)
            .resizable(false)
            .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
            .open(&mut self.popups.link_config.show_advanced_config)
            .max_width(256.)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                .max_height(256.)
                .show(ui, |ui| {


                ui.checkbox(&mut self.popups.link_config.is_admin, {
                    "以管理员权限运行"
                });

                if self.popups.link_config.is_admin {
                    ui.label(egui::RichText::new(
                        "⚠ 权限的提升可能是危险的，请确保你信任这个程序"
                    ).color(egui::Color32::LIGHT_RED));
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("工作目录");
                    ui.add(egui::TextEdit::singleline(&mut self.popups.link_config.working_directory)
                    .hint_text(
                        Path::new(&self.popups.link_config.file_path)
                        .parent()
                        .unwrap_or(Path::new("默认为程序所在目录"))
                        .to_str()
                        .unwrap_or("默认为程序所在目录")
                    ))
                    .context_menu(|ui| {
                        if ui.button("选择一个文件夹").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("任意文件", &["*"])
                                .pick_folder() {
                                    self.popups.link_config.working_directory = path.display().to_string();
                                }
                            ui.close_menu();
                        }
                    });
                });

                if !Path::new(&self.popups.link_config.working_directory).exists() && !self.popups.link_config.working_directory.is_empty() {
                    ui.label(egui::RichText::new(
                        "⚠ 此工作目录无效或者我无法访问它"
                    ).color(egui::Color32::LIGHT_RED));
                }

                ui.label(
                    egui::RichText::new("tip: 右键路径输入框可以打开路径选择器")
                        .weak()
                );

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("配置命令参数");
                    let arg_button = ui.button(
                        if self.popups.link_config.arguments.is_empty() {
                            "没有参数".to_string()
                        } else {
                            format!("{} 个参数", self.popups.link_config.arguments.len())
                        } + " ⚙");
                    if arg_button.clicked() {
                        self.popups.link_config.show_args_config = true;
                        if let Some(window) = arguments_config_window {
                            window.response.request_focus();
                        }
                    }
                });

            })});


            ui.horizontal(|ui| {
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.button("高级选项 ⚙").clicked() {
                        debug!("打开高级选项");
                        self.popups.link_config.show_advanced_config = true;
                        if let Some(window) = advanced_config_window {
                            window.response.request_focus();
                        }
                    }
                });
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
                            let new_link = ProgramLink::new(
                                self.popups.link_config.name.clone().split("/").map(|s| s.to_string()).collect(),
                                self.popups.link_config.icon_path.clone().unwrap_or("".to_string()),
                                self.popups.link_config.file_path.clone(),
                                self.popups.link_config.arguments.clone(),
                                self.popups.link_config.tags.clone().into_iter().collect(),
                                self.popups.link_config.is_admin,
                            );
                            self.texture_mgr.register_usage(&new_link.icon_path, &new_link.uuid);
                            self.program_links.push(new_link);

                            should_save = true;
                            should_close = true;
                        }
                    });

                } else {
                    if ui.button("保存").clicked() {
                        let current_link = &mut self.program_links[self.popups.link_config.index_of_the_link];
                        let old_icon_path = current_link.icon_path.clone();
                        let uuid = current_link.uuid.clone();

                        self.texture_mgr.release_usage(&old_icon_path, &uuid);

                        current_link.name = self.popups.link_config.name.clone().split("/").map(|s| s.to_string()).collect();
                        current_link.icon_path = self.popups.link_config.icon_path.clone().unwrap_or("".to_string());
                        current_link.run_command = self.popups.link_config.file_path.clone();
                        current_link.working_directory = self.popups.link_config.working_directory.clone();
                        current_link.arguments = self.popups.link_config.arguments.clone();
                        current_link.tags = self.popups.link_config.tags.clone().into_iter().collect();
                        current_link.is_admin = self.popups.link_config.is_admin;

                        self.texture_mgr.register_usage(&current_link.icon_path, &current_link.uuid);

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
            debug!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;
            self.popups.link_config.show_args_config = false;
            self.popups.link_config.show_advanced_config = false;
            
            if let Some(icon_path) = self.popups.link_config.icon_path.clone() {
                if !should_save {
                    self.texture_mgr.schedule_forget(icon_path);
                }
            }

            if should_save {
                self.save_conf();
            }
        }
    }
}


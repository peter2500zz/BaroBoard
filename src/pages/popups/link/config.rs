use egui;
use std::collections::HashSet;
use rfd;

use crate::my_structs::*;

/// 表示参数在列表中的索引位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArgumentIndex(usize);

pub struct LinkConfig {
    is_new_link: bool,
    
    index_of_the_link: usize,

    // 临时变量们
    pub name: String,
    pub icon_path: Option<String>,
    pub run_command: String,
    pub arguments: Vec<String>,
    pub tags: HashSet<String>,

    // 子窗口配置
    show_args_config: bool,
}

impl LinkConfig {
    pub fn new() -> Self {
        Self {
            is_new_link: false,
            index_of_the_link: 0,
            name: "".to_string(),
            icon_path: None,
            run_command: "".to_string(),
            arguments: Vec::new(),
            tags: HashSet::new(),

            show_args_config: false,
        }
    }


    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.is_new_link = false;
        self.index_of_the_link = position.link_index;

        self.name = link.name.clone().join("/");
        self.icon_path = Some(link.icon_path.clone());
        self.run_command = link.run_command.clone();
        self.arguments = link.arguments.clone();
        self.tags = HashSet::from_iter(link.tags.clone());
    }

    
    pub fn config_new_link(&mut self) {
        self.is_new_link = true;

        self.name = "".to_string();
        self.icon_path = None;
        self.run_command = "".to_string();
        self.arguments = Vec::new();
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

            ui.horizontal(|ui| {
                ui.label("参数");
                let arg_button = ui.button(
                    if self.popups.link_config.arguments.is_empty() {
                        "没有参数".to_string()
                    } else {
                        format!("{} 个参数", self.popups.link_config.arguments.len())
                    } + " ⚙");
                if arg_button.clicked() {
                    self.popups.link_config.show_args_config = true;
                }
            });

            egui::Window::new("参数配置")
            .collapsible(false)
            .resizable(false)
            .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
            .open(&mut self.popups.link_config.show_args_config)
            .show(ui.ctx(), |ui| {
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
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("➕").clicked() {
                            self.popups.link_config.arguments.push("".to_string());
                        }
                    });
                });
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
                                    self.popups.link_config.arguments.clone(),
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
                        current_link.arguments = self.popups.link_config.arguments.clone();
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
            println!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;
            self.popups.link_config.show_args_config = false;
            
            if let Some(icon_path) = self.popups.link_config.icon_path.clone() {
                if !should_save {
                    self.icon_will_clean.push(icon_path);
                }
            }
            
            if should_save {
                self.save_conf();
            }
        }
    }
}


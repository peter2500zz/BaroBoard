
use egui;
use log::debug;

use crate::my_structs::*;

/// 表示程序链接在列表中的索引位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProgramLinkIndex(usize);

impl MyApp {
    pub(super) fn show_links(&mut self, ui: &mut egui::Ui) {
        // 显示页面
        let mut should_save = false;

        // 记录拖拽源和目标位置
        let mut drag_from = None;
        let mut drag_to = None;

        // 如果搜索框里有内容，则使用排序后的程序列表，否则使用页面中的程序列表
        let display_program_links = if self.search_text.is_empty() {
            if let Some(tag) = self.current_tag.clone() {sort_by_tag(self.program_links.clone(), tag)} else {self.program_links.clone()}
        } else {
            self.sorted_program_links.clone()
        };

        let chunks: Vec<_> = display_program_links.chunks(6).collect();

        if chunks.is_empty() && !self.edit_mode {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(
                        if self.search_text.is_empty() {"这里还没有任何快捷方式！你可以随便拖点东西进来，或者在编辑模式中创建一个"
                        } else {
                            "没有找到任何快捷方式"
                        })
                        .weak()
                        .size(16.)
                );
            });
        }

        // 新建链接的按钮
        let mut show_on_next_line = true;

        // 遍历每个chunk显示
        for (i, chunk) in chunks.iter().enumerate() {
            ui.horizontal(|ui| {
                for (link_index, program) in (*chunk).iter().enumerate() {
                    // 计算当前项目在整个列表中的绝对索引
                    let absolute_index = i * 6 + link_index;

                    // 图标与名称
                    ui.vertical(|ui| {
                        // 注册对icon_path的缓存
                        self.texture_mgr.register_usage(&program.icon_path, &program.uuid);

                        let btn = Box::new(|ui: &mut egui::Ui| {
                            if !self.edit_mode {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                            };

                            let hover_text = format!("{} {}", program.run_command, program.arguments.join(" "));

                            let image_response = ui.add_sized(
                                egui::vec2(96.0, 96.0),
                                egui::ImageButton::new(format!("file://{}", &program.icon_path))
                            );

                            if hover_text.trim().is_empty() {
                                image_response.on_hover_text_at_pointer("这个快捷方式还没有运行命令")
                            } else {
                                image_response.on_hover_text_at_pointer(&hover_text)
                            }
                        });

                        let enable_drag = self.edit_mode && !self.popups.called;

                        let response = if enable_drag {
                            // 在编辑模式下启用拖拽
                            ui.dnd_drag_source(egui::Id::new(&program.uuid), ProgramLinkIndex(absolute_index), |ui| {
                                // 绘制图标按钮
                                btn(ui)
                            }).response
                        } else {
                            btn(ui)
                        };

                        // 检查是否有拖拽悬停在当前项目上
                        if enable_drag {
                            if let (Some(pointer), Some(_)) = (
                                ui.input(|i| i.pointer.interact_pos()),
                                response.dnd_hover_payload::<ProgramLinkIndex>(),
                            ) {
                                // 获取当前项目的矩形区域，用于绘制视觉提示
                                let rect = response.rect;

                                // 创建线条样式
                                let stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);

                                // 根据鼠标位置确定插入位置
                                if pointer.x < rect.center().x {
                                    // 在左侧绘制垂直线
                                    ui.painter().vline(rect.left(), rect.y_range(), stroke);
                                } else {
                                    // 在右侧绘制垂直线
                                    ui.painter().vline(rect.right(), rect.y_range(), stroke);
                                }

                                // 检查是否释放了拖拽
                                if let Some(dragged_index) = response.dnd_release_payload::<ProgramLinkIndex>() {
                                    // 记录拖拽源和目标
                                    drag_from = Some(dragged_index.0);

                                    // 根据鼠标位置确定是插入到左侧还是右侧
                                    let target_index = if pointer.x < rect.center().x {
                                        absolute_index
                                    } else {
                                        absolute_index + 1
                                    };

                                    drag_to = Some(target_index);

                                    debug!("由于拖拽 尝试保存");
                                    should_save = true;
                                }
                            }
                        }

                        if !self.popups.called {
                            if self.edit_mode && response.clicked() {
                                // 打开设置窗口
                                self.popups.config_existing_link(LinkPosition::new(absolute_index), program);

                            } else {
                                if response.clicked() {
                                    self.run_program(program.clone());

                                    if !self.search_text.is_empty() {
                                        self.hide_window();
                                    }
                                }

                                // 右键点击图标，显示上下文菜单
                                response.context_menu(|ui| {
                                    // 显示名称
                                    ui.horizontal(|ui| {
                                        ui.label(if program.name.is_empty() {
                                            egui::RichText::new("未命名").weak()
                                        } else {
                                            egui::RichText::new(&program.name.get(0).unwrap_or(&"".to_string()).to_owned())
                                        });
                                    });

                                    ui.separator();

                                    if ui.button("运行")
                                    .clicked() {
                                        self.run_program(program.clone());

                                        ui.close_menu();
                                    }
                                    if ui.button("编辑").clicked() {
                                        self.popups.config_existing_link(LinkPosition::new(absolute_index), program);
                                        ui.close_menu();
                                    }

                                    if ui.button("删除")
                                    .clicked() {
                                        self.popups.delete_link(LinkPosition::new(absolute_index));
                                        // self.delete_link(link_index);

                                        ui.close_menu();
                                    }
                                });
                            }
                        };

                        // 快捷方式名称Label，最大宽度为96px，仅限一行
                        ui.allocate_ui(egui::Vec2 { x: 96.0, y: 96.0 }, |ui| {
                            let mut job = egui::text::LayoutJob::single_section(program.name.get(0).unwrap_or(&"".to_string()).to_owned(), 
                                egui::TextFormat {
                                ..Default::default()
                            });
                            job.wrap = egui::text::TextWrapping {
                                max_rows: 1,
                                break_anywhere: true,
                                overflow_character: Some('…'),
                                ..Default::default()
                            };

                            ui.label(job);
                        });
                    });
                };

                // 只有在不是最后一个chunk时才添加间隔
                if i == chunks.len() - 1 && chunk.len() < 6 && self.edit_mode {
                    show_on_next_line = false;
                    ui.vertical(|ui| {
                        let response = ui.add_sized(
                            egui::vec2(96.0, 96.0),
                            egui::Button::new(egui::RichText::new("➕").size(48.))
                        );
                        if response.clicked() && !self.popups.called  {
                            self.popups.config_new_link();
                        }

                    });
                }
            });
            // if i != chunks.len() - 1 {
            // } 
            if i != chunks.len() - 1 || (show_on_next_line && self.edit_mode) {
                ui.horizontal(|ui| {
                    ui.label("");
                });
            }
            if i == chunks.len() - 1 && show_on_next_line && self.edit_mode {
                ui.vertical(|ui| {
                    let response = ui.add_sized(
                        egui::vec2(96.0, 96.0),
                        egui::Button::new(egui::RichText::new("➕").size(48.))
                    );
                    if response.clicked() && !self.popups.called  {
                        self.popups.config_new_link();
                    }
                });
            }
        };

        if chunks.is_empty() && self.edit_mode {
            ui.vertical(|ui| {
                let response = ui.add_sized(
                    egui::vec2(96.0, 96.0),
                    egui::Button::new(egui::RichText::new("➕").size(48.))
                );
                if response.clicked() && !self.popups.called  {
                    self.popups.config_new_link();
                }
            });
        }

        // 处理拖拽重排
        if let (Some(from_idx), Some(to_idx)) = (drag_from, drag_to) {
            if from_idx != to_idx && self.search_text.is_empty() {
                // 获取对当前页面的可变引用
                // 先移除源项目
                let program = self.program_links.remove(from_idx);

                // 调整目标索引（如果源在目标之前）
                let adjusted_to_idx = if from_idx < to_idx {
                    to_idx - 1
                } else {
                    to_idx
                };

                // 插入到目标位置
                self.program_links.insert(adjusted_to_idx, program);

            }
        }
        // ctx.texture_ui(ui);

        if should_save {
            self.save_conf();
        }
    }
}

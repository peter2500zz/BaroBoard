pub mod popups;
mod sidebar;

use egui;
use log::{debug, info};
use std::collections::HashSet;
use strsim::jaro_winkler;
use pinyin::ToPinyin;

use crate::my_structs::*;

/// 表示程序链接在列表中的索引位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProgramLinkIndex(usize);

impl MyApp {
    pub fn main_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui)  {        
        // 添加面板的顺序非常重要，影响最终的布局
        egui::TopBottomPanel::top("title")
        .resizable(false)
        .min_height(32.0)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            ui.heading(egui::RichText::new(&self.title))
                            .context_menu(|ui| {

                                if ui.button("自身信息").clicked() {
                                    info!("{:?}", self);
                                }

                                if ui.button("所有快捷方式").clicked() {
                                    info!("{:?}", self.program_links);
                                }
                                if ui.button("已缓存的图片").clicked() {
                                    info!("{:?}", self.cached_icon);
                                }

                                if ui.button("隐藏").clicked() {
                                    self.hide_window();
                                }

                                if ui.button("获取图标").clicked() {
                                    crate::utils::windows_utils::get_icon_from_exe("C:\\Windows\\System32\\notepad.exe").unwrap();
                                }
                            });
                            
                        });
                        if self.wont_save {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.heading(egui::RichText::new("自动保存已禁用").color(egui::Color32::LIGHT_RED))
                                .on_hover_ui(|ui| {
                                    ui.heading("为什么无法自动保存？");
                                    ui.label("当无法正常读取配置文件时，程序会关闭自动保存功能，以防破坏原本的配置文件");
                                    ui.label("你可以在设置中手动保存");
                                    ui.label(egui::RichText::new(
                                        "注意: 不推荐在自动保存禁用的情况下手动保存，这会丢失原先的配置文件。如果可能，请先尝试修复配置文件"
                                    ).color(egui::Color32::LIGHT_RED));
                                })
                                ;
                            });
                        }
                    })
                });

                ui.vertical_centered(|ui: &mut egui::Ui| {
                    // 搜索框占据中间位置
                    let search_text = ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                    // 如果程序被唤起，则请求焦点
                    let mut called_guard = self.called.lock().unwrap();
                    if *called_guard {  // 被呼叫了！
                        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Focus);
                        // self.edit_mode = false;
                        self.search_text = "".to_string();
                        search_text.request_focus();
                        *called_guard = false;
                    }

                    // 如果搜索框里有内容，则进行搜索
                    if !self.search_text.is_empty() {
                        // 计算相似度
                        let mut results: Vec<(ProgramLink, f64)> = if let Some(tag) = self.current_tag.clone() {sort_by_tag(self.program_links.clone(), tag)} else {self.program_links.clone()}
                            .iter()
                            .map(|program_link| {
                                let max_score = program_link.name.iter().map(|name| {
                                    // 计算多种情况下的相似度得分
                                    let original_score = jaro_winkler(&self.search_text, &name);
                                    let lower_score = jaro_winkler(&self.search_text, &name.to_lowercase());

                                    let pinyin_score = jaro_winkler(
                                        &self.search_text, 
                                        &name.chars().map(|c| {
                                            c.to_pinyin()
                                            .map(|p| p.plain().to_string())
                                            .unwrap_or_else(|| c.to_string())
                                    }).collect::<String>());

                                    // 取最高分
                                    original_score
                                        .max(lower_score)
                                        .max(pinyin_score)
                                })
                                .collect::<Vec<f64>>()
                                .iter()
                                .cloned()
                                .fold(0., f64::max);

                                (program_link.clone(), max_score)
                            })
                            // 设置相似度阈值
                            .filter(|(_, score)| *score > 0.5)
                            .collect();
                        
                        // 按相似度降序排列
                        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                        if !results.is_empty() {
                            // 更新排序后的程序列表
                            self.sorted_program_links = results.iter().map(|(program, _)| program.clone()).collect();
                            // 如果按下回车键，则运行选中的程序
                            // if search_text.has_focus() {
                            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && 
                                // 这一步的作用是，如果用户使用Tab聚焦到按钮时，不会触发搜索框的lost_focus，避免重复触发
                                search_text.lost_focus()
                            {
                                
                                info!("选中: {} 权重: {}", self.sorted_program_links[0].name.get(0).unwrap_or(&"".to_string()), results[0].1);
                                self.run_program(self.sorted_program_links[0].clone());
                                self.search_text = "".to_string();

                                self.hide_window();
                            }
                            // }
                            
                        } else {
                            // 如果搜索框里没有内容，则清空排序后的程序列表
                            self.sorted_program_links.clear();
                        }
                    }
                });
            });
        });

        egui::SidePanel::left("side_bar")
        .resizable(false)
        .default_width(150.0)
        // .width_range(80.0..=200.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("标签");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.side_bar(ui);

                // 版本号水印
                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                    ui.label(egui::RichText::new(crate::PROGRAM_VERSION).weak());
                });
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                // ui.heading("页面内容");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.edit_mode, "编辑模式");
                });
            });
            // });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_popup(ui);
                // self.show_delete_link(ui);
                // self.show_setting_window(ui);
                // self.show_config_save_error(ui);
                self.show_page(ui);
            });
        });
    }


    fn show_page(&mut self, ui: &mut egui::Ui) {
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
                        if self.search_text.is_empty() {"这里还没有任何快捷方式！不过你可以在编辑模式中创建一个"
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
                        // 注册对icon_path的缓存 - 使用entry API优化
                        self.cached_icon
                            .entry(program.icon_path.clone())
                            .or_insert_with(HashSet::new)
                            .insert(program.uuid.clone());
                        
                        let btn = Box::new(|ui: &mut egui::Ui| {
                            if self.edit_mode {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                            };

                            ui.add_sized(
                                egui::vec2(96.0, 96.0),
                                egui::ImageButton::new(format!("file://{}", &program.icon_path))
                            )
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

pub mod popups;
mod sidebar;

use egui;
use std::process::Command;
use std::collections::HashSet;
use strsim::jaro_winkler;
use pinyin::ToPinyin;

use crate::my_structs::*;

impl MyApp {
    pub fn main_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui)  {        
        // 添加面板的顺序非常重要，影响最终的布局
        egui::TopBottomPanel::top("title")
        .resizable(false)
        .min_height(32.0)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(&self.title)
                    .context_menu(|ui| {
                        if ui.button("添加一个子页面").clicked() {
                            self.pages.push(
                                Page::new(
                                    "新页面".to_string(),
                                    vec![]
                                )
                            );
                        }
                        if ui.button("所有快捷方式").clicked() {
                            println!("{:?}", self.pages);
                        }
                        if ui.button("已缓存的图片").clicked() {
                            println!("{:?}", self.cached_icon);
                        }

                        if ui.button("隐藏").clicked() {
                            self.hide_window();
                        }
                    });
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
                        let mut results: Vec<(ProgramLink, f64)> = self.pages[self.current_page_index].program_links
                            .iter()
                            .map(|program_link| {
                                // 计算多种情况下的相似度得分
                                let original_score = jaro_winkler(&self.search_text, &program_link.name);
                                
                                let pinyin_score = jaro_winkler(
                                    &self.search_text, 
                                    &program_link.name.chars().map(|c| {
                                        c.to_pinyin()
                                        .map(|p| p.plain().to_string())
                                        .unwrap_or_else(|| c.to_string())
                                }).collect::<String>());
                                
                                // 取最高分
                                let max_score = original_score
                                    .max(pinyin_score);

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
                                println!("选中的程序: {} 权重: {}", results[0].0.name, results[0].1);
                                match Command::new(&results[0].0.run_command).spawn() {
                                    Ok(_) => println!("{} 运行成功", results[0].0.name),
                                    Err(e) => {
                                        println!("{} 运行失败: {}", results[0].0.name, e);
                                    },
                                }
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
                ui.heading("分类");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.side_bar(ui);
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
                self.show_delete_link(ui);
                self.show_setting_window(ui);
                self.show_config_save_error(ui);
                self.show_page(ui);
            });
        });
    }


    fn show_page(&mut self, ui: &mut egui::Ui) {
        // 显示页面
        if let Some(page) = self.pages.get(self.current_page_index) {
            
            // 如果搜索框里有内容，则使用排序后的程序列表，否则使用页面中的程序列表
            let chunks: Vec<_> = (if self.search_text.is_empty() {
                &page.program_links
            } else {
                &self.sorted_program_links
            })
            // 每次选取6个程序，并显示在同一行
            .chunks(6).collect();

            if chunks.is_empty() && !self.edit_mode {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(
                            if self.search_text.is_empty() {"
                                这个页面中还没有任何快捷方式，你可以在编辑模式中创建一个"
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
                        // 图标与名称
                        ui.vertical(|ui| {
                            // 注册对icon_path的缓存 - 使用entry API优化
                            self.cached_icon
                                .entry(program.icon_path.clone())
                                .or_insert_with(HashSet::new)
                                .insert(program.uuid.clone());
                            
                            let btn = Box::new(|ui: &mut egui::Ui| {
                                ui.add_sized(
                                    egui::vec2(96.0, 96.0),
                                    egui::ImageButton::new(format!("file://{}", &program.icon_path))
                                )
                            });

                            let response = if self.edit_mode {
                                ui.dnd_drag_source(egui::Id::new(&program.uuid), (), |ui| {
                                    btn(ui)
                                }).response
                            } else {
                                btn(ui)
                            };
                            
                            if !self.link_popups.link_config.called {
                                if self.edit_mode && response.clicked() {
                                    // 打开设置窗口
                                    self.link_popups.link_config.config_existing_link(LinkPosition::new(self.current_page_index, link_index), program);

                                } else {
                                    if response.clicked() {
                                        match Command::new(&program.run_command).spawn() {
                                            Ok(_) => println!("{} 运行成功", program.name),
                                            Err(e) => {
                                                println!("{} 运行失败: {}", program.name, e);
                                                
                                            },
                                        }
                                    }
                                    
                                    // 右键点击图标，显示上下文菜单
                                    response.context_menu(|ui| {
                                        // 显示名称
                                        ui.horizontal(|ui| {
                                            ui.label(if program.name.is_empty() {
                                                egui::RichText::new("未命名").weak()
                                            } else {
                                                egui::RichText::new(&program.name)
                                            });
                                        });

                                        ui.separator();
            
                                        if ui.button("运行")
                                        .clicked() {
                                            match Command::new(&program.run_command).spawn() {
                                                Ok(_) => println!("{} 运行成功", program.name),
                                                Err(e) => {
                                                    println!("{} 运行失败: {}", program.name, e);
                                                    
                                                },
                                            }

                                            ui.close_menu();
                                        }
                                        if ui.button("编辑").clicked() {
                                            self.link_popups.link_config.config_existing_link(LinkPosition::new(self.current_page_index, link_index), program);
                                            ui.close_menu();
                                        }
                                        
                                        if ui.button("删除")
                                        .clicked() {
                                            
                                            self.link_popups.link_delete.delete_link(self.current_page_index, link_index);
                                            
                                            ui.close_menu();
                                        }
                                    });
                                }
                            };
                            
                            // 快捷方式名称Label，最大宽度为96px，仅限一行
                            ui.allocate_ui(egui::Vec2 { x: 96.0, y: 96.0 }, |ui| {
                                let mut job = egui::text::LayoutJob::single_section(program.name.to_owned(), 
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
                            if response.clicked() && !self.link_popups.link_config.called  {
                                self.link_popups.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
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
                        if response.clicked() && !self.link_popups.link_config.called  {
                            self.link_popups.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
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
                    if response.clicked() && !self.link_popups.link_config.called  {
                        self.link_popups.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
                    }
                });
            }
        }
        // ctx.texture_ui(ui);
    }
}

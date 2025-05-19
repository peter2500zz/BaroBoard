use eframe::egui;
use std::process::Command;
use std::collections::HashSet;
use strsim::jaro_winkler;

use crate::my_structs::*;


impl MyApp {
    pub fn main_ui(&mut self, ui: &mut egui::Ui)  {        
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
                    });
                });

                ui.vertical_centered(|ui: &mut egui::Ui| {
                    // 搜索框占据中间位置
                    let search_text = ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                    
                    if self.called {
                        search_text.request_focus();
                        self.called = false;
                    }
                    
                    if self.search_text != "" {
                        search_text.show_tooltip_ui(|ui| {
                            let mut results: Vec<(ProgramLink, f64)> = self.pages[self.current_page_index].program_links
                                .iter()
                                .map(|name| {
                                    // TODO! 应当处理大小写 空格 
                                    (name.clone(), jaro_winkler(&self.search_text, &name.name))
                                })
                                .filter(|(_, score)| *score > 0.5) // 设置相似度阈值
                                .collect();
                            
                            // 按相似度降序排列
                            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                            
                            if !results.is_empty() {
                                for (program, score) in results {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(false, &program.name).clicked() {
                                            println!("选中的程序: {} 权重: {}", program.name, score);
                                            match Command::new(&program.run_command).spawn() {
                                                Ok(_) => println!("运行成功"),
                                                Err(e) => {
                                                    println!("运行失败: {}", e);
                                                },
                                            }
                                            self.search_text = "".to_string();
                                        }
                                    });
                                }
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("没有找到相关程序").weak());
                                });
                            }
                        });
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
                ui.heading("左导航栏");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.side_bar(ui);
            });
        });

        // egui::SidePanel::right("right_panel")
        // .resizable(true)
        // .default_width(150.0)
        // .width_range(80.0..=200.0)
        // .show_inside(ui, |ui| {
        //     ui.vertical_centered(|ui| {
        //         ui.heading("右导航栏");
        //     });
        //     egui::ScrollArea::vertical().show(ui, |ui| {
        //         ui.label("右导航栏内容");
        //     });
        // });

        // egui::TopBottomPanel::bottom("bottom_panel")
        // .resizable(false)
        // .min_height(0.0)
        // .show_inside(ui, |ui| {
        //     ui.vertical_centered(|ui| {
        //         ui.heading("状态栏");
        //     });
        //     ui.vertical_centered(|ui| {
        //         ui.label("状态栏内容");
        //     });
        // });

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
                self.show_setting_window(ui);
                self.show_config_save_error(ui);
                self.show_delete_link(ui);
                self.show_page(ui);
            });
        });
    }


    fn show_page(&mut self, ui: &mut egui::Ui) {

        // 显示页面
        if let Some(page) = self.pages.get(self.current_page_index) {
            // 每次选取6个程序，并显示在同一行
            let chunks: Vec<_> = page.program_links.chunks(6).collect();
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
                            
                            let response = ui.add_sized(
                                egui::vec2(96.0, 96.0),
                                egui::ImageButton::new(format!("file://{}", &program.icon_path))
                            )
                            // .on_hover_ui_at_pointer(|ui| {
                            //     ui.label(&program.name);
                            // })
                            ;
                            
                            if !self.link_config.called {
                                if self.edit_mode && response.clicked() {
                                    // 打开设置窗口
                                    self.link_config.config_existing_link(LinkPosition::new(self.current_page_index, link_index), program);

                                } else {
                                    if response.clicked() {
                                        match Command::new(&program.run_command).spawn() {
                                            Ok(_) => println!("运行成功"),
                                            Err(e) => {
                                                println!("运行失败: {}", e);
                                                
                                            },
                                        }
                                    }
                                    
                                    response.context_menu(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(&program.name);
                                        });
            
                                        if ui.button("运行")
                                        .clicked() {
                                            println!("运行");

                                            match Command::new(&program.run_command).spawn() {
                                                Ok(_) => println!("运行成功"),
                                                Err(e) => {
                                                    println!("运行失败: {}", e);
                                                    
                                                },
                                            }

                                            ui.close_menu();
                                        }
                                        if ui.button("编辑").clicked() {
                                            self.link_config.config_existing_link(LinkPosition::new(self.current_page_index, link_index), program);
                                            ui.close_menu();
                                        }
                                        
                                        if ui.button("删除")
                                        .clicked() {
                                            
                                            println!("删除");
                                            
                                            self.config_save.delete_link(self.current_page_index, link_index);
                                            
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
                            if response.clicked() && !self.link_config.called  {
                                println!("点击了添加按钮");

                                self.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
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
                        if response.clicked() && !self.link_config.called  {
                            println!("点击了添加按钮");

                            self.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
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
                    if response.clicked() && !self.link_config.called  {
                        println!("点击了添加按钮");

                        self.link_config.config_new_link(LinkPosition::new(self.current_page_index, 0));
                    }
                });
            }
        }
    }
}

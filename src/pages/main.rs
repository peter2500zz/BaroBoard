use eframe::egui::{self, Align, Layout};
use std::process::Command;
use std::collections::HashSet;
use rfd;
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
                        if ui.button("在当前页面添加一个程序").clicked() {
                            self.pages[self.current_page_index].program_links.push(
                                ProgramLink::new(
                                    "Bilibili".to_string(),
                                    "src/assets/images/Impulse_Command_Block.gif".to_string(),
                                    r"https://www.bilibili.com/".to_string(),
                                )
                            );
                        }
                    });
                });

                ui.vertical_centered(|ui: &mut egui::Ui| {
                    let search_text = ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                    if self.called {
                        search_text.request_focus();
                        self.called = false;
                    }
                    
                    if self.search_text != "" {
                        search_text.show_tooltip_ui(|ui| {
                            let mut results: Vec<(ProgramLink, f64)> = self.pages[self.current_page_index].program_links
                                .iter()
                                .map(|name| (name.clone(), jaro_winkler(&self.search_text, &name.name)))
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

                    // if search_text.lost_focus() {
                    //     self.search_text = "".to_string();
                    // }
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
            ui.vertical_centered(|ui| {
                ui.heading("页面内容");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_page(ui);
            });
        });
    }

    fn show_page(&mut self, ui: &mut egui::Ui) {
        // 配置文件弹窗
        if let Some((title, message)) = self.conf_error.clone() {
            egui::Window::new(title)
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(message);
                    ui.separator();
                    
                    ui.with_layout(Layout {
                        cross_align: Align::RIGHT,
                        ..Default::default()
                    }, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("好的").clicked() {
                                self.conf_error = None;
                            };
                            if ui.button("重试").clicked() {
                                match self.save_conf() {
                                    Ok(_) => println!("保存成功"),
                                    Err(e) => {
                                        println!("保存失败: {}", e);
                                        self.conf_error = Some((
                                            "无法写入配置文件！".to_string(),
                                            "你可以尝试删除配置文件并尝试再次保存".to_string()
                                        ));
                                    },
                                };
                            }
                        });
                    });
                });
            });
        }

        // 删除快捷方式弹窗
        if let Some((page_index, link_index)) = self.link_should_delete {
            self.setting_open = false;

            egui::Window::new("你确定要删除这个快捷方式吗？")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("你确定要删除这个快捷方式吗？");
                    ui.label(format!("“{}”将会永久消失！（真的很久！）", self.pages[page_index].program_links[link_index].name));
                    ui.separator();
                    
                    ui.with_layout(Layout {
                        cross_align: Align::RIGHT,
                        ..Default::default()
                    }, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                            .clicked() {
                                let program_links = &mut self.pages[page_index].program_links;
                                self.cached_icon
                                    .entry(program_links[link_index].icon_path.clone())
                                    .or_insert_with(HashSet::new)
                                    .remove(&program_links[link_index].uuid);
    
                                self.icon_will_clean.push(program_links[link_index].icon_path.clone());
                                program_links.remove(link_index);
                                println!("删除成功: {:?}", program_links);
                                match self.save_conf() {
                                    Ok(_) => println!("保存成功"),
                                    Err(e) => {
                                        println!("保存失败: {}", e);
                                        self.conf_error = Some((
                                            "无法写入配置文件！".to_string(),
                                            "你可以尝试删除配置文件并尝试再次保存".to_string()
                                        ));
                                    },
                                };
                                self.link_should_delete = None;
                            }
                            if ui.button("取消").clicked() {
                                self.link_should_delete = None;
                            }
                        });
                    });
                });
            });
        }

        // 设置快捷方式弹窗
        if self.setting_open {
            // 设置页面
            egui::Window::new("配置快捷方式")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            // .open(&mut self.setting_open)

            .show(ui.ctx(), |ui| {
                if ui.add_sized(
                    egui::vec2(96.0, 96.0),
                    egui::ImageButton::new(format!("file://{}", &self.temp_icon_path))
                ).clicked() {
                    println!("点击了图片");
                    if let Some(path) = rfd::FileDialog::new()
                    .add_filter("text", &["png", "svg", "gif"])
                    .pick_file() {
                        // println!("{}", path.display());
                        self.icon_will_clean.push(self.temp_icon_path.clone());
                        self.temp_icon_path = path.display().to_string();
                    }
                }
                
                ui.label(&self.temp_icon_path);

                ui.horizontal(|ui| {
                    ui.label("名称");
                    ui.add(egui::TextEdit::singleline(&mut self.temp_name).hint_text("e.g. 记事本"));
                    
                });

                ui.horizontal(|ui| {
                    ui.label("命令");
                    ui.add(egui::TextEdit::singleline(&mut self.temp_run_command).hint_text("e.g. C:\\Windows\\System32\\notepad.exe"));
                });

                ui.separator();
                // 保存与取消按钮
                ui.with_layout(Layout {
                    cross_align: Align::RIGHT,
                    ..Default::default()
                }, |ui| {ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        println!("保存");
                        let current_link = &mut self.pages[self.current_setting_page].program_links[self.current_setting_link];
                        // 尝试移除之前的缓存标记
                        if let Some(icon_path) = self.cached_icon.get_mut(&current_link.icon_path) {
                            icon_path.remove(&current_link.uuid);
                        } else {
                            // 如果不行则强制清空
                            self.cached_icon.insert(current_link.icon_path.clone(), HashSet::new());
                        }

                        // 如果缓存为空，则删除缓存
                        self.icon_will_clean.push(current_link.icon_path.clone());

                        current_link.name = self.temp_name.clone();
                        current_link.icon_path = self.temp_icon_path.clone();
                        current_link.run_command = self.temp_run_command.clone();

                        match self.save_conf() {
                            Ok(_) => println!("保存成功"),
                            Err(e) => {
                                println!("保存失败: {}", e);
                                self.conf_error = Some((
                                    "无法写入配置文件！".to_string(),
                                    "你可以尝试删除配置文件并尝试再次保存".to_string()
                                ));
                            },
                        };
                        
                        self.setting_open = false;
                    }
                    if ui.button("取消").clicked() {
                        println!("取消");
                        // 如果此图片没有被其他程序使用，则删除缓存
                        self.icon_will_clean.push(self.temp_icon_path.clone());
                        self.setting_open = false;
                    }
                })});
            });
        };

        // 显示页面
        if let Some(page) = self.pages.get(self.current_page_index) {
            // 每次选取6个程序，并显示在同一行
            let chunks: Vec<_> = page.program_links.chunks(6).collect();
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
                                if ui.button("编辑")
                                .clicked() {
                                    println!("{:?}", self.cached_icon);
                                    // 先关掉已经打开的设置窗口
                                    if self.setting_open {
                                        self.setting_open = false;
                                        // 清除之前设置窗口的临时图片
                                        self.icon_will_clean.push(self.temp_icon_path.clone());
                                    }
                                    // 打开设置窗口
                                    self.setting_open = true;
                                    // 设置当前设置页面和链接
                                    self.current_setting_page = self.current_page_index;
                                    self.current_setting_link = link_index;

                                    self.temp_name = program.name.clone();
                                    self.temp_icon_path = program.icon_path.clone();
                                    self.temp_run_command = program.run_command.clone();
                                    // println!("{} {} {}", self.current_setting_page, self.current_setting_link, self.pages.len());
                                    ui.close_menu();
                                }
                                
                                if ui.button("删除")
                                .clicked() {
                                    
                                    println!("删除");
                                    
                                    self.link_should_delete = Some((self.current_page_index, link_index));
                                    
                                    ui.close_menu();
                                }
                            });
                            
                            
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
                    }
                });
                
                // 只有在不是最后一个chunk时才添加间隔
                if i < chunks.len() - 1 {
                    // ui.separator();
                    ui.label("");
                }
            }
        }
    }
}

use eframe::egui::{self, Align, Layout};
use std::process::Command;
use std::collections::HashSet;

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
                    ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                });
                
                if self.setting_open {
                    // 设置页面
                    egui::Window::new("配置快捷方式")
                    .collapsible(false)
                    .resizable(false)
                    // .open(&mut self.setting_open)

                    .show(ui.ctx(), |ui| {
                        ui.add_sized(
                            egui::vec2(96.0, 96.0),
                            egui::Image::new(format!("file://{}", &self.temp_icon_path))
                        );
                        
                        ui.add(egui::TextEdit::singleline(&mut self.temp_name).hint_text("名称"));

                        ui.add(egui::TextEdit::singleline(&mut self.temp_icon_path).hint_text("图标路径"));

                        ui.add(egui::TextEdit::singleline(&mut self.temp_run_command).hint_text("命令"));

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
                                if self.cached_icon[&current_link.icon_path].is_empty() {
                                    ui.ctx().forget_image(&format!("file://{}", &current_link.icon_path));
                                    self.cached_icon.remove(&current_link.icon_path);
                                }

                                current_link.name = self.temp_name.clone();
                                current_link.icon_path = self.temp_icon_path.clone();
                                current_link.run_command = self.temp_run_command.clone();
                                self.setting_open = false;
                            }
                            if ui.button("取消").clicked() {
                                println!("取消");
                                // 如果此图片没有被其他程序使用，则删除缓存
                                if !self.cached_icon.contains_key(&self.temp_icon_path) {
                                    ui.ctx().forget_image(&format!("file://{}", &self.temp_icon_path));
                                }
                                self.setting_open = false;
                            }
                        })});
                    });
                };
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
        if let Some(page) = self.pages.get(self.current_page_index) {
            // 每次选取6个程序，并显示在同一行
            let chunks: Vec<_> = page.program_links.chunks(6).collect();
            for (i, chunk) in chunks.iter().enumerate() {
                ui.horizontal(|ui| {
                    for (link_index, program) in (*chunk).iter().enumerate() {
                        // 图标与名称
                        ui.vertical(|ui| {
                            // 注册对icon_path的缓存
                            if !self.cached_icon.contains_key(&program.icon_path) {
                                self.cached_icon.insert(program.icon_path.clone(), HashSet::new());
                            }
                            if !self.cached_icon[&program.icon_path].contains(&program.uuid) {
                                if let Some(icon_path) = self.cached_icon.get_mut(&program.icon_path) {
                                    icon_path.insert(program.uuid.clone());
                                } else {
                                    self.cached_icon.insert(program.icon_path.clone(), HashSet::from_iter(vec![program.uuid.clone()]));
                                }
                            }

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
                                ui.label(&program.name);
    
                                if ui.button("运行")
                                .clicked() {
                                    println!("运行");
                                    ui.close_menu();
                                }
                                if ui.button("修改")
                                .clicked() {
                                    println!("{:?}", self.cached_icon);
                                    // 先关掉已经打开的设置窗口
                                    if self.setting_open {
                                        self.setting_open = false;
                                        // 清除之前设置窗口的临时图片
                                        if !self.cached_icon.contains_key(&self.temp_icon_path) {
                                            ui.ctx().forget_image(&format!("file://{}", &self.temp_icon_path));
                                        }
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

use egui;
use log::info;
use strsim::jaro_winkler;
use pinyin::ToPinyin;

use crate::my_structs::*;


impl MyApp {
    pub(super) fn show_top(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_wrapped(|ui| {

                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                        ui.heading(egui::RichText::new(&self.title))
                        .context_menu(|ui| {

                            if ui.button("自身信息").clicked() {
                                info!("{:#?}", self);
                            }

                            if ui.button("所有快捷方式").clicked() {
                                info!("{:#?}", self.program_links);
                            }
                            if ui.button("已缓存的图片").clicked() {
                                info!("{:#?}", self.texture_mgr.usage_map());
                            }

                            if ui.button("隐藏").clicked() {
                                self.hide_window();
                            }

                            if ui.button("获取图标").clicked() {
                                crate::utils::get_icon_from_exe("C:\\Windows\\System32\\notepad.exe").unwrap();
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

                let ctx = ui.ctx();

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
    }
}

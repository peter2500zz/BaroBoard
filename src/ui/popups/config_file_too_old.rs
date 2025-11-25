
use crate::my_structs::MyApp;

use super::Popup;

#[derive(Debug)]
pub struct ConfigTooOld {
    confirm: bool,
}

impl ConfigTooOld {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            confirm: false
        })
    }
}

impl Popup for ConfigTooOld {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;

        // 删除快捷方式弹窗
        let popup = egui::Window::new("配置文件版本过旧")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        .fade_in(true)
        .fade_out(true)
        .open(showing)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    "仍然尝试读取？"
                );

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("是").color(egui::Color32::RED))
                        .clicked() {
                            self.confirm = true;
                            should_close = true;
                        }
                        if ui.button("否").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });

        if should_close {
            *showing = false
        };

        return popup.is_none();
    }

    fn close_desc(&self) -> String {
        if self.confirm {
            "尝试强制读取配置文件".to_string()
        } else {
            "配置文件过旧弹窗关闭".to_string()
        }
    }

    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>> {
        let confirmed = self.confirm;
        Some(Box::new(move |app| {
            if confirmed {
                app.force_read_config();
            } else {
                app.wont_save = true;
            }
        }))
    }
}

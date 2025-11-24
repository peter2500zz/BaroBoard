use log::{debug, warn};

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_config_file_too_old(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_force_read = false;

        // 删除快捷方式弹窗
        egui::Window::new("配置文件版本过旧")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("配置文件版本过旧");
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
                            should_force_read = true;
                            should_close = true;
                        }
                        if ui.button("否").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            debug!("配置文件过旧弹窗关闭");
            // debug!("*你* 关闭了对吧？");
            // 用户关闭

            self.popups.called = false;

            if should_force_read {
                warn!("尝试强制读取配置文件");
                self.force_read_config();
            }
        }
    }
}

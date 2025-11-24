use log::debug;

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_config_file_format_error(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_auto_fix = false;

        egui::Window::new("无法读取配置文件")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("嘿！我无法读取你的配置文件！");
                ui.separator();
                ui.label(
                    "这可能是由于配置文件的版本过旧，或是配置文件被损坏。"
                );
                ui.label(
                    "你可以尝试自动修复程序，否则为了安全起见，程序将不会自动保存你的任何操作，直到你有了一份正确的配置文件。"
                );

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("尝试修复").color(egui::Color32::RED))
                        .clicked() {
                            should_auto_fix = true;
                            should_close = true;
                        }
                        if ui.button("在无自动保存的情况下继续").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });

        if (!show && !should_close && self.popups.called) || should_close {
            debug!("配置文件格式错误弹窗关闭");
            // debug!("*你* 关闭了对吧？");
            // 用户关闭
            if should_auto_fix {
                self.config_auto_fix();
            }

            self.popups.called = false;
        }
    }
}

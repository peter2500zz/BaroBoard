use log::debug;

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_config_not_a_json(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;

        egui::Window::new("此配置文件不是一个有效的JSON文件")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("此配置文件不是一个有效的JSON文件");
                ui.label(
                    "请检查文件结构是否正确。"
                );

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("好的").clicked() {
                            self.wont_save = true;
                            should_close = true;
                        }
                    });
                });
            });
        });

        if (!show && !should_close && self.popups.called) || should_close {
            debug!("配置文件不是JSON弹窗关闭");
            // debug!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;
        }
    }
}

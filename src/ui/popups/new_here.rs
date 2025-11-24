use log::debug;

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_new_here(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;

        // 删除快捷方式弹窗
        egui::Window::new("嗨！欢迎使用BaroBoard！")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    "这是一个轻量，快速的工具箱，旨在帮助你快速打开各种文件和程序！"
                );
                ui.label(
                    "你可以把一个文件拖进窗口里来快速创建一个快捷方式。"
                );
                ui.label(
                    "或者启动编辑模式，你会拥有更多选择。"
                );
                ui.label(
                    egui::RichText::new("顺带一提，你在创建第一个快捷方式后，下次双击启动工具箱，就默认是后台运行。").color(egui::Color32::RED)
                );
                ui.label(
                    "快速按两下 LeftAlt 键来召唤工具箱，或者右键托盘图标。"
                );
                ui.label(
                    egui::RichText::new("点叉不会关闭程序，需要右键托盘图标来退出工具箱。").color(egui::Color32::RED)
                );

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("好的").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });

        if (!show && !should_close && self.popups.called) || should_close {
            debug!("新用户弹窗关闭");

            self.popups.called = false;
        }
    }
}

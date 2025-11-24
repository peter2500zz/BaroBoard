use log::debug;

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_delete_tag(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("你确定要删除这个标签吗？")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个标签吗？");
                ui.label(format!(
                    "所有快捷方式的 “{}” 标签将会被删除", 
                    self
                    // 这里不能unwarp的原因是
                    // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
                    .popups.tag_to_delete
                ));

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            self.tags.remove(&self.popups.tag_to_delete);

                            debug!("删除成功: {:?}", self.popups.tag_to_delete);

                            should_save = true;
                            should_close = true;
                        }
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            debug!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }
}

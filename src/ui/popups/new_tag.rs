use log::debug;

use crate::my_structs::MyApp;


impl MyApp {
    pub(super) fn show_new_tag(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("创建一个新的标签")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("创建一个新的标签");

                ui.add(egui::TextEdit::singleline(&mut self.popups.tag_new).hint_text("请输入标签名称"));

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {

                        if self.popups.tag_new.is_empty() {
                            ui.disable();
                        }

                        if ui.button(egui::RichText::new("创建"))
                        .clicked() {
                            self.tags.insert(self.popups.tag_new.clone());

                            debug!("创建成功: {:?}", self.popups.tag_new);

                            should_save = true;
                            should_close = true;
                        }
                        });

                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });
        });


        if (!show && !should_close && self.popups.called) || should_close {
            debug!("创建新标签弹窗关闭");
            // debug!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }
}

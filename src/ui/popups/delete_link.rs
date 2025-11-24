use log::debug;

use crate::my_structs::{MyApp, ProgramLink};


impl MyApp {
    pub(super) fn show_delete_link(&mut self, ui: &mut egui::Ui) {
        let mut show = self.popups.called.clone();
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        egui::Window::new("你确定要删除这个快捷方式吗？")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fade_in(true)
        .fade_out(true)
        .open(&mut show)

        .show(ui.ctx(), |ui| {
            let current_index = self.popups.link_to_delete.index_of_the_link;

            ui.vertical_centered(|ui| {
                ui.heading("你确定要删除这个快捷方式吗？");
                ui.label(format!(
                    "“{}”将会永久消失！（真的很久！）", 
                    self
                    // 这里不能unwarp的原因是
                    // egui关闭窗口的动画效果会延迟关闭，这段时间内仍然会被使用
                    .program_links.get(current_index).unwrap_or(&ProgramLink::default())
                    //(ProgramLink { name: "已删除".to_string(), ..Default::default()})
                    .name.get(0).unwrap_or(&"已删除".to_string())
                ));

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            let program_links = &mut self.program_links;

                            let icon_path = program_links[current_index].icon_path.clone();
                            let uuid = program_links[current_index].uuid.clone();
                            self.texture_mgr.release_usage(&icon_path, &uuid);

                            let name = program_links[current_index].name.clone();
                            program_links.remove(current_index);
                            debug!("删除成功: {:?}", name);

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
            debug!("删除快捷方式弹窗关闭");
            // debug!("*你* 关闭了对吧？");
            // 用户关闭
            self.popups.called = false;

            if should_save {
                self.save_conf();
            }
        }
    }
}

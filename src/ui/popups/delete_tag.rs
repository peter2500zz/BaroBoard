
use crate::my_structs::MyApp;

use super::Popup;

#[derive(Debug)]
pub struct DeleteTag {
    tag: String,

    confirm: bool,
}

impl DeleteTag {
    pub fn new(tag: String) -> Box<Self> {
        Box::new(Self {
            tag,
            confirm: false
        })
    }
}

impl Popup for DeleteTag {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        let popup = egui::Window::new("你确定要删除这个标签吗？")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        .fade_in(true)
        .fade_out(true)
        .open(showing)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(format!(
                    "所有快捷方式的 “{}” 标签将会被删除", 
                    self.tag
                ));

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("确定").color(egui::Color32::RED))
                        .clicked() {
                            self.confirm = true;

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

        if should_close {
            *showing = false
        };

        return popup.is_none();
    }

    fn close_desc(&self) -> String {
        if self.confirm {
            format!("删除成功: {:?}", self.tag)
        } else {
            "没有删除任何东西".to_string()
        }
    }

    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>> {
        if self.confirm {
            let tag = self.tag.clone();
            Some(Box::new(move |app| {
                app.tags.remove(&tag);
            }))
        } else {
            None
        }
    }
}

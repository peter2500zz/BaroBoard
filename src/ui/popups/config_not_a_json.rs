
use crate::my_structs::MyApp;

use super::Popup;

#[derive(Debug)]
pub struct ConfigNotAJson;

impl ConfigNotAJson {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Popup for ConfigNotAJson {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;

        let popup = egui::Window::new("此配置文件不是一个有效的JSON文件")
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
                    "请检查文件结构是否正确。"
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

        if should_close {
            *showing = false
        };

        return popup.is_none();
    }

    fn close_desc(&self) -> String {
        "配置文件不是JSON弹窗关闭".to_string()
    }
    // #D11F3C
    // #FD2D5C
    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>> {
        Some(Box::new(|app| {
            app.wont_save = true;
        }))
    }
}

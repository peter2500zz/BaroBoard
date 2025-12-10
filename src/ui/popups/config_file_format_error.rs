
use crate::my_structs::MyApp;

use super::Popup;

#[derive(Debug)]
pub struct ConfigFormatError {
    confirm: bool,
}

impl ConfigFormatError {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            confirm: false
        })
    }
}

impl Popup for ConfigFormatError {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;

        let popup = egui::Window::new("无法读取配置文件")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        .fade_in(true)
        .fade_out(true)
        .open(showing)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
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
                            self.confirm = true;
                            should_close = true;
                        }
                        if ui.button("在无自动保存的情况下继续").clicked() {
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
                app.config_auto_fix();
            } else {
                app.wont_save = true;
            }
        }))
    }
}

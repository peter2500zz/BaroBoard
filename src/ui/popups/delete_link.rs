
use crate::my_structs::{MyApp, ProgramLink};

use super::Popup;

#[derive(Debug)]
pub struct DeleteLink {
    index: usize,
    pl: ProgramLink,

    confirm: bool,
}

impl DeleteLink {
    pub fn new(index: usize, pl: ProgramLink) -> Box<Self> {
        Box::new(Self {
            index,
            pl,
            confirm: false
        })
    }
}

impl Popup for DeleteLink {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        let popup = egui::Window::new("你确定要删除这个快捷方式吗？")
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
                    "“{}”将会永久消失！（真的很久！）", 
                    self.pl.name.get(0).unwrap_or(&"已删除".to_string())
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
            format!("删除成功: {:?}", self.pl.name.get(0).unwrap_or(&"已删除".to_string()))
        } else {
            "删除快捷方式弹窗关闭".to_string()
        }
    }

    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>> {
        if self.confirm {
            let index = self.index;
            Some(Box::new(move |app| {
                let program_link = &mut app.program_links[index];

                let icon_path = program_link.icon_path.clone();
                let uuid = program_link.uuid.clone();
                app.texture_mgr.release_usage(&icon_path, &uuid);

                app.program_links.remove(index);
                app.save_conf();
            }))
        } else {
            None
        }
    }
}

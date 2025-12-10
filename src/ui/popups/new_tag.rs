
use crate::my_structs::MyApp;
use super::Popup;

#[derive(Debug)]
pub struct NewTag {
    new_tag: String,

    confirm: bool,
}

impl NewTag {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            new_tag: "".to_string(),
            confirm: false
        })
    }
}

impl Popup for NewTag {
    fn show(&mut self, ui: &mut egui::Ui, showing: &mut bool) -> bool {
        let mut should_close = false;
        let mut should_save = false;

        // 删除快捷方式弹窗
        let popup = egui::Window::new("创建一个新的标签")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::pos2(crate::WINDOW_SIZE.0 / 2.0, crate::WINDOW_SIZE.1 / 2.0))
        .fade_in(true)
        .fade_out(true)
        .open(showing)

        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {

                ui.add(egui::TextEdit::singleline(&mut self.new_tag).hint_text("请输入标签名称"));

                ui.separator();

                ui.with_layout(egui::Layout {
                    cross_align: egui::Align::RIGHT,
                    ..Default::default()
                }, |ui| {
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {

                        if self.new_tag.is_empty() {
                            ui.disable();
                        }

                        if ui.button(egui::RichText::new("创建"))
                        .clicked() {
                            self.confirm = true;

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


        if should_close {
            *showing = false
        };

        return popup.is_none();
    }

    fn close_desc(&self) -> String {
        if self.confirm {
            format!("创建成功: {:?}", self.new_tag)
        } else {
            "创建新标签弹窗关闭".to_string()
        }
    }

    fn on_close(&self) -> Option<Box<dyn FnOnce(&mut MyApp)>> {
        if self.confirm {
            let tag = self.new_tag.clone();
            Some(Box::new(move |app| {
                app.tags.insert(tag);
                app.save_conf();
            }))
        } else {
            None
        }
    }
}
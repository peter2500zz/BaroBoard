use egui;

use crate::{my_structs::*, ui::popups::{delete_tag::DeleteTag, new_tag::NewTag}};


impl MyApp {
    pub(super) fn show_side_bar(&mut self, ui: &mut egui::Ui) {

        ui.vertical_centered_justified(|ui| {
            if !self.edit_mode && self.tags.is_empty() {
                ui.label(egui::RichText::new("这里还没有任何标签！").weak());
            }

            ui.vertical_centered_justified(|ui| {
                for tag in self.tags.clone() {
                    if self.edit_mode {
                        self.current_tag = None;
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::LIGHT_RED;
                    }

                    let is_selected = self.current_tag.as_ref().unwrap_or(&"".to_string()) == &tag;

                    if ui.selectable_label(
                        is_selected,
                        tag.clone()
                    )

                    .clicked() {
                        if self.edit_mode {
                            self.popupgmr.show(DeleteTag::new(tag));
                        } else {
                            if is_selected {
                                self.current_tag = None;
                            } else {
                                self.current_tag = Some(tag.clone());
                            }
                        }
                    }
                }
            });

            if self.edit_mode {
                if ui.button("➕").clicked() {
                    self.popupgmr.show(NewTag::new());
                }
            }
        });
    }
}

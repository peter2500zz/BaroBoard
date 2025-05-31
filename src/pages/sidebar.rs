use egui;

use crate::my_structs::*;


impl MyApp {
    pub fn side_bar(&mut self, ui: &mut egui::Ui) {
        
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
                    
                    .clicked() && !self.popups.called {
                        if self.edit_mode {
                            self.popups.delete_tag(tag.clone());
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
                    if !self.popups.called {
                        // debug!("添加标签");
                        self.popups.new_tag();
                    }
                }
            }
        });
    }
}

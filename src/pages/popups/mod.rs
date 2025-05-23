pub mod link;
// pub mod info;

use link::save;

use crate::my_structs::*;

#[derive(Clone)]
pub enum PopupType {
    LinkConfig,
    Delete,
    CannotSave,
    Info,
}

pub struct Popups {
    pub called: bool,
    popup_type: Option<PopupType>,
    link_config: link::config::LinkConfig,
    // pub info: info::Info,
}

impl Popups {
    pub fn new() -> Self {
        Self {
            called: false,
            popup_type: None,
            link_config: link::config::LinkConfig::new(),
            // info: info::Info::new(),
        }
    }

    pub fn save_conf(&mut self, program_links: Vec<ProgramLink>) {
        match save::save_conf(program_links) {
            Ok(_) => println!("保存成功"),
            Err(e) => {
                println!("保存失败: {}", e);
                self.called = true;
                self.popup_type = Some(PopupType::CannotSave);
            },
        }
    }

    pub fn config_existing_link(&mut self, position: LinkPosition, link: &ProgramLink) {
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_existing_link(position, link);
    }

    pub fn config_new_link(&mut self) {
        self.called = true;
        self.popup_type = Some(PopupType::LinkConfig);
        self.link_config.config_new_link();
    }
    
}


impl MyApp {
    pub fn show_popup(&mut self, ui: &mut egui::Ui) {
        if self.popups.called {
            if let Some(popup_type) = self.popups.popup_type.clone() {
                match popup_type {
                    PopupType::LinkConfig => self.show_link_config(ui),
                    // PopupType::Delete => self.popups.info.show(ui),
                    // PopupType::Info => self.popups.info.show(ui),
                    _ => {}
                }
            }
        }
    }
}

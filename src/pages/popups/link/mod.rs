pub mod config;
pub mod save;
pub mod delete;


pub struct LinkPopups {
    pub link_config: config::LinkConfig,
    pub link_save: save::LinkSave,
    pub link_delete: delete::LinkDelete,
}

impl LinkPopups {
    pub fn new() -> Self {
        Self {
            link_config: config::LinkConfig::new(),
            link_save: save::LinkSave::new(),
            link_delete: delete::LinkDelete::new(),
        }
    }
}

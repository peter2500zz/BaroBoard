use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};
use std::collections::HashSet;
use log::info;

use crate::my_structs::*;

#[derive(Serialize, Deserialize)]
pub struct LinkConfigSchema {
    pub version: u32,
    pub tags: HashSet<String>,
    pub program_links: Vec<ProgramLink>,
}

impl Default for LinkConfigSchema {
    fn default() -> Self {
        Self {
            version: crate::CONFIG_FILE_VERSION,
            tags: HashSet::new(),
            program_links: Vec::new(),
        }
    }
}

pub fn save_conf(program_links: Vec<ProgramLink>, tags: HashSet<String>) -> Result<(), std::io::Error> {
    save_conf_to_path(program_links, tags, format!("{}/{}", crate::CONFIG_SAVE_PATH, crate::CONFIG_FILE_NAME).as_str())
}


pub fn save_conf_to_path(program_links: Vec<ProgramLink>, tags: HashSet<String>, path: &str) -> Result<(), std::io::Error> {
    let links_config = LinkConfigSchema {
        version: crate::CONFIG_FILE_VERSION,
        tags: tags,
        program_links: program_links,
    };

    let serialized = serde_json::to_string_pretty(&links_config)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    info!("保存配置文件: {}", path);
    Ok(())
}

pub fn load_conf(path: &str) -> Result<serde_json::Value, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;


    let links_config: serde_json::Value = serde_json::from_str(&buffer)?;

    info!("加载配置文件: {}", path);
    Ok(links_config)
}

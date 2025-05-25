use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};
use std::collections::HashSet;

use crate::my_structs::*;


pub const CURRENT_VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct LinkConfigSchema {
    pub version: u32,
    pub tags: HashSet<String>,
    pub program_links: Vec<ProgramLink>,
}

pub fn save_conf(program_links: Vec<ProgramLink>, tags: HashSet<String>) -> Result<(), std::io::Error> {
    save_conf_to_path(program_links, tags, ".links.json")
}


pub fn save_conf_to_path(program_links: Vec<ProgramLink>, tags: HashSet<String>, path: &str) -> Result<(), std::io::Error> {
    let links_config = LinkConfigSchema {
        version: CURRENT_VERSION,
        tags: tags,
        program_links: program_links,
    };

    let serialized = serde_json::to_string_pretty(&links_config)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn load_conf(path: &str) -> Result<serde_json::Value, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;


    let links_config: serde_json::Value = serde_json::from_str(&buffer)?;

    Ok(links_config)
}

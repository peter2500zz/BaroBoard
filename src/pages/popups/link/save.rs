use egui;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};

use crate::my_structs::*;


#[derive(Serialize, Deserialize)]
pub struct LinkConfigSchema {
    pub version: String,
    pub program_links: Vec<ProgramLink>,
}

pub fn save_conf(program_links: Vec<ProgramLink>) -> Result<(), std::io::Error> {
    save_conf_to_path(program_links, ".links.json")
}


pub fn save_conf_to_path(program_links: Vec<ProgramLink>, path: &str) -> Result<(), std::io::Error> {
    let links_config = LinkConfigSchema {
        version: "0.1.1".to_string(),
        program_links: program_links,
    };

    let serialized = serde_json::to_string_pretty(&links_config)?;
    let mut file = File::create(path)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn load_conf(path: &str) -> Result<LinkConfigSchema, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;


    let links_config: LinkConfigSchema = serde_json::from_str(&buffer)?;

    Ok(links_config)
}

use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

use crate::my_structs::*;


#[derive(Serialize, Deserialize)]
pub struct LinksConfig {
    pub version: String,
    pub pages: Vec<Page>
}

impl MyApp {
    pub fn load_conf(path: &str) -> Result<LinksConfig, std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;


        let links_config: LinksConfig = serde_json::from_str(&buffer)?;

        Ok(links_config)
    }
}

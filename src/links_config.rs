use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};

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

    pub fn save_conf(&self) -> Result<(), std::io::Error> {
        self.save_conf_to_path(".links.json")
    }

    pub fn save_conf_to_path(&self, path: &str) -> Result<(), std::io::Error> {
        let links_config = LinksConfig {
            version: "1.0".to_string(),
            pages: self.pages.clone(),
        };

        let serialized = serde_json::to_string_pretty(&links_config)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }
}

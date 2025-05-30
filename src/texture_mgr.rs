use md5;
use crate::my_structs;


impl my_structs::MyApp {
    #[cfg(target_os = "windows")]
    pub fn save_exe_icon(&self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        use crate::utils::windows_utils::get_icon_from_exe;
        
        let icon = get_icon_from_exe(&path)?;
        
        std::fs::create_dir_all(format!("{}/cache/exe_icon", crate::CONFIG_SAVE_PATH))?;

        let icon_path = format!("{}/cache/exe_icon/{:x}.png", crate::CONFIG_SAVE_PATH, md5::compute(path.as_bytes()));
        if !std::path::Path::new(&icon_path).exists() {
            std::fs::write(icon_path, icon)?;
        }

        Ok(())
    }
}

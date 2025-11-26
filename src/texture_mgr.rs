use std::{collections::{HashMap, HashSet}, fs};
use log::debug;
use md5;

use crate::utils::{get_icon_from_exe, get_invalid_icon};

#[derive(Debug, Default)]
pub struct TextureManager {
    // icon_path -> set of owners (uuid)
    icon_refs: HashMap<String, HashSet<String>>,
    // 图标忘记执行的等待队列
    pending_forget: Vec<String>,
}

impl TextureManager {
    pub fn register_usage(&mut self, icon_path: &str, owner_id: &str) {
        self.icon_refs
            .entry(icon_path.to_string())
            .or_insert_with(HashSet::new)
            .insert(owner_id.to_string());
    }

    pub fn release_usage(&mut self, icon_path: &str, owner_id: &str) {
        if icon_path.is_empty() {
            return;
        }

        if let Some(set) = self.icon_refs.get_mut(icon_path) {
            set.remove(owner_id);
        } else {
            self.icon_refs.insert(icon_path.to_string(), HashSet::new());
        }

        self.schedule_forget(icon_path.to_string());
    }

    pub fn schedule_forget(&mut self, icon_path: String) {
        if icon_path.is_empty() || self.pending_forget.contains(&icon_path) {
            return;
        }

        self.pending_forget.push(icon_path);
    }

    pub fn cleanup(&mut self, ctx: &egui::Context) {
        for icon_path in std::mem::take(&mut self.pending_forget) {
            if self
                .icon_refs
                .get(&icon_path)
                .map_or(false, |set| !set.is_empty())
            {
                debug!("图片仍在被使用，将不会释放 {}", icon_path);
                continue;
            }

            debug!("释放图片资源 {}", icon_path);
            ctx.forget_image(&format!("file://{}", icon_path));
            self.icon_refs.remove(&icon_path);

            match std::fs::remove_file(&icon_path) {
                Ok(_) => debug!("删除缓存图片资源 {} 成功", icon_path),
                Err(e) => debug!("删除缓存图片资源 {} 失败: {}", icon_path, e),
            }
        }
    }

    pub fn usage_map(&self) -> &HashMap<String, HashSet<String>> {
        &self.icon_refs
    }
}

pub fn cache_img(img_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(format!("{}/cache/exe_icon", crate::CONFIG_SAVE_PATH))?;

    let icon_path = format!(
        "{}/cache/exe_icon/{:x}.{}",
        crate::CONFIG_SAVE_PATH,
        md5::compute(img_path.as_bytes()),
        std::path::Path::new(&img_path).extension().unwrap_or_default().to_string_lossy()
    );
    debug!("缓存 {} 至 {}", img_path, icon_path);
    if !std::path::Path::new(&icon_path).exists() {
        std::fs::write(icon_path.clone(), fs::read(img_path)?)?;
    }

    Ok(icon_path)
}


pub fn save_icon(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let icon = get_icon_from_exe(&path)?;

    std::fs::create_dir_all(format!("{}/cache/exe_icon", crate::CONFIG_SAVE_PATH))?;

    let icon_path = format!(
        "{}/cache/exe_icon/{:x}.png",
        crate::CONFIG_SAVE_PATH,
        md5::compute(path.as_bytes())
    );
    if !std::path::Path::new(&icon_path).exists() {
        std::fs::write(icon_path.clone(), icon)?;
    }

    Ok(icon_path)
}

pub fn save_invalid_icon() -> Result<String, Box<dyn std::error::Error>> {
    let icon = get_invalid_icon();

    std::fs::create_dir_all(format!("{}/cache/exe_icon", crate::CONFIG_SAVE_PATH))?;

    let icon_path = format!("{}/cache/exe_icon/_.png", crate::CONFIG_SAVE_PATH);
    if !std::path::Path::new(&icon_path).exists() {
        std::fs::write(icon_path.clone(), icon)?;
    }

    Ok(icon_path)
}

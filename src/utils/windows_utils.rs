#[cfg(target_os = "windows")]
use systemicons::get_icon;

#[cfg(target_os = "windows")]
pub fn get_icon_from_exe(exe_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let sizes = [1024, 512, 256, 128, 96, 64, 48, 32, 24, 16];
    
    for size in sizes {
        if let Ok(icon) = get_icon(exe_path, size) {
            return Ok(icon);
        }
    }
    
    Err("无法获取图标".into())
}

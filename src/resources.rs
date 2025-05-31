use std::sync::Arc;
use log::{error, info, warn};

pub const LOGO_ICO: &[u8] = include_bytes!("../assets/logo.ico");


//自定义字体
pub fn setup_custom_fonts(ctx: &egui::Context) {
    // 创建一个默认的字体定义对象
    let mut fonts = egui::FontDefinitions::default();

    // 根据不同操作系统选择不同的字体路径
    let font_path = if cfg!(target_os = "windows") {
        // Windows系统下微软雅黑的默认位置
        std::path::Path::new("C:/Windows/Fonts/msyh.ttc")
    } else if cfg!(target_os = "linux") {
        // Linux系统下常见的中文字体路径，尝试几个常见位置
        let possible_paths = [
            "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",         // Noto Sans CJK
            "/usr/share/fonts/wenquanyi/wqy-microhei.ttc",           // 文泉驿微米黑
            "/usr/share/fonts/wenquanyi/wqy-zenhei.ttc",             // 文泉驿正黑
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf", // Droid Sans
        ];
        
        // 查找第一个存在的字体文件
        let mut found_path = std::path::Path::new("/usr/share/fonts"); // 默认路径
        for path in possible_paths {
            let full_path = std::path::Path::new(path);
            if full_path.exists() {
                found_path = full_path;
                break;
            }
        }
        found_path
    } else if cfg!(target_os = "macos") {
        // macOS系统下常见的中文字体
        std::path::Path::new("/System/Library/Fonts/PingFang.ttc")  // 苹方字体
    } else {
        // 其他操作系统使用一个不太可能存在的路径，将回退到默认字体
        std::path::Path::new("/non-existent-path")
    };
    
    if font_path.exists() {
        // 如果找到字体文件，从文件读取
        match std::fs::read(font_path) {
            Ok(font_data) => {
                info!("使用字体: {}", font_path.display());
                fonts.font_data.insert(
                    "my_font".to_owned(),
                    // 这里也使用Arc共享字体数据，但这里的Arc主要用于避免数据复制，而非线程安全
                    // 在egui中，Arc用于智能地共享大型资源(如字体)，减少内存使用
                    Arc::new(egui::FontData::from_owned(font_data)),
                );
                
                // 将字体添加到 Proportional 字体族的第一个位置
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "my_font".to_owned());

                // 将字体添加到 Monospace 字体族的末尾
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("my_font".to_owned());
            },
            Err(err) => {
                error!("无法加载系统字体 {:?}: {}", font_path, err);
                warn!("将使用默认字体");
                // 加载失败时使用默认字体
            }
        }
    } else {
        warn!("用默认字体");
    }

    // 将字体设置应用到 egui 上下文
    ctx.set_fonts(fonts);
}

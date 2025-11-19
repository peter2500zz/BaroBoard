use std::sync::Arc;
use log::{error, info, warn};

pub const LOGO_ICO: &[u8] = include_bytes!("../assets/logo.ico");


//自定义字体
pub fn setup_custom_fonts(ctx: &egui::Context) {
    // 创建一个默认的字体定义对象
    let mut fonts = egui::FontDefinitions::default();

    // 根据不同操作系统选择不同的字体路径
    let font_path = std::path::Path::new("C:/Windows/Fonts/msyh.ttc");
    
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

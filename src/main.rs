mod my_structs;
mod pages;
mod links_config;

use eframe::egui;
use std::sync::Arc;

use crate::my_structs::*;

fn main() -> Result<(), eframe::Error> {
    let eframe_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800., 500.])
            .with_resizable(false)
            .with_title("BaroBoard 工具箱")
            ,
        ..Default::default()
    };
    
    eframe::run_native(
        "My egui App", // 应用程序的标题
        eframe_options, // 视口选项
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::new()))
        }),
    )

}


//自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
    // 创建一个默认的字体定义对象
    let mut fonts = egui::FontDefinitions::default();

    //安装的字体支持.ttf和.otf文件
    //文件放在main.rs的同级目录下（可以自定义到其它目录）
    fonts.font_data.insert(
        "my_font".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "assets/fonts/msyh.ttc"
        ))),
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

    // 将加载的字体设置到 egui 的上下文中
    ctx.set_fonts(fonts);
}


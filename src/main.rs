mod my_structs;
mod pages;

use eframe::egui;
use std::sync::Arc;
use std::process::Command;

use crate::my_structs::*;

fn main() -> Result<(), eframe::Error> {
    let eframe_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800., 500.])
            .with_resizable(false),
            
        ..Default::default()
    };
    
    eframe::run_native(
        "My egui App", // 应用程序的标题
        eframe_options, // 视口选项
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(cc)))
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


impl MyApp {
    fn show_page(&mut self, ui: &mut egui::Ui) {
        if let Some(page) = self.pages.get(self.current_page_index) {
            // 每次选取6个程序，并显示在同一行
            for chunk in page.programms.chunks(6) {
                ui.horizontal(|ui| {
                    for program in chunk {
                        let response = ui.add_sized(
                            egui::vec2(96.0, 96.0),
                            egui::ImageButton::new(egui::include_image!("assets/images/Grass_Block_JE7_BE6.png"))
                        ).on_hover_ui_at_pointer(|ui| {
                            ui.label(&program.name);
                        });
                        
                        if response.clicked() {
                            match Command::new(&program.run_command).spawn() {
                                Ok(_) => println!("运行成功"),
                                Err(e) => {
                                    println!("运行失败: {}", e);
                                    
                                },
                            }
                        }
                        
                        response.context_menu(|ui| {
                            ui.label(&program.name);

                            if ui.button("运行")
                            .clicked() {
                                println!("运行");
                                ui.close_menu();
                            }
                            if ui.button("修改")
                            .clicked() {
                                if !self.setting_open {
                                    self.setting_open = true;
                                    self.setting_ui_closure = Some(Box::new(|ui| {
                                        ui.label("哈？");
                                    }));
                                }
                                ui.close_menu();
                            }
                            
                            if ui.button("删除")
                            .clicked() {
                                println!("删除");
                                ui.close_menu();
                            }
                        });
                    }
                });
            }
        }
    }
}


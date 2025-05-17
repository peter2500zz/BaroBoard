use eframe::egui;
use std::sync::Arc;

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


enum Pages {
    Setting,
    Test,
}

struct MyApp {
    page: Pages,
    title: String,
    search_text: String,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            page: Pages::Setting,
            title: "设置".to_string(),
            search_text: "".to_string(),
        }
    }

    fn side_bar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            if ui.button("设置").clicked() {
                self.page = Pages::Setting;
            }
            if ui.button("测试").clicked() {
                self.page = Pages::Test;
            }
        });
    }

    fn show_page(&mut self, ui: &mut egui::Ui) {
        match self.page {
            Pages::Setting => {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("assets/images/Grass_Block_JE7_BE6.png"))
                        .fit_to_exact_size(egui::vec2(96.0, 96.0))
                    );
                    
                    ui.add(
                        egui::Image::new(egui::include_image!("assets/images/Grass_Block_JE7_BE6.png"))
                        .fit_to_exact_size(egui::vec2(96.0, 96.0))
                    );

                    ui.add(
                        egui::Image::new(egui::include_image!("assets/images/Grass_Block_JE7_BE6.png"))
                        .fit_to_exact_size(egui::vec2(96.0, 96.0))
                    );
                    
                });
                self.title = "设置".to_string();
            },
            Pages::Test => {
                ui.label("测试页面");
                self.title = "测试".to_string();
            },
        }
    }

    fn main_ui(&mut self, ui: &mut egui::Ui)  {        
        // 添加面板的顺序非常重要，影响最终的布局
        egui::TopBottomPanel::top("title")
        .resizable(false)
        .min_height(32.0)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(&self.title)
                    .context_menu(|ui| {
                        ui.label("设置页面");
                        ui.label("设置页面2");
                        ui.label("设置页面3");
                    });
                });
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                });
            });
        });

        egui::SidePanel::left("side_bar")
        .resizable(true)
        .default_width(150.0)
        .width_range(80.0..=200.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("左导航栏");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.side_bar(ui);
            });
        });

        // egui::SidePanel::right("right_panel")
        // .resizable(true)
        // .default_width(150.0)
        // .width_range(80.0..=200.0)
        // .show_inside(ui, |ui| {
        //     ui.vertical_centered(|ui| {
        //         ui.heading("右导航栏");
        //     });
        //     egui::ScrollArea::vertical().show(ui, |ui| {
        //         ui.label("右导航栏内容");
        //     });
        // });

        // egui::TopBottomPanel::bottom("bottom_panel")
        // .resizable(false)
        // .min_height(0.0)
        // .show_inside(ui, |ui| {
        //     ui.vertical_centered(|ui| {
        //         ui.heading("状态栏");
        //     });
        //     ui.vertical_centered(|ui| {
        //         ui.label("状态栏内容");
        //     });
        // });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("页面内容");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_page(ui);
            });
        });
    }    
    
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("注册页面");

            // ui.label("1. 个人信息");
            self.main_ui(ui);
        });
    }
}


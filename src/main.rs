use eframe::egui;
use std::sync::Arc;
use std::process::Command;

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


struct ProgramLink {
    name: String,
    icon_path: String,
    run_command: String,
}

impl ProgramLink {
    fn new(name: String, icon_path: String, run_command: String) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
        }
    }
}


struct Page {
    programms: Vec<ProgramLink>,
    title: String,
}

impl Page {
    fn new(title: String, programms: Vec<ProgramLink>) -> Self {

        Self {
            programms: programms,
            title: title,
        }
    }
}

struct MyApp {
    pages: Vec<Page>,
    current_page_index: usize,
    title: String,
    search_text: String,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let program1 = ProgramLink::new(
            "SpaceSniffer".to_string(),
            "../assets/images/Crafting_Table_JE4_BE3.png".to_string(),
            r"D:\Softwares\SpaceSniffer.exe".to_string(),
        );
        let program2 = ProgramLink::new(
            "MAA".to_string(),
            "../assets/images/Crafting_Table_JE7_BE3.png".to_string(),
            r"D:\Softwares\MAA\MAA.exe".to_string(),
        );
        let program3 = ProgramLink::new(
            "Plain Craft Launcher 2".to_string(),
            "../assets/images/Lit_Furnace_JE7_BE2.png".to_string(),
            r"D:\Softwares\1.21\Plain Craft Launcher 2.exe".to_string(),
        );

        let pages = vec![
            Page::new(
                "默认页面".to_string(),
                vec![program1, program2]
            ),
            Page::new(
                "设置页面".to_string(),
                vec![program3]
            )
        ];

        Self {
            pages,
            current_page_index: 0,
            title: "默认页面".to_string(),
            search_text: "".to_string(),
        }
    }

    fn side_bar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            for (i, _page) in self.pages.iter().enumerate() {
                if ui.button( &_page.title).clicked() {
                    self.current_page_index = i;
                    println!("现在是第 {} 页", i);
                }
            }
        });
    }

    fn show_page(&mut self, ui: &mut egui::Ui) {
        if let Some(page) = self.pages.get(self.current_page_index) {
            ui.horizontal(|ui| {
                for program in &page.programms {
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
                        }
                        if ui.button("修改")
                        .clicked() {
                            println!("修改");
                        }
        
                        if ui.button("删除")
                        .clicked() {
                            println!("删除");
                        }
                    });
                }
            });
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
        .resizable(false)
        .default_width(150.0)
        // .width_range(80.0..=200.0)
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
            self.main_ui(ui);
        });
    }
}


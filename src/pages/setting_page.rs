use eframe::egui;
use std::process::Command;


#[derive(Debug, Clone)]
pub struct ProgramLink {
    name: String,
    icon_path: String,
    run_command: String,
}

impl ProgramLink {
    pub fn new(name: &str, icon_path: &str, run_command: &str) -> Self {
        Self { 
            name: name.to_string(), 
            icon_path: icon_path.to_string(), 
            run_command: run_command.to_string() 
        }
    }
}


pub fn page(ui: &mut egui::Ui) {
    let program1 = ProgramLink::new(
        "SpaceSniffer",
        "../assets/images/Crafting_Table_JE4_BE3.png",
        r"D:\Softwares\SpaceSniffer.exe",
    );
    let program2 = ProgramLink::new(
        "MAA",
        "../assets/images/Crafting_Table_JE7_BE3.png",
        r"D:\Softwares\MAA\MAA.exe",
    );
    let program3 = ProgramLink::new(
        "Plain Craft Launcher 2",
        "../assets/images/Lit_Furnace_JE7_BE2.png",
        r"D:\Softwares\1.21\Plain Craft Launcher 2.exe",
    );
    
    let programs = vec![program1, program2, program3];

    // 每行6个图标
    ui.horizontal(|ui| {
        for program in programs {
            let response = ui.add_sized(
                egui::vec2(96.0, 96.0),
                egui::ImageButton::new(egui::include_image!("../assets/images/Grass_Block_JE7_BE6.png"))
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

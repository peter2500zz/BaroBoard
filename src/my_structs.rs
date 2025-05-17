use eframe::egui;


pub struct ProgramLink {
    pub name: String,
    pub icon_path: String,
    pub run_command: String,
}

impl ProgramLink {
    pub fn new(name: String, icon_path: String, run_command: String) -> Self {
        Self {
            name: name,
            icon_path: icon_path,
            run_command: run_command,
        }
    }
}

pub struct Page {
    pub programms: Vec<ProgramLink>,
    pub title: String,
}


impl Page {
    pub fn new(title: String, programms: Vec<ProgramLink>) -> Self {

        Self {
            programms: programms,
            title: title,
        }
    }
}


pub struct MyApp {
    pub pages: Vec<Page>,
    pub current_page_index: usize,
    pub title: String,
    pub search_text: String,
    pub setting_open: bool,
    // 设置窗口的UI closure
    pub setting_ui_closure: Option<Box<dyn Fn(&mut egui::Ui) -> ()>>,
}

impl MyApp {
    pub 
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
            setting_open: false,
            setting_ui_closure: None,
        }
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

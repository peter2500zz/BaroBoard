use eframe::egui;

use crate::my_structs::*;


impl MyApp {
    pub fn main_ui(&mut self, ui: &mut egui::Ui)  {        
        // 添加面板的顺序非常重要，影响最终的布局
        egui::TopBottomPanel::top("title")
        .resizable(false)
        .min_height(32.0)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(&self.title)
                    .context_menu(|ui| {
                        if ui.button("添加一个子页面").clicked() {
                            self.pages.push(
                                Page::new(
                                    "新页面".to_string(),
                                    vec![]
                                )
                            );
                        }
                        if ui.button("在当前页面添加一个程序").clicked() {
                            self.pages[self.current_page_index].programms.push(
                                ProgramLink::new(
                                    "Plain Craft Launcher 2".to_string(),
                                    "../assets/images/Lit_Furnace_JE7_BE2.png".to_string(),
                                    r"D:\Softwares\1.21\Plain Craft Launcher 2.exe".to_string(),
                                )
                            );
                        }
                    });
                });
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("搜索"));
                });
                if self.setting_open {
                    egui::Window::new("配置快捷方式")
                    .collapsible(false)
                    .resizable(false)

                    .show(ui.ctx(), self.setting_ui_closure.as_ref().unwrap());
                };
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

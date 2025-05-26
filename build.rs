

fn main() {
    // 只在Windows平台上编译资源
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // 设置图标
        res.set_icon("assets/logo.ico");
        
        // 设置应用程序信息
        res.set("ProductName", "BaroBoard");
        res.set("FileDescription", "BaroBoard 工具箱");
        res.set("CompanyName", "Barometer & MyGO+");
        res.set("LegalCopyright", "Copyright (C) 2025");
        
        // 编译资源
        res.compile().unwrap();
    }
}

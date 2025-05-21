mod my_structs;
mod pages;
mod window;

use std::sync::Arc;  // Arc = 原子引用计数(Atomically Reference Counted)，一种线程安全的智能指针，允许在多个线程间共享所有权
use window::{event, glow_app};
use egui_winit::winit;
use my_structs::MyApp;

// use crate::my_structs::*;


fn main() {
    // --- 事件循环设置 ---
    // 创建事件循环，支持自定义事件类型UserEvent
    // 事件循环是GUI应用程序的核心，负责处理用户输入、窗口事件和自定义事件
    // 使用with_user_event()允许应用程序创建和处理自定义事件类型
    let event_loop = winit::event_loop::EventLoop::<event::UserEvent>::with_user_event()
        .build()
        .unwrap();
    // 创建事件代理，允许从其他线程发送事件到主事件循环
    // 这是线程间通信的关键机制，因为UI操作必须在主线程上进行
    let proxy = event_loop.create_proxy();

    // --- 异步运行时设置 ---
    // 创建Tokio异步运行时
    // Tokio是Rust的异步运行时，允许使用async/await编写高效的非阻塞并发代码
    // 它提供了任务调度器、事件循环和其他基础设施来处理异步任务
    let rt = tokio::runtime::Runtime::new().unwrap();
    // 进入运行时上下文，允许在当前线程使用tokio的异步功能
    // _guard是一个RAII守卫，当它被丢弃时会清理运行时上下文
    let _guard = rt.enter();
    
    // --- 后台任务准备 ---
    // 为后台任务创建事件代理的克隆
    // Clone是必要的，因为每个线程需要自己的代理实例来发送事件
    // 所有克隆的代理都指向同一个事件循环
    let proxy_clone = proxy.clone();

    // --- 后台任务示例(当前已注释) ---
    // 以下是一个被注释掉的后台任务，展示了如何使用tokio的异步功能来执行后台工作
    rt.spawn(async move {
        // // 为后台任务创建MyApp的克隆
        // let my_app_clone = my_app.clone();
        
        // loop {
        //     // // 模拟状态变化，例如每5秒改变UI显示状态
        //     // {
        //     //     // 获取互斥锁，修改应用状态
        //     //     // 花括号创建了一个作用域，确保lock()返回的MutexGuard在使用后立即被释放
        //     //     // 这样可以最小化锁的持有时间，防止其他线程等待过长时间
        //     //     let mut app = my_app_clone.lock().unwrap();
        //     //     app.called = !app.called; // 切换状态
        //     //     println!("状态已更改: called = {}", app.called);
        //     // } // <- MutexGuard在这里被释放，解锁Mutex
            
        //     // 通过代理发送重绘事件到主线程的事件循环
        //     // 这是线程间通信的关键机制：后台线程不能直接操作UI，但可以发送事件请求UI更新
        //     proxy_clone
        //         .send_event(event::UserEvent::HideWindow)
        //         .unwrap();
        //     proxy_clone
        //         .send_event(event::UserEvent::Redraw(Duration::ZERO))
        //         .unwrap();
                
        //     // 异步等待5秒钟，不会阻塞线程或其他任务
        //     // 这利用了tokio的异步特性，使当前任务"让出"执行权，直到指定时间后才恢复
        //     tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        //     proxy_clone
        //         .send_event(event::UserEvent::ShowWindow)
        //         .unwrap();
        //     proxy_clone
        //         .send_event(event::UserEvent::Redraw(Duration::ZERO))
        //         .unwrap();
        //     tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        // }
    });


    let winit_window_builder = winit::window::WindowAttributes::default()
        .with_resizable(false)
        .with_inner_size(winit::dpi::LogicalSize {
            width: 800.0,
            height: 500.0,
        })
        .with_title("BaroBoard 工具箱") // 参见 https://github.com/emilk/egui/pull/2279
        // .with_visible(false)
        ;

    // 创建主应用程序
    let mut app = glow_app::GlowApp::new(
        winit_window_builder,
        proxy,
        Box::new(move |egui_ctx| {
            // 安装图片加载器，允许egui加载和显示图片
            egui_extras::install_image_loaders(egui_ctx);
            // 设置自定义字体，支持中文显示
            setup_custom_fonts(egui_ctx);

            Box::new(MyApp::new())
        }),
    );

    // 启动事件循环，这通常是阻塞的，会一直运行直到应用程序关闭
    // 事件循环会不断处理输入事件、UI更新和渲染，这是GUI应用程序的主要执行模式
    event_loop.run_app(&mut app).expect("failed to run app");
}


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
                println!("已加载系统字体: {:?}", font_path);
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
                eprintln!("无法加载系统字体 {:?}: {}", font_path, err);
                // 加载失败时使用默认字体
            }
        }
    } else {
        eprintln!("未找到系统字体 {:?}，将使用默认字体", font_path);
    }

    // 将字体设置应用到 egui 上下文
    ctx.set_fonts(fonts);
}

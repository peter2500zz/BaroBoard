#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在Windows的发布版本中隐藏控制台窗口

mod my_structs;
mod pages;
mod window;

use std::sync::{Arc, Mutex};  // Arc = 原子引用计数(Atomically Reference Counted)，一种线程安全的智能指针，允许在多个线程间共享所有权
use egui_winit::winit;
use rdev::{listen, EventType, Key};
use std::time::{Duration, Instant};
use tray_item::{TrayItem, IconSource};
use single_instance::SingleInstance;

use window::{event, glow_app};
use my_structs::{MyApp, DOUBLE_ALT_COOLDOWN};




fn main() {
    let instance = SingleInstance::new("BaroBoard").unwrap();
    
    if !instance.is_single() {
        println!("BaroBoard 已经在运行");
        return;
    }

    // 创建事件循环
    let event_loop = winit::event_loop::EventLoop::<event::UserEvent>::with_user_event()
        .build()
        .unwrap();
    
    let proxy = event_loop.create_proxy();

    // 创建Tokio异步运行时
    let rt = tokio::runtime::Runtime::new().unwrap();

    // AIGC 添加
    // 进入运行时上下文，允许在当前线程使用tokio的异步功能
    // _guard是一个RAII守卫，当它被丢弃时会清理运行时上下文
    let _guard = rt.enter();
    
    // 后台任务
    let proxy_clone = proxy.clone();

    let called = Arc::new(Mutex::new(true));
    let called_clone = called.clone();

    

    rt.spawn(async move {
        // loop {
            let proxy_clone_loop = proxy_clone.clone();
            let called_clone_loop = called_clone.clone();
            // 添加变量来跟踪Alt键状态
            let mut last_alt_release = None::<Instant>;
            // 添加冷却期变量
            let mut cooldown_until = None::<Instant>;

            listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        if let Key::Alt = key {
                            // 检查是否在上次Alt释放后的限定秒内
                            let mut should_show = false;
                            {
                                let last_release = last_alt_release;
                                if let Some(time) = last_release {
                                    let elapsed = time.elapsed();
                                    if elapsed <= Duration::from_millis(DOUBLE_ALT_COOLDOWN) {
                                        println!("Double Alt Detected, delay {:?}", elapsed);
                                        should_show = true;
                                    }
                                }
                            }
                            
                            // 如果应该显示，则发送事件
                            if should_show {
                                *called_clone_loop.lock().unwrap() = true;
                                proxy_clone_loop
                                    .send_event(event::UserEvent::ShowWindow)
                                    .unwrap();
                                // 设置冷却期，限定秒内忽略Alt释放
                                cooldown_until = Some(Instant::now() + Duration::from_millis(DOUBLE_ALT_COOLDOWN));
                            }
                        } else {
                            // println!("其他键释放");
                            last_alt_release = None;
                        }
                    },
                    EventType::KeyRelease(key) => {
                        if let Key::Alt = key {
                            // 检查是否在冷却期内
                            let now = Instant::now();
                            if let Some(cooldown_time) = cooldown_until {
                                if now < cooldown_time {
                                    // 在冷却期内，忽略这次释放
                                    return;
                                }
                            }
                            // 不在冷却期内，记录Alt键释放的时间
                            last_alt_release = Some(now);
                        } else {
                            // println!("其他键释放");
                            last_alt_release = None;
                        }
                    },
                    _ => (),
                }
            }).unwrap()
            ;
        // }
    });


    let winit_window_builder = winit::window::WindowAttributes::default()
        .with_resizable(false)
        .with_visible(false)
        .with_inner_size(winit::dpi::LogicalSize {
            width: 800.0,
            height: 500.0,
        })
        .with_title("BaroBoard 工具箱") // 参见 https://github.com/emilk/egui/pull/2279
        // .with_visible(false)
        ;

    let proxy_clone_tray = proxy.clone();
    let called_clone_tray = called.clone();

    // 创建托盘图标
    let mut tray = TrayItem::new("Tray Example", IconSource::RawIcon(4545)).unwrap();
    
    tray.add_menu_item("显示工具箱", move || {
        *called_clone_tray.lock().unwrap() = true;
        proxy_clone_tray
            .send_event(event::UserEvent::ShowWindow)
            .unwrap();
    }).unwrap();

    tray.add_menu_item("退出", || {
        std::process::exit(0);
    }).unwrap();

    // 创建主应用程序
    let proxy_clone_app = proxy.clone();
    let mut app = glow_app::GlowApp::new(
        winit_window_builder,
        proxy.clone(),
        Box::new(move |egui_ctx| {
            egui_ctx.send_viewport_cmd(egui::viewport::ViewportCommand::EnableButtons {
                close: true,
                minimized: true,
                maximize: false,
            });
            // 安装图片加载器，允许egui加载和显示图片
            egui_extras::install_image_loaders(egui_ctx);
            // 设置自定义字体，支持中文显示
            setup_custom_fonts(egui_ctx);

            Box::new(MyApp::new(
                called.clone(),
                proxy_clone_app.clone()
            ))
        }),
    );

    // 在这里控制是否在打开程序的时候就显示
    #[cfg(debug_assertions)]
    proxy.send_event(event::UserEvent::ShowWindow).unwrap();
    // 启动事件循环，这通常是阻塞的，会一直运行直到应用程序关闭
    // 事件循环会不断处理输入事件、UI更新和渲染，这是GUI应用程序的主要执行模式
    event_loop.run_app(&mut app).expect("failed to run app");
}


//自定义字体
fn setup_custom_fonts(ctx: &egui::Context) {
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

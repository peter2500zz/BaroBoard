#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在Windows的发布版本中隐藏控制台窗口

mod my_structs;
mod pages;
mod resources;
mod window;
mod utils;
mod texture_mgr;
mod logging;

use std::sync::{Arc, Mutex};  // Arc = 原子引用计数(Atomically Reference Counted)，一种线程安全的智能指针，允许在多个线程间共享所有权
use egui_winit::winit;
use rdev::{listen, EventType, Key};
use std::time::{Duration, Instant};
use trayicon;
use single_instance::SingleInstance;
use log::{info, warn, debug, trace};

use window::{event, glow_app};
use my_structs::MyApp;
use logging::init_logger;


pub const WINDOW_SIZE: (f32, f32) = (800.0, 500.0);
pub const PROGRAM_VERSION: &str = "v0.1.4-alpha.01";
pub const CONFIG_FILE_VERSION: u32 = 6;
pub const CONFIG_SAVE_PATH: &str = ".baro";
pub const CONFIG_FILE_NAME: &str = "links.json";
pub const DOUBLE_ALT_COOLDOWN: u64 = 500;


fn main() {
    init_logger();
    info!("BaroBoard 工具箱 {} 开始运行", PROGRAM_VERSION);

    let instance = SingleInstance::new("BaroBoard").unwrap();
    
    if !instance.is_single() {
        warn!("BaroBoard 已经在运行，将不会启动新的实例");
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

    // 是否允许双击呼出
    let all_by_double_alt = Arc::new(Mutex::new(true));
    let all_by_double_alt_clone = all_by_double_alt.clone();

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
                            trace!("侦测到Alt键按下");
                            // 检查是否在上次Alt释放后的限定秒内
                            let mut should_show = false;
                            {
                                let last_release = last_alt_release;
                                if let Some(time) = last_release {
                                    let elapsed = time.elapsed();
                                    if elapsed <= Duration::from_millis(DOUBLE_ALT_COOLDOWN) {
                                        debug!("侦测到双击Alt键，两次之间间隔 {:?}", elapsed);
                                        should_show = true;
                                    }
                                }
                            }
                            
                            // 如果应该显示，则发送事件
                            if should_show && *all_by_double_alt_clone.lock().unwrap() {
                                *called_clone_loop.lock().unwrap() = true;
                                proxy_clone_loop
                                    .send_event(event::UserEvent::ShowWindow)
                                    .unwrap();
                                // 设置冷却期，限定秒内忽略Alt释放
                                cooldown_until = Some(Instant::now() + Duration::from_millis(DOUBLE_ALT_COOLDOWN));
                            }
                        } else {
                            // debug!("其他键释放");
                            last_alt_release = None;
                        }
                    },
                    EventType::KeyRelease(key) => {
                        if let Key::Alt = key {
                        trace!("侦测到Alt键释放");
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
                            // debug!("其他键释放");
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
            width: WINDOW_SIZE.0,
            height: WINDOW_SIZE.1,
        })
        .with_title("BaroBoard 工具箱") // 参见 https://github.com/emilk/egui/pull/2279
        .with_window_icon({
            let rgba = image::load_from_memory(resources::LOGO_ICO).unwrap().to_rgba8();
            let (width, height) = rgba.dimensions();
            let rgba_data = rgba.into_raw();
            Some(winit::window::Icon::from_rgba(rgba_data, width, height).unwrap())
        })
        // .with_visible(false)
        ;

    let proxy_clone_tray = proxy.clone();

    // 创建托盘图标
    let tray_icon = trayicon::TrayIconBuilder::new()
    .sender(move |e: &event::UserEvent| {
        let _ = proxy_clone_tray.send_event(e.clone());
    })
    .icon_from_buffer(resources::LOGO_ICO)
    .tooltip("BaroBoard 工具箱")

    .on_click(event::UserEvent::LeftClickTrayIcon)
    .on_right_click(event::UserEvent::RightClickTrayIcon)

    .menu(
        trayicon::MenuBuilder::new()
        .item("显示工具箱", event::UserEvent::ShowWindow)
        .checkable("双击呼出", *all_by_double_alt.lock().unwrap(), event::UserEvent::ChangeDoubleAlt)
        .item("退出", event::UserEvent::Exit)
    )

    .build()
    .unwrap();

    // 创建主应用程序
    let proxy_clone_app = proxy.clone();
    let mut app = glow_app::GlowApp::new(
        all_by_double_alt,
        winit_window_builder,
        tray_icon,
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
            resources::setup_custom_fonts(egui_ctx);

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

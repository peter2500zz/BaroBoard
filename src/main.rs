#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在Windows的发布版本中隐藏控制台窗口

mod my_structs;
mod pages;
mod resources;
mod window;
mod utils;
mod texture_mgr;
mod logging;

use std::sync::{Arc, Mutex};
use egui_winit::winit;
use rdev::{listen, EventType, Key};
use winit::event_loop::EventLoopProxy;
use std::time::{Duration, Instant};
use trayicon;
use single_instance::SingleInstance;
use log::{info, warn, debug, trace};

use window::{event, glow_app};
use my_structs::MyApp;
use logging::init_logger;

use crate::window::event::UserEvent;


pub const WINDOW_SIZE: (f32, f32) = (800.0, 500.0);
pub const PROGRAM_VERSION: &str = "v0.1.4-alpha.05";
pub const CONFIG_FILE_VERSION: u32 = 7;
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
    let called = Arc::new(Mutex::new(true));

    // 是否允许双击呼出
    let all_by_double_alt = Arc::new(Mutex::new(true));

    rt.spawn(double_tap_call(proxy.clone(), Arc::clone(&called), Arc::clone(&all_by_double_alt)));


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


async fn double_tap_call(
    proxy: EventLoopProxy<UserEvent>,
    called: Arc<Mutex<bool>>,
    all_by_double_alt_clone: Arc<Mutex<bool>>,
) {
    // 记录“序列第一下”的按下时间：从这个时间到“第二下的释放”之间若 <= 窗口则触发
    let mut first_alt_press_at = None::<Instant>;
    // 已完成的 tap 数（一次 tap=按下+释放），用于确认到“第二次释放”才触发
    let mut completed_taps: u8 = 0;

    // 冷却期：触发后在窗口期内忽略 Alt 释放，避免连环触发
    let mut cooldown_until = None::<Instant>;

    // 当前 Alt 是否处于按下状态，防止重复 KeyPress
    let mut alt_down = false;

    listen(move |event| {
        match event.event_type {
            EventType::KeyPress(key) => {
                if let Key::Alt = key {
                    if alt_down {
                        // 防止某些平台重复 KeyPress
                        return;
                    }
                    alt_down = true;
                    trace!("侦测到Alt键按下");

                    // 冷却期内直接忽略
                    if let Some(t) = cooldown_until {
                        if Instant::now() < t {
                            return;
                        }
                    }

                    let now = Instant::now();
                    match first_alt_press_at {
                        None => {
                            // 作为序列第一下的按下
                            first_alt_press_at = Some(now);
                            completed_taps = 0;
                            debug!("记录第一下按下时间: {:?}", now);
                        }
                        Some(t0) => {
                            // 若从第一下到现在已超窗，则以此按下作为新序列的第一下
                            if now.duration_since(t0) > Duration::from_millis(DOUBLE_ALT_COOLDOWN) {
                                first_alt_press_at = Some(now);
                                completed_taps = 0;
                                debug!("窗口过期，重置为新序列第一下: {:?}", now);
                            }
                        }
                    }
                } else {
                    // 其他键是否打断序列：维持你原来的“打断”语义
                    first_alt_press_at = None;
                    completed_taps = 0;
                    alt_down = false;
                }
            }

            EventType::KeyRelease(key) => {
                if let Key::Alt = key {
                    trace!("侦测到Alt键释放");

                    // 如果之前没有对应的按下，忽略
                    if !alt_down {
                        return;
                    }
                    alt_down = false;

                    // 冷却期内忽略释放
                    let now = Instant::now();
                    if let Some(t) = cooldown_until {
                        if now < t {
                            return;
                        }
                    }

                    // 必须有“第一下按下”的时间点
                    if let Some(t0) = first_alt_press_at {
                        // 若超出窗口，序列失效，等待下一次按下开启新序列
                        let elapsed = now.duration_since(t0);
                        if elapsed > Duration::from_millis(DOUBLE_ALT_COOLDOWN) {
                            debug!("第一下到当前释放已超窗: {:?}", elapsed);
                            first_alt_press_at = None;
                            completed_taps = 0;
                            return;
                        }

                        // 每次释放都算完成一个 tap
                        completed_taps = completed_taps.saturating_add(1);

                        if completed_taps >= 2 && *all_by_double_alt_clone.lock().unwrap() {
                            debug!(
                                "侦测到双击Alt：第一下按下到第二次释放间隔 {:?}",
                                elapsed
                            );
                            *called.lock().unwrap() = true;
                            let _ = proxy
                                .send_event(event::UserEvent::ShowWindow);

                            // 进入冷却期，避免连环触发
                            cooldown_until = Some(
                                Instant::now() + Duration::from_millis(DOUBLE_ALT_COOLDOWN),
                            );

                            // 复位状态机
                            first_alt_press_at = None;
                            completed_taps = 0;
                            alt_down = false;
                        } else {
                            // 第一次释放，等待第二次
                            trace!("完成第 {completed_taps} 次 tap，等待下一次释放");
                        }
                    } else {
                        // 没有起始按下，不计
                        debug!("没有记录第一下按下时间，忽略此次释放");
                    }
                } else {
                    // 其他键释放打断
                    first_alt_press_at = None;
                    completed_taps = 0;
                    alt_down = false;
                }
            }

            _ => {}
        }
    })
    .unwrap();
}


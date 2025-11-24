pub mod event;
pub mod glow_app;

use std::num::NonZeroU32;
use glutin::{config::ConfigTemplateBuilder, context::ContextAttributesBuilder};
use glutin_winit::{ApiPreference, DisplayBuilder};
use windows::Win32::Foundation::HWND;
use winit::raw_window_handle::HasWindowHandle;
use log::debug;

pub trait App {
    // 初始化
    fn init(&mut self, hwnd: Option<HWND>);

    // 更新
    fn update(&mut self, ctx: &egui::Context);

    // 文件悬浮
    fn on_file_hovered(&mut self, path: String);

    // 文件悬浮取消
    fn on_file_hover_cancelled(&mut self);

    // 文件释放
    fn on_file_dropped(&mut self, path: String);
}


pub struct GlutinWindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    // refactor this function to use `glutin-winit` crate eventually.
    // preferably add android support at the same time.
    // 最终重构此函数以使用`glutin-winit`包。
    // 同时最好添加Android支持。
    // #[allow(unsafe_code)]
    pub fn new(
        // winit 事件循环的引用。事件循环负责监听和分发操作系统事件（如键盘输入、鼠标移动、窗口关闭等）。
        event_loop: &winit::event_loop::ActiveEventLoop,
        // winit 窗口的配置属性，例如窗口的初始大小、标题、是否全屏等。
        winit_window_builder: winit::window::WindowAttributes,
    ) -> Self {
        // --- 导入必要的 Trait ---
        // 这些 Trait 提供了在后续代码中使用的 glutin 核心方法和类型。
        use glutin::context::NotCurrentGlContext;
        use glutin::display::GetGlDisplay;
        use glutin::display::GlDisplay;
        use glutin::prelude::GlSurface;

        // --- 1. 配置 OpenGL 渲染参数（Config Template） ---
        let config_template_builder = ConfigTemplateBuilder::new()
            // 尝试使用硬件加速。None 表示由驱动程序决定是否优先使用硬件加速。
            .prefer_hardware_accelerated(None)
            // 设置深度缓冲区大小为 0。深度缓冲区用于3D渲染中判断哪些物体可见。这里设为 0 表示不使用或不需要深度测试。
            .with_depth_size(0)
            // 设置模板缓冲区大小为 0。模板缓冲区用于更高级的渲染技术（如阴影）。
            .with_stencil_size(0)
            // 设置窗口为不透明。如果设置为 true，则可以创建透明窗口。
            .with_transparency(false);
            

        debug!("trying to get gl_config"); // 调试信息：尝试获取合适的 OpenGL 配置。

        // --- 2. 创建窗口和选择 OpenGL 配置 (GlConfig) ---
        let (mut window, gl_config) =
            // DisplayBuilder 是 glutin-winit 提供的工具，用于简化创建窗口和选择 OpenGL/WebGL 配置的复杂过程。
            DisplayBuilder::new() 
                // 让glutin-winit辅助包处理OpenGL上下文创建的复杂部分
                // 强制使用 EGL 作为后备 API。EGL 是一种在嵌入式系统和部分桌面系统上管理 OpenGL 上下文的跨平台 API。
                .with_preference(ApiPreference::FallbackEgl) 
                // https://github.com/emilk/egui/issues/2520#issuecomment-1367841150
                // 传入之前定义的窗口属性，以便在选择 GlConfig 时创建或准备创建窗口。
                .with_window_attributes(Some(winit_window_builder.clone()))
                .build(
                    event_loop,
                    config_template_builder,
                    // 这是一个闭包（回调函数），用于在找到多个兼容配置时，选择其中一个。
                    // .next() 选择第一个找到的配置。
                    |mut config_iterator| {
                        config_iterator.next().expect(
                            "failed to find a matching configuration for creating glutin config",
                        )
                    },
                )
                .expect("failed to create gl_config"); // 如果找不到任何 GlConfig，程序会崩溃并打印此错误。

        let gl_display = gl_config.display(); // 从 GlConfig 中获取 GlDisplay，代表底层图形设备/驱动。
        debug!("found gl_config: {:?}", &gl_config); // 调试信息：找到了 GlConfig。

        // --- 3. 准备创建 OpenGL 上下文 ---
        let raw_window_handle = window.as_ref().map(|w| {
            // 获取底层操作系统的窗口句柄/标识符。
            // 这是跨平台图形编程中连接 Winit 窗口和 Glutin 上下文的关键一步。
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw() // 获取原始的、平台特定的句柄。
        });
        debug!("raw window handle: {:?}", raw_window_handle);

        // 构建核心 OpenGL 上下文的属性。这是现代桌面 OpenGL 的推荐方式。
        let context_attributes =
            ContextAttributesBuilder::new().build(raw_window_handle);
            
        // by default, glutin will try to create a core opengl context. but, if it is not available, try to create a gl-es context using this fallback attributes
        // 默认情况下，glutin会尝试创建一个核心OpenGL上下文。但如果不可用，则尝试使用这个后备属性创建gl-es上下文
        let fallback_context_attributes = ContextAttributesBuilder::new()
            // 明确指定使用 OpenGL ES 上下文，版本号由驱动程序自行选择。
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);

        // --- 4. 创建 OpenGL 上下文（带回退机制） ---
        let not_current_gl_context = unsafe { // 此操作涉及与底层图形驱动程序交互，因此被标记为不安全 (unsafe)。
            gl_display
                // 尝试用核心 OpenGL 属性创建上下文
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    // 如果核心上下文创建失败，执行此回退逻辑
                    debug!("failed to create gl_context with attributes: {:?}. retrying with fallback context attributes: {:?}",
                        &context_attributes,
                        &fallback_context_attributes);
                    gl_config
                        .display()
                        // 尝试用 OpenGL ES (GLES) 属性创建上下文
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context even with fallback attributes") // 如果 GLES 上下文也失败，程序会崩溃。
                })
        };

        // --- 5. 确保窗口已创建 (Winit Window) ---
        // this is where the window is created, if it has not been created while searching for suitable gl_config
        // 这里是创建窗口的地方，如果在搜索合适的gl_config时还没有创建窗口
        let window = window.take().unwrap_or_else(|| {
            debug!("window doesn't exist yet. creating one now with finalize_window");
            // 使用 glutin-winit 的辅助函数来完成 winit 窗口的创建和配置。
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });

        // --- 6. 计算和规范化窗口尺寸 ---
        // 获取窗口的宽度和高度，用于创建渲染表面。
        let (width, height): (u32, u32) = match winit_window_builder.inner_size {
            Some(size) => size.to_physical::<u32>(1.0).into(), // 使用 builder 中的尺寸
            None => window.inner_size().into(),                 // 使用实际创建窗口的尺寸
        };

        // 确保宽度和高度至少为 1（NonZeroU32），因为图形 API 不允许创建尺寸为 0 的表面。
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);

        // --- 7. 创建渲染表面 (GlSurface) 的属性 ---
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );
        debug!(
            "creating surface with attributes: {:?}",
            &surface_attributes
        );

        // --- 8. 创建并激活渲染表面 ---
        let gl_surface = unsafe { // 同样是与底层API交互的不安全操作。
            gl_display
                // 根据 GlConfig 和 SurfaceAttributes 在窗口上创建实际的渲染表面。
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        debug!("surface created successfully: {gl_surface:?}.making context current");

        // 将之前创建的“非当前”上下文与渲染表面关联，使其成为“当前”上下文。
        // 只有当前上下文才能执行 OpenGL 渲染命令。
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        // --- 9. 设置垂直同步 (VSync) ---
        gl_surface
            .set_swap_interval(
                &gl_context,
                // 设置交换间隔为等待（Wait）。NonZeroU32::MIN 通常是 1，表示等待 1 个垂直消隐周期。
                // 简而言之：启用 VSync (垂直同步)，防止画面撕裂 (Tearing)。
                glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
            )
            .unwrap();

        // --- 10. 返回 GlutinWindowContext 结构体 ---
        Self {
            window,         // winit 窗口实例
            gl_context,     // 当前活动的 OpenGL 上下文
            gl_display,     // 底层图形设备/驱动的抽象
            gl_surface,     // 窗口上的渲染表面
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        if physical_size.width == 0 || physical_size.height == 0 {
            return;
        }
        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;
        self.gl_display.get_proc_address(addr)
    }
}

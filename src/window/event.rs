// !NOTICE: user event
// !注意: 用户事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Redraw(std::time::Duration),
    HideWindow,
    ShowWindow,
    #[cfg(target_os = "windows")]
    Exit,

    // 托盘相关
    #[cfg(target_os = "windows")]
    LeftClickTrayIcon,
    #[cfg(target_os = "windows")]
    RightClickTrayIcon,
    #[cfg(target_os = "windows")]
    ChangeDoubleAlt,

    // 文件相关
    FileHovered(String),
    FileHoverCancelled,
    FileDropped(String),
}

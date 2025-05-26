// !NOTICE: user event
// !注意: 用户事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Redraw(std::time::Duration),
    HideWindow,
    ShowWindow,
    Exit,

    // 托盘相关
    LeftClickTrayIcon,
    RightClickTrayIcon,
}

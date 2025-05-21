// !NOTICE: user event
// !注意: 用户事件
#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
    HideWindow,
    ShowWindow,
}

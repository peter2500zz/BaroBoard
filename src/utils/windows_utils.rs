use systemicons::get_icon;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{GetLastError, HWND}, 
        UI::{
            Shell::{
                SEE_MASK_FLAG_LOG_USAGE, 
                SEE_MASK_FLAG_NO_UI, 
                SEE_MASK_INVOKEIDLIST, 
                SEE_MASK_NOASYNC, 
                SHELLEXECUTEINFOW, 
                ShellExecuteExW
            }, 
            WindowsAndMessaging::{
                MB_ICONERROR, 
                MB_OK, 
                MessageBoxW, 
                SW_SHOWNORMAL
            }
        }
    },
    core::{
        PCWSTR, w
    },
};

pub fn get_icon_from_exe(exe_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let sizes = [1024, 512, 256, 128, 96, 64, 48, 32, 24, 16];

    for size in sizes {
        if let Ok(icon) = get_icon(exe_path, size) {
            return Ok(icon);
        }
    }

    Err("无法获取图标".into())
}

pub fn create_process(hwnd: HWND, app: &str, work_dir: &str, args: &str, admin: bool) -> Result<(), Box<dyn std::error::Error>> {
    let app16 = U16CString::from_str(app)?;
    let wd16 = U16CString::from_str(work_dir)?;
    let args16 = U16CString::from_str(args)?;
    // 我讨厌匈牙利命名法
    let lpapp = PCWSTR(app16.as_ptr());
    let lpwd = PCWSTR(wd16.as_ptr());
    let lpargs = PCWSTR(args16.as_ptr());

    let mut sei = SHELLEXECUTEINFOW::default();
    sei.cbSize = size_of_val(&sei) as u32;
    sei.hwnd = hwnd;
    sei.fMask =
        SEE_MASK_INVOKEIDLIST |   // 告诉 shell：用 IContextMenu/动词系统
        SEE_MASK_NOASYNC |        // Explorer 不希望异步 DDE 之类搞太久
        SEE_MASK_FLAG_NO_UI |     // 不想弹错误对话框（有的路径下会加）
        SEE_MASK_FLAG_LOG_USAGE;  // 让 shell 记录“常用程序”之类
    sei.nShow = SW_SHOWNORMAL.0;

    sei.lpFile = lpapp;
    sei.lpVerb = if admin { w!("runas") } else { w!("open") };
    sei.lpParameters = lpargs;
    sei.lpDirectory = lpwd;

    unsafe {
        let result = ShellExecuteExW(&mut sei);

        if let Err(e) = result {
            let text = U16CString::from_str(format!("{}({}) {:?}", e.message(), e.code(), GetLastError())).unwrap();
            MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                PCWSTR::null(),
                MB_ICONERROR | MB_OK,
            );
        }
    }

    Ok(())
}

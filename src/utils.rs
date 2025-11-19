use std::mem;

use image::RgbaImage;
use systemicons::get_icon;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{GetLastError, HWND}, Graphics::Gdi::{BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetBitmapBits, GetDC, GetDIBits, GetObjectW, HBITMAP, HDC, HGDIOBJ, ReleaseDC}, UI::{
            Shell::{
                ExtractIconExW,
                SEE_MASK_FLAG_LOG_USAGE, 
                SEE_MASK_FLAG_NO_UI, 
                SEE_MASK_INVOKEIDLIST, 
                SEE_MASK_NOASYNC, 
                SHELLEXECUTEINFOW, 
                ShellExecuteExW
            }, 
            WindowsAndMessaging::{
                GetIconInfoExW, HICON, ICONINFO, ICONINFOEXW, MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWNORMAL
            }
        }
    },
    core::{
        PCWSTR, w
    },
};


#[test]
fn g() {
    unsafe {

        let hmax = get_hicon_from_exe(r"D:\Files\Rust\hexphage\target\hexphage-0.1.0-windows-msvc-x64.exe")[0];

        println!("{:?}", hmax);

        let mut info = ICONINFOEXW::default();
        info.cbSize = size_of_val(&info) as _;
        let _ = GetIconInfoExW(hmax, &mut info);
        let mut bm = BITMAP::default();
        GetObjectW(HGDIOBJ(info.hbmColor.0), size_of_val(&bm) as i32, Some(&raw mut bm as _));

        let rgba = convert_icon_to_image(hmax);

        std::fs::create_dir_all(format!("{}/cache/exe_icon", crate::CONFIG_SAVE_PATH)).unwrap();

        let icon_path = format!("{}/cache/exe_icon/gggg.png", crate::CONFIG_SAVE_PATH);
        rgba.save(icon_path);
        // if !std::path::Path::new(&icon_path).exists() {
        //     std::fs::write(icon_path.clone(), rgba.as_ref()).unwrap();
        // }
    }
}


fn get_hicon_from_exe(path: &str) -> Vec<HICON> {
    let u16path = U16CString::from_str(path).unwrap();

    unsafe {
        let count = ExtractIconExW(
            PCWSTR(u16path.as_ptr()), 
            -1,
            None,
            None, 
            0
        );

        let mut hicons: Vec<HICON> = Vec::new();

        for index in 0..count {
            let mut hicon = HICON::default();
            let getted = ExtractIconExW(
                PCWSTR(u16path.as_ptr()), 
                index as i32,
                Some(&mut hicon),
                None, 
                1
            );
            println!("get {}", getted);
            if getted > 0 {
                hicons.push(hicon);
            }
        }

        hicons
    }
}

/// By [liaohp](https://users.rust-lang.org/u/liaohp/summary)
/// 
/// https://users.rust-lang.org/t/how-to-convert-hicon-to-png/90975/15
/// 
/// Modified
unsafe fn convert_icon_to_image(icon: HICON) -> RgbaImage {
    // 1. 取 ICONINFO
    let mut info = ICONINFOEXW::default();
    info.cbSize = size_of_val(&info) as _;
    let ok = GetIconInfoExW(icon, &mut info);
    assert!(ok.as_bool()); // 或者自己处理错误

    let mut bitmap = BITMAP::default();
    let result = GetObjectW(
        HGDIOBJ(info.hbmColor.0),
        size_of::<BITMAP>() as i32,
        Some(&raw mut bitmap as _),
    );
    assert!(result == size_of::<BITMAP>() as i32);

    let width = bitmap.bmWidth;
    let height = bitmap.bmHeight;

    let buf_size = (width * height * 4) as usize;
    let mut pixels = vec![0u8; buf_size];

    let dc = GetDC(None);
    assert!(dc != HDC::default());

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as _,
            biWidth: width,
            biHeight: -height.abs(), // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let scanlines = GetDIBits(
        dc,
        info.hbmColor,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    assert!(scanlines != 0);

    ReleaseDC(None, dc);
    DeleteObject(HGDIOBJ(info.hbmColor.0));
    DeleteObject(HGDIOBJ(info.hbmMask.0));

    // BGRA -> RGBA
    for p in pixels.chunks_exact_mut(4) {
        p.swap(0, 2); // B <-> R
    }

    RgbaImage::from_vec(width as u32, height as u32, pixels).unwrap()
}


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

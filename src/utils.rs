use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{
            GetLastError, 
            HWND
        }, 
        Graphics::Gdi::{
            BI_RGB, 
            BITMAP, 
            BITMAPINFO, 
            BITMAPINFOHEADER, 
            DIB_RGB_COLORS, 
            DeleteObject, 
            GetDC, 
            GetDIBits, 
            GetObjectW, 
            HDC, 
            HGDIOBJ, 
            ReleaseDC
        }, 
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES, 
        UI::{
            Controls::{
                IImageList, 
                ILD_TRANSPARENT
            }, 
            Shell::{
                SEE_MASK_FLAG_LOG_USAGE, 
                SEE_MASK_FLAG_NO_UI, 
                SEE_MASK_INVOKEIDLIST, 
                SEE_MASK_NOASYNC, 
                SHELLEXECUTEINFOW, 
                SHFILEINFOW, 
                SHGFI_SYSICONINDEX, 
                SHGetFileInfoW, 
                SHGetImageList, 
                SHIL_JUMBO, 
                ShellExecuteExW
            }, 
            WindowsAndMessaging::{
                DestroyIcon, 
                GetIconInfoExW, 
                HICON, 
                ICONINFOEXW, 
                MB_ICONERROR, 
                MB_OK, 
                MessageBoxW, 
                SW_SHOWNORMAL
            }
        }
    },
    core::{
        PCWSTR, 
        Result as WinResult, 
        w
    },
};


pub fn get_icon_from_exe(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unsafe {
        let hicon = get_hicon_from_shell(path)
            .unwrap_or(get_windows_default_icon());

        let mut info = ICONINFOEXW::default();
        info.cbSize = size_of_val(&info) as _;

        let _ = GetIconInfoExW(hicon, &mut info);
        let mut bm = BITMAP::default();
        GetObjectW(HGDIOBJ(info.hbmColor.0), size_of_val(&bm) as i32, Some(&raw mut bm as _));

        let rgba = convert_icon_to_image(hicon)
            .unwrap_or(invalid_icon());

        DestroyIcon(hicon)?;

        let mut buf = Cursor::new(Vec::new());
        rgba.write_to(&mut buf, ImageFormat::Png)?;

        Ok(buf.into_inner())
    }
}

pub fn get_invalid_icon() -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    invalid_icon().write_to(&mut buf, ImageFormat::Png).unwrap();

    buf.into_inner()
}


fn invalid_icon() -> RgbaImage {
    let width = 256u32;
    let height = 256u32;

    let mut img = RgbaImage::new(width, height);

    let black = Rgba([0x00, 0x00, 0x00, 0xFF]);       // #000000
    let purple = Rgba([0xF8, 0x00, 0xF8, 0xFF]);      // #F800F8

    for y in 0..height {
        for x in 0..width {
            let top = y < width / 2;
            let left = x < height / 2;

            let color = if (top && left) || (!top && !left) {
                black
            } else {
                purple
            };

            img.put_pixel(x, y, color);
        }
    }

    img
}

fn get_windows_default_icon() -> HICON {
    unsafe {
        let iml = SHGetImageList::<IImageList>(SHIL_JUMBO as _).unwrap();

        let hicon = iml.GetIcon(0, ILD_TRANSPARENT.0).unwrap();

        hicon
    }
}

fn get_hicon_from_shell(path: &str) -> WinResult<HICON> {
    let u16path = U16CString::from_str(path).unwrap();

    unsafe {
        let mut sfi = SHFILEINFOW::default();

        SHGetFileInfoW(
            PCWSTR(u16path.as_ptr()), 
            FILE_FLAGS_AND_ATTRIBUTES(0), 
            Some(&mut sfi), 
            size_of_val(&sfi) as u32, 
            SHGFI_SYSICONINDEX
        );

        let iml = SHGetImageList::<IImageList>(SHIL_JUMBO as _)?;

        let hicon = iml.GetIcon(sfi.iIcon, ILD_TRANSPARENT.0)?;

        Ok(hicon)
    }
}


/// By [liaohp](https://users.rust-lang.org/u/liaohp/summary)
/// 
/// https://users.rust-lang.org/t/how-to-convert-hicon-to-png/90975/15
/// 
/// Modified
fn convert_icon_to_image(icon: HICON) -> Option<RgbaImage> {
    unsafe {
        let mut info = ICONINFOEXW::default();
        info.cbSize = size_of_val(&info) as _;

        if !GetIconInfoExW(icon, &mut info).as_bool() {
            return None;
        }

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
                biHeight: -height.abs(),
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
        
        if !DeleteObject(HGDIOBJ(info.hbmColor.0)).as_bool()
        || !DeleteObject(HGDIOBJ(info.hbmMask.0)).as_bool() {
            return None;
        }

        for p in pixels.chunks_exact_mut(4) {
            p.swap(0, 2);
        }

        let sizes = [16, 24, 32, 48, 64, 72, 96, 128, 192, 256];

        if let Some(min_size) = find_min_size(&pixels, width, height, &sizes) {
            println!("Resizing to: {}", min_size);

            let cropped_pixels: Vec<u8> = pixels
                .chunks(4)
                .enumerate()
                .filter(|(index, _)| {
                    let y = (*index as i32) / width;
                    let x = (*index as i32) % width;
                    x < min_size && y < min_size
                })
                .map(|(_, pixel_chunk)| pixel_chunk)
                .flatten()
                .copied()
                .collect();

            return RgbaImage::from_vec(min_size as u32, min_size as u32, cropped_pixels);
        }

        RgbaImage::from_vec(width as u32, height as u32, pixels)
    }
}


/// By Gemini
fn find_min_size(pixels: &[u8], width: i32, _: i32, sizes: &[i32]) -> Option<i32> {
    let mut max_x = -1;
    let mut max_y = -1;

    // 1. 只需要遍历一次所有像素，找出非透明像素的最大 x 和 y
    // chunks(4) 对应 [R, G, B, A]
    for (i, chunk) in pixels.chunks(4).enumerate() {
        // 检查 Alpha 通道 (chunk[3])
        if chunk[3] != 0 {
            let idx = i as i32;
            let y = idx / width;
            let x = idx % width;

            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
        }
    }

    // 如果全是透明的，可能返回最小尺寸或者 None，视需求而定
    if max_x == -1 {
        return None; // 或者 return None
    }

    // 2. 计算所需的最小宽高 (坐标是从0开始的，所以大小需要+1)
    let required_w = max_x + 1;
    let required_h = max_y + 1;
    let required_size = required_w.max(required_h);

    // 3. 在 sizes 数组中从小到大找到第一个满足要求的尺寸
    // 这里假设 sizes 已经是升序排列的 [16, 24, ...]
    for &size in sizes {
        if size >= required_size {
            return Some(size);
        }
    }

    // 如果都不满足（比如内容超出了 256），返回 None 或最大值
    None
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

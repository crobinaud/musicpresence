use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    ExtractIconW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    KillTimer, LoadIconW, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
    SetForegroundWindow, SetTimer, SetWindowLongPtrW, TrackPopupMenu, GWLP_USERDATA, HICON,
    ICONINFO, IDI_APPLICATION, MF_STRING, MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WM_CONTEXTMENU, WM_DESTROY,
    WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_TIMER, WM_USER,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = WM_USER + 100;
const WM_UPDATE_TRAY_STATUS: u32 = WM_USER + 101;

const ID_MENU_PRESENCE: usize = 1001;
const ID_MENU_QUIT: usize = 1002;

struct TrayContext {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    status_text: Arc<Mutex<String>>,
    should_exit: Arc<AtomicBool>,
    custom_icon: Option<HICON>,
    taskbar_created_msg: u32,
    is_added: bool,
}

pub struct TrayIcon {
    pub should_exit: Arc<AtomicBool>,
    status_text: Arc<Mutex<String>>,
    hwnd: Arc<AtomicIsize>,
}

impl TrayIcon {
    pub fn new() -> Self {
        let should_exit = Arc::new(AtomicBool::new(false));
        let status_text = Arc::new(Mutex::new("Waiting for Apple Music...".to_string()));
        let hwnd_holder = Arc::new(AtomicIsize::new(0));

        let exit_clone = should_exit.clone();
        let status_clone = status_text.clone();
        let hwnd_clone = hwnd_holder.clone();

        thread::spawn(move || {
            run_tray_thread(exit_clone, status_clone, hwnd_clone);
        });

        Self {
            should_exit,
            status_text,
            hwnd: hwnd_holder,
        }
    }

    pub fn update_status(&self, text: &str) {
        if let Ok(mut st) = self.status_text.lock() {
            *st = text.to_string();
        }

        let h = self.hwnd.load(Ordering::SeqCst);
        if h != 0 {
            let hwnd = HWND(h as *mut _);
            unsafe {
                let _ = PostMessageW(hwnd, WM_UPDATE_TRAY_STATUS, WPARAM(0), LPARAM(0));
            }
        }
    }

    pub fn is_exit_requested(&self) -> bool {
        self.should_exit.load(Ordering::SeqCst)
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.should_exit.store(true, Ordering::SeqCst);
        let h = self.hwnd.load(Ordering::SeqCst);
        if h != 0 {
            let hwnd = HWND(h as *mut _);
            unsafe {
                let _ = PostMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
            }
        }
    }
}

fn run_tray_thread(
    should_exit: Arc<AtomicBool>,
    status_text: Arc<Mutex<String>>,
    hwnd_out: Arc<AtomicIsize>,
) {
    let class_name: Vec<u16> = OsStr::new("AppleMusicPresenceTrayClass")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let _ = RegisterClassW(&wnd_class);

        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            hinstance,
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                crate::log_status(&format!("[TRAY_ERROR] CreateWindowExW failed: {:?}", e));
                return;
            }
        };

        hwnd_out.store(hwnd.0 as isize, Ordering::SeqCst);

        // Retrieve icon (custom Apple Music icon or fallback to system icon)
        let (hicon, custom_icon) = if let Some(ic) = create_apple_music_icon() {
            (ic, Some(ic))
        } else {
            let extracted = ExtractIconW(HINSTANCE(std::ptr::null_mut()), w!("shell32.dll"), 238);
            if !extracted.is_invalid() {
                (extracted, Some(extracted))
            } else {
                (LoadIconW(None, IDI_APPLICATION).unwrap_or_default(), None)
            }
        };

        let taskbar_created_msg = RegisterWindowMessageW(w!("TaskbarCreated"));

        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: hicon,
            ..Default::default()
        };

        let tip: Vec<u16> = OsStr::new("Apple Music Discord Presence")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let copy_len = tip.len().min(nid.szTip.len() - 1);
        nid.szTip[..copy_len].copy_from_slice(&tip[..copy_len]);

        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        let add_res = Shell_NotifyIconW(NIM_ADD, &nid);
        let is_added = add_res.as_bool();
        crate::log_status(&format!(
            "[TRAY_INIT] NIM_ADD result={:?}, HWND={:?}",
            add_res, hwnd
        ));

        let context = Box::new(TrayContext {
            hwnd,
            nid,
            status_text: status_text.clone(),
            should_exit: should_exit.clone(),
            custom_icon,
            taskbar_created_msg,
            is_added,
        });

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

        // Start periodic timer (1 sec) to retry adding icon if Explorer was not ready
        if !is_added {
            let _ = SetTimer(hwnd, 1, 1000, None);
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);

            if should_exit.load(Ordering::SeqCst) {
                break;
            }
        }

        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayContext;
        if !ptr.is_null() {
            let ctx = Box::from_raw(ptr);
            let _ = KillTimer(hwnd, 1);
            let _ = Shell_NotifyIconW(NIM_DELETE, &ctx.nid);
            if let Some(ic) = ctx.custom_icon {
                let _ = DestroyIcon(ic);
            }
        }
        let _ = DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayContext;
    if !ptr.is_null() {
        let ctx = &mut *ptr;

        if msg == WM_TIMER && wparam.0 == 1 {
            if !ctx.is_added {
                let res = Shell_NotifyIconW(NIM_ADD, &ctx.nid);
                if res.as_bool() {
                    ctx.is_added = true;
                    crate::log_status("[TRAY_INIT] System Tray successfully connected to taskbar!");
                    let _ = KillTimer(hwnd, 1);
                }
            }
            return LRESULT(0);
        } else if ctx.taskbar_created_msg != 0 && msg == ctx.taskbar_created_msg {
            crate::log_status("[TRAY_EVENT] TaskbarCreated received, re-adding icon...");
            let res = Shell_NotifyIconW(NIM_ADD, &ctx.nid);
            ctx.is_added = res.as_bool();
            return LRESULT(0);
        } else if msg == WM_UPDATE_TRAY_STATUS {
            if let Ok(st) = ctx.status_text.lock() {
                let full_tip = format!("Apple Music Presence\n{}", *st);
                let tip_w: Vec<u16> = OsStr::new(&full_tip)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                let copy_l = tip_w.len().min(ctx.nid.szTip.len() - 1);
                ctx.nid.szTip = [0; 128];
                ctx.nid.szTip[..copy_l].copy_from_slice(&tip_w[..copy_l]);
                ctx.nid.uFlags = NIF_TIP | NIF_MESSAGE | NIF_ICON;
                if ctx.is_added {
                    let _ = Shell_NotifyIconW(NIM_MODIFY, &ctx.nid);
                } else {
                    let res = Shell_NotifyIconW(NIM_ADD, &ctx.nid);
                    if res.as_bool() {
                        ctx.is_added = true;
                    }
                }
            }
            return LRESULT(0);
        } else if msg == WM_TRAYICON {
            let event = (lparam.0 as u32) & 0xFFFF;
            if event == WM_RBUTTONUP
                || event == WM_RBUTTONDOWN
                || event == WM_LBUTTONUP
                || event == WM_LBUTTONDBLCLK
                || event == WM_CONTEXTMENU
                || event == 0x0400 // NIN_SELECT
                || event == 0x0401
            // NIN_KEYSELECT
            {
                show_menu(ctx);
            }
            return LRESULT(0);
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn show_menu(ctx: &TrayContext) {
    let menu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return,
    };

    let presence_w: Vec<u16> = OsStr::new("Music Presence")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let quit_w: Vec<u16> = OsStr::new("Quit")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_PRESENCE,
        PCWSTR(presence_w.as_ptr()),
    );
    let _ = AppendMenuW(menu, MF_STRING, ID_MENU_QUIT, PCWSTR(quit_w.as_ptr()));

    let mut cursor_pos = POINT::default();
    let _ = GetCursorPos(&mut cursor_pos);

    let _ = SetForegroundWindow(ctx.hwnd);

    let cmd = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        cursor_pos.x,
        cursor_pos.y,
        0,
        ctx.hwnd,
        None,
    );

    let _ = PostMessageW(ctx.hwnd, WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);

    match cmd.0 as usize {
        ID_MENU_PRESENCE => {
            let _ = Command::new("notepad.exe").arg("config.toml").spawn();
        }
        ID_MENU_QUIT => {
            ctx.should_exit.store(true, Ordering::SeqCst);
            PostQuitMessage(0);
        }
        _ => {}
    }
}

unsafe fn create_apple_music_icon() -> Option<HICON> {
    let width = 32;
    let height = 32;

    let mut color_pixels = vec![0u32; (width * height) as usize];
    let mut mask_pixels = vec![0u8; ((width * height) / 8) as usize];

    let center_x = 15.5;
    let center_y = 15.5;
    let radius = 14.5;
    let rad = -22.0f64.to_radians();
    let cos_t = rad.cos();
    let sin_t = rad.sin();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let fx = x as f64 + 0.5;
            let fy = y as f64 + 0.5;

            let dx = fx - center_x;
            let dy = fy - center_y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= radius * radius {
                let n1_dx = fx - 11.0;
                let n1_dy = fy - 20.5;
                let r1_x = n1_dx * cos_t - n1_dy * sin_t;
                let r1_y = n1_dx * sin_t + n1_dy * cos_t;
                let in_head1 = (r1_x * r1_x) / (3.4 * 3.4) + (r1_y * r1_y) / (2.5 * 2.5) <= 1.0;

                let n2_dx = fx - 19.5;
                let n2_dy = fy - 17.5;
                let r2_x = n2_dx * cos_t - n2_dy * sin_t;
                let r2_y = n2_dx * sin_t + n2_dy * cos_t;
                let in_head2 = (r2_x * r2_x) / (3.4 * 3.4) + (r2_y * r2_y) / (2.5 * 2.5) <= 1.0;

                let in_stem1 = (12.0..=13.8).contains(&fx) && (8.5..=21.0).contains(&fy);
                let in_stem2 = (20.5..=22.3).contains(&fx) && (5.5..=18.0).contains(&fy);
                let beam_top = 8.5 + (fx - 12.0) * (-3.0 / 10.3);
                let in_beam = (12.0..=22.3).contains(&fx) && fy >= beam_top && fy <= beam_top + 2.9;

                if in_head1 || in_head2 || in_stem1 || in_stem2 || in_beam {
                    color_pixels[idx] = 0x00FFFFFF;
                } else {
                    color_pixels[idx] = 0x00443CFC;
                }
            } else {
                let mask_idx = idx / 8;
                let bit_idx = 7 - (idx % 8);
                mask_pixels[mask_idx] |= 1 << bit_idx;
                color_pixels[idx] = 0;
            }
        }
    }

    let hbm_color = CreateBitmap(
        width,
        height,
        1,
        32,
        Some(color_pixels.as_ptr() as *const _),
    );
    let hbm_mask = CreateBitmap(width, height, 1, 1, Some(mask_pixels.as_ptr() as *const _));

    let icon_info = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = CreateIconIndirect(&icon_info).ok();
    let _ = DeleteObject(hbm_color);
    let _ = DeleteObject(hbm_mask);
    hicon
}

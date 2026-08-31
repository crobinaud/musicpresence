use crate::config::Config;
use crate::updater::{self, CURRENT_VERSION};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, SetBkColor, SetBkMode, SetTextColor,
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, FW_BOLD, FW_NORMAL, FW_SEMIBOLD,
    HBRUSH, HDC, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, MessageBoxW,
    PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW,
    SetWindowTextW, ShowWindow, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON,
    BS_GROUPBOX, BS_PUSHBUTTON, ES_NUMBER, GWLP_USERDATA, MB_ICONINFORMATION, MB_ICONQUESTION,
    MB_ICONWARNING, MB_OK, MB_YESNO, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_RESTORE, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG,
    WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY, WM_SETFONT, WNDCLASSW, WS_BORDER, WS_CAPTION,
    WS_CHILD, WS_EX_CLIENTEDGE, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

static SETTINGS_HWND: AtomicIsize = AtomicIsize::new(0);

const BST_UNCHECKED: usize = 0;
const BST_CHECKED: usize = 1;
const SS_LEFT: u32 = 0x00000000;
const IDYES: i32 = 6;

const ID_EDIT_POLL_INTERVAL: usize = 2001;
const ID_CHK_AUTOSTART: usize = 2002;
const ID_CHK_AUTOUPDATE: usize = 2003;
const ID_BTN_CHECK_UPDATE: usize = 2004;
const ID_BTN_SAVE: usize = 2005;
const ID_BTN_CANCEL: usize = 2006;
const ID_BTN_RESET: usize = 2007;

struct SettingsContext {
    hwnd: HWND,
    config: Arc<RwLock<Config>>,
    font_regular: HFONT,
    font_bold: HFONT,
    font_title: HFONT,
    font_badge: HFONT,
    bg_brush: HBRUSH,
    edit_brush: HBRUSH,
    hwnd_poll_interval: HWND,
    hwnd_autostart: HWND,
    hwnd_autoupdate: HWND,
}

pub fn open_settings_window(config: Arc<RwLock<Config>>) {
    let existing_h = SETTINGS_HWND.load(Ordering::SeqCst);
    if existing_h != 0 {
        let hwnd = HWND(existing_h as *mut _);
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
        return;
    }

    thread::spawn(move || {
        run_settings_gui(config);
    });
}

fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn run_settings_gui(config: Arc<RwLock<Config>>) {
    let class_name = to_wstring("AppleMusicPresenceSettingsWindow");

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let bg_color = COLORREF(0x00F5F5F5);
        let edit_bg = COLORREF(0x00FFFFFF);
        let bg_brush = CreateSolidBrush(bg_color);
        let edit_brush = CreateSolidBrush(edit_bg);

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(settings_wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: bg_brush,
            ..Default::default()
        };

        let _ = RegisterClassW(&wnd_class);

        let win_width = 430;
        let win_height = 385;

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - win_width) / 2;
        let y = (screen_h - win_height) / 2;

        let title_w = to_wstring("Music Presence — Settings");

        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x,
            y,
            win_width,
            win_height,
            None,
            None,
            hinstance,
            None,
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        SETTINGS_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

        // Enable modern dark mode title bar on Windows 10/11
        let dark_mode_val: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode_val as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );

        let font_name = to_wstring("Segoe UI");
        let font_regular = CreateFontW(
            -13,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(font_name.as_ptr()),
        );

        let font_bold = CreateFontW(
            -13,
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(font_name.as_ptr()),
        );

        let font_title = CreateFontW(
            -18,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(font_name.as_ptr()),
        );

        let font_badge = CreateFontW(
            -12,
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(font_name.as_ptr()),
        );

        let mut context = Box::new(SettingsContext {
            hwnd,
            config: config.clone(),
            font_regular,
            font_bold,
            font_title,
            font_badge,
            bg_brush,
            edit_brush,
            hwnd_poll_interval: HWND(std::ptr::null_mut()),
            hwnd_autostart: HWND(std::ptr::null_mut()),
            hwnd_autoupdate: HWND(std::ptr::null_mut()),
        });

        create_controls(hwnd, &mut context, hinstance.into());

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }

        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsContext;
        if !ptr.is_null() {
            let ctx = Box::from_raw(ptr);
            let _ = DeleteObject(ctx.font_regular);
            let _ = DeleteObject(ctx.font_bold);
            let _ = DeleteObject(ctx.font_title);
            let _ = DeleteObject(ctx.font_badge);
            let _ = DeleteObject(ctx.bg_brush);
            let _ = DeleteObject(ctx.edit_brush);
        }

        SETTINGS_HWND.store(0, Ordering::SeqCst);
    }
}

unsafe fn create_controls(parent: HWND, ctx: &mut SettingsContext, hinstance: HINSTANCE) {
    let cfg = match ctx.config.read() {
        Ok(c) => c.clone(),
        Err(_) => Config::default(),
    };

    // Header Title
    let title_text = to_wstring("Music Presence");
    let h_title = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(title_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SS_LEFT),
        22,
        16,
        150,
        24,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_title,
        WM_SETFONT,
        WPARAM(ctx.font_title.0 as isize as usize),
        LPARAM(1),
    );

    // Version Badge
    let ver_text = to_wstring(&format!("v{}", CURRENT_VERSION));
    let h_ver = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(ver_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SS_LEFT),
        175,
        21,
        120,
        18,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_ver,
        WM_SETFONT,
        WPARAM(ctx.font_badge.0 as isize as usize),
        LPARAM(1),
    );

    // Header Subtitle
    let sub_text = to_wstring("Apple Music Discord Rich Presence Settings");
    let h_sub = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(sub_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SS_LEFT),
        22,
        42,
        370,
        18,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_sub,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Group Box: Preferences
    let grp_text = to_wstring(" Preferences ");
    let h_grp = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(grp_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
        18,
        68,
        378,
        218,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_grp,
        WM_SETFONT,
        WPARAM(ctx.font_bold.0 as isize as usize),
        LPARAM(1),
    );

    // Label: Polling interval
    let lbl_poll = to_wstring("Refresh interval (milliseconds):");
    let h_lbl_poll = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(lbl_poll.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SS_LEFT),
        32,
        92,
        260,
        18,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_lbl_poll,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Edit: Polling interval
    let poll_val = to_wstring(&cfg.poll_interval_ms.to_string());
    let h_edit_poll = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        w!("EDIT"),
        PCWSTR(poll_val.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | WINDOW_STYLE(ES_NUMBER as u32),
        32,
        114,
        120,
        24,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_POLL_INTERVAL as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_edit_poll,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );
    ctx.hwnd_poll_interval = h_edit_poll;

    // Helper text
    let help_text = to_wstring("(default: 1500 ms, min: 200 ms).");
    let h_lbl_help = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("STATIC"),
        PCWSTR(help_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(SS_LEFT),
        162,
        118,
        210,
        18,
        parent,
        None,
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_lbl_help,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Checkbox: Launch on system startup
    let chk_autostart_text = to_wstring("Start Music Presence on Windows startup");
    let h_chk_autostart = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(chk_autostart_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
        32,
        152,
        340,
        22,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_CHK_AUTOSTART as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_chk_autostart,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    let is_autostart = cfg.auto_start || crate::autostart::is_autostart_enabled();
    let _ = SendMessageW(
        h_chk_autostart,
        BM_SETCHECK,
        WPARAM(if is_autostart {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }),
        LPARAM(0),
    );
    ctx.hwnd_autostart = h_chk_autostart;

    // Checkbox: Automatically check for updates
    let chk_autoupdate_text = to_wstring("Automatically check for updates on startup");
    let h_chk_autoupdate = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(chk_autoupdate_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
        32,
        182,
        340,
        22,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_CHK_AUTOUPDATE as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_chk_autoupdate,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    let _ = SendMessageW(
        h_chk_autoupdate,
        BM_SETCHECK,
        WPARAM(if cfg.auto_update {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }),
        LPARAM(0),
    );
    ctx.hwnd_autoupdate = h_chk_autoupdate;

    // Button: Check for updates
    let btn_check_text = to_wstring("Check for Updates");
    let h_btn_check = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(btn_check_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
        32,
        218,
        140,
        28,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_CHECK_UPDATE as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_btn_check,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Bottom Buttons
    // Reset to defaults
    let btn_reset_text = to_wstring("Default");
    let h_btn_reset = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(btn_reset_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
        18,
        304,
        84,
        28,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_RESET as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_btn_reset,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Cancel
    let btn_cancel_text = to_wstring("Cancel");
    let h_btn_cancel = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(btn_cancel_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
        216,
        304,
        84,
        28,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_CANCEL as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_btn_cancel,
        WM_SETFONT,
        WPARAM(ctx.font_regular.0 as isize as usize),
        LPARAM(1),
    );

    // Save
    let btn_save_text = to_wstring("Save");
    let h_btn_save = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR(btn_save_text.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
        312,
        304,
        84,
        28,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_SAVE as *mut _),
        hinstance,
        None,
    )
    .unwrap_or_default();
    SendMessageW(
        h_btn_save,
        WM_SETFONT,
        WPARAM(ctx.font_bold.0 as isize as usize),
        LPARAM(1),
    );
}

unsafe fn get_window_text_string(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; (len + 1) as usize];
    GetWindowTextW(hwnd, &mut buf);
    if let Some(pos) = buf.iter().position(|&c| c == 0) {
        buf.truncate(pos);
    }
    String::from_utf16_lossy(&buf)
}

unsafe fn trigger_manual_update_check(hwnd: HWND) {
    match updater::check_for_updates() {
        Ok(Some(info)) => {
            let msg = to_wstring(&format!(
                "A new version is available!\n\n• Current version: v{}\n• Latest version: v{}\n\nWould you like to download and install this update automatically?",
                info.current_version, info.latest_version
            ));
            let title_avail = to_wstring("Update Available");
            let res = MessageBoxW(
                hwnd,
                PCWSTR(msg.as_ptr()),
                PCWSTR(title_avail.as_ptr()),
                MB_YESNO | MB_ICONQUESTION,
            );

            if res.0 == IDYES {
                if let Some(download_url) = info.download_url {
                    // Apply self update
                    match updater::apply_update(&download_url) {
                        Ok(_) => {}
                        Err(e) => {
                            let err_msg = to_wstring(&format!("Failed to install update:\n{}\n\nYou can manually download it from:\n{}", e, info.release_url));
                            let err_title = to_wstring("Update Error");
                            MessageBoxW(
                                hwnd,
                                PCWSTR(err_msg.as_ptr()),
                                PCWSTR(err_title.as_ptr()),
                                MB_OK | MB_ICONWARNING,
                            );
                        }
                    }
                } else {
                    let info_msg = to_wstring(&format!(
                        "The new update is available on GitHub:\n{}",
                        info.release_url
                    ));
                    let info_title = to_wstring("Release Page");
                    MessageBoxW(
                        hwnd,
                        PCWSTR(info_msg.as_ptr()),
                        PCWSTR(info_title.as_ptr()),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
            }
        }
        Ok(None) => {
            let msg = to_wstring(&format!(
                "You are using the latest version of Music Presence (v{}).",
                CURRENT_VERSION
            ));
            let title_ok = to_wstring("Up to Date");
            MessageBoxW(
                hwnd,
                PCWSTR(msg.as_ptr()),
                PCWSTR(title_ok.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        Err(e) => {
            let msg = to_wstring(&format!(
                "Could not check for updates:\n{}\n\nPlease verify your internet connection.",
                e
            ));
            let title_err = to_wstring("Update Check Failed");
            MessageBoxW(
                hwnd,
                PCWSTR(msg.as_ptr()),
                PCWSTR(title_err.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }
}

unsafe fn save_settings(ctx: &SettingsContext) -> bool {
    let poll_str = get_window_text_string(ctx.hwnd_poll_interval);
    let poll_interval_ms: u64 = match poll_str.trim().parse::<u64>() {
        Ok(v) if v >= 200 => v,
        _ => {
            let msg = to_wstring("Please enter a valid refresh interval (minimum: 200 ms).");
            let title = to_wstring("Invalid Value");
            MessageBoxW(
                ctx.hwnd,
                PCWSTR(msg.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
            return false;
        }
    };

    let auto_start = SendMessageW(ctx.hwnd_autostart, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as usize
        == BST_CHECKED;
    let auto_update = SendMessageW(ctx.hwnd_autoupdate, BM_GETCHECK, WPARAM(0), LPARAM(0)).0
        as usize
        == BST_CHECKED;

    let mut current_config = match ctx.config.read() {
        Ok(c) => c.clone(),
        Err(_) => Config::default(),
    };

    current_config.poll_interval_ms = poll_interval_ms;
    current_config.auto_start = auto_start;
    current_config.auto_update = auto_update;

    // Apply startup registry setting
    let _ = crate::autostart::set_autostart(auto_start);

    // Save to config.toml
    if let Err(e) = current_config.save("config.toml") {
        let msg = to_wstring(&format!("Failed to save config.toml: {}", e));
        let title = to_wstring("Error");
        MessageBoxW(
            ctx.hwnd,
            PCWSTR(msg.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONWARNING,
        );
        return false;
    }

    // Update in-memory config
    if let Ok(mut cfg_guard) = ctx.config.write() {
        *cfg_guard = current_config;
    }

    true
}

unsafe fn reset_to_defaults(ctx: &SettingsContext) {
    let def = Config::default();
    let poll_w = to_wstring(&def.poll_interval_ms.to_string());
    let _ = SetWindowTextW(ctx.hwnd_poll_interval, PCWSTR(poll_w.as_ptr()));
    let _ = SendMessageW(
        ctx.hwnd_autostart,
        BM_SETCHECK,
        WPARAM(BST_UNCHECKED),
        LPARAM(0),
    );
    let _ = SendMessageW(
        ctx.hwnd_autoupdate,
        BM_SETCHECK,
        WPARAM(if def.auto_update {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }),
        LPARAM(0),
    );
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsContext;

    match msg {
        WM_CTLCOLORSTATIC | WM_CTLCOLORDLG | WM_CTLCOLORBTN => {
            if !ptr.is_null() {
                let ctx = &*ptr;
                let hdc = HDC(wparam.0 as *mut _);
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00222222));
                return LRESULT(ctx.bg_brush.0 as isize);
            }
        }
        WM_CTLCOLOREDIT => {
            if !ptr.is_null() {
                let ctx = &*ptr;
                let hdc = HDC(wparam.0 as *mut _);
                SetBkColor(hdc, COLORREF(0x00FFFFFF));
                SetTextColor(hdc, COLORREF(0x00111111));
                return LRESULT(ctx.edit_brush.0 as isize);
            }
        }
        WM_COMMAND => {
            let id = wparam.0 & 0xFFFF;
            if !ptr.is_null() {
                let ctx = &*ptr;
                match id {
                    ID_BTN_CHECK_UPDATE => {
                        trigger_manual_update_check(hwnd);
                        return LRESULT(0);
                    }
                    ID_BTN_SAVE => {
                        if save_settings(ctx) {
                            let _ = DestroyWindow(hwnd);
                        }
                        return LRESULT(0);
                    }
                    ID_BTN_CANCEL => {
                        let _ = DestroyWindow(hwnd);
                        return LRESULT(0);
                    }
                    ID_BTN_RESET => {
                        reset_to_defaults(ctx);
                        return LRESULT(0);
                    }
                    _ => {}
                }
            }
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            return LRESULT(0);
        }
        WM_DESTROY => {
            SETTINGS_HWND.store(0, Ordering::SeqCst);
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

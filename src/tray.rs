use crate::config::Config;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu,
    DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    HICON, IDI_APPLICATION, KillTimer, LoadIconW, MF_SEPARATOR, MF_STRING, MSG, PostMessageW,
    PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, SetTimer,
    SetWindowLongPtrW, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TrackPopupMenu, WINDOW_EX_STYLE, WM_CONTEXTMENU, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
    WM_NULL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_TIMER, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{PCWSTR, w};

const WM_TRAYICON: u32 = WM_USER + 100;
const WM_UPDATE_TRAY_STATUS: u32 = WM_USER + 101;

const ID_MENU_PRESENCE: usize = 1001;
const ID_MENU_QUIT: usize = 1002;

struct TrayContext {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    status_text: Arc<Mutex<String>>,
    should_exit: Arc<AtomicBool>,
    config: Arc<RwLock<Config>>,
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
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        let should_exit = Arc::new(AtomicBool::new(false));
        let status_text = Arc::new(Mutex::new("Waiting for Apple Music...".to_string()));
        let hwnd_holder = Arc::new(AtomicIsize::new(0));

        let exit_clone = should_exit.clone();
        let status_clone = status_text.clone();
        let hwnd_clone = hwnd_holder.clone();

        thread::spawn(move || {
            run_tray_thread(exit_clone, status_clone, hwnd_clone, config);
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
    config: Arc<RwLock<Config>>,
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
            Err(_) => return,
        };

        hwnd_out.store(hwnd.0 as isize, Ordering::SeqCst);

        // Load the embedded application icon (Resource ID 1 compiled from app.ico)
        #[allow(clippy::manual_dangling_ptr)]
        let (hicon, custom_icon) = match LoadIconW(hinstance, PCWSTR(1 as *const u16)) {
            Ok(ic) => (ic, Some(ic)),
            Err(_) => (LoadIconW(None, IDI_APPLICATION).unwrap_or_default(), None),
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

        let context = Box::new(TrayContext {
            hwnd,
            nid,
            status_text: status_text.clone(),
            should_exit: should_exit.clone(),
            config,
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
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayContext;
        if !ptr.is_null() {
            let ctx = &mut *ptr;

            if msg == WM_TIMER && wparam.0 == 1 {
                if !ctx.is_added {
                    let res = Shell_NotifyIconW(NIM_ADD, &ctx.nid);
                    if res.as_bool() {
                        ctx.is_added = true;
                        let _ = KillTimer(hwnd, 1);
                    }
                }
                return LRESULT(0);
            } else if ctx.taskbar_created_msg != 0 && msg == ctx.taskbar_created_msg {
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
}

unsafe fn show_menu(ctx: &TrayContext) {
    unsafe {
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
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
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
                crate::gui::open_settings_window(ctx.config.clone());
            }
            ID_MENU_QUIT => {
                ctx.should_exit.store(true, Ordering::SeqCst);
                PostQuitMessage(0);
            }
            _ => {}
        }
    }
}

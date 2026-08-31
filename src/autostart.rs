use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("MusicPresence");

/// Checks if MusicPresence is registered to launch on Windows startup in HKCU
pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_QUERY_VALUE, &mut hkey).is_ok() {
            let mut buf = [0u16; 512];
            let mut size = (buf.len() * 2) as u32;
            let res = RegQueryValueExW(
                hkey,
                APP_NAME,
                None,
                None,
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut size),
            );
            let _ = RegCloseKey(hkey);
            return res.is_ok();
        }
    }
    false
}

/// Enables or disables MusicPresence in the Windows HKCU Run registry key
pub fn set_autostart(enable: bool) -> Result<(), String> {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_SET_VALUE, &mut hkey).is_err() {
            return Err("Failed to open Windows Run registry key.".to_string());
        }

        let res = if enable {
            let exe_path = env::current_exe().map_err(|e| e.to_string())?;
            let path_str = format!("\"{}\"", exe_path.to_string_lossy());
            let wide_path: Vec<u16> = OsStr::new(&path_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let byte_len = (wide_path.len() * std::mem::size_of::<u16>()) as u32;
            RegSetValueExW(
                hkey,
                APP_NAME,
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    wide_path.as_ptr() as *const u8,
                    byte_len as usize,
                )),
            )
        } else {
            RegDeleteValueW(hkey, APP_NAME)
        };

        let _ = RegCloseKey(hkey);

        // Deleting a non-existent key returns error code 2 (ERROR_FILE_NOT_FOUND) which is fine
        if res.is_err() && (!enable && res.0 != 2) {
            return Err(format!(
                "Registry operation failed with error code: {:?}",
                res
            ));
        }

        Ok(())
    }
}

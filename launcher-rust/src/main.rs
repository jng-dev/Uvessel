#![cfg_attr(windows, windows_subsystem = "windows")]

mod logging;
mod paths;
mod runner;
mod state;
mod config;

use anyhow::Result;

fn main() -> Result<()> {
    let _single_instance = acquire_single_instance();
    if _single_instance.is_none() {
        return Ok(());
    }

    let root = paths::root_dir()?;
    logging::init(&root)?;
    runner::run(&root)
}

#[cfg(windows)]
fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let name = mutex_name();
    let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain(once(0)).collect();
    let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, wide.as_ptr()) };
    if handle == 0 {
        return Some(SingleInstanceGuard { handle });
    }
    let last_error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    if last_error == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(handle) };
        return None;
    }
    Some(SingleInstanceGuard { handle })
}

#[cfg(not(windows))]
fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    Some(SingleInstanceGuard {})
}

#[cfg(windows)]
fn mutex_name() -> String {
    let id = config::APP_ID.trim();
    let fallback = if !config::PRODUCT_NAME.trim().is_empty() {
        config::PRODUCT_NAME
    } else {
        config::NAME
    };
    let base = if id.is_empty() { fallback } else { id };
    let cleaned: String = base
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' { ch } else { '_' })
        .collect();
    format!("Local\\Uvessel-{}", cleaned)
}

#[cfg(windows)]
struct SingleInstanceGuard {
    handle: isize,
}

#[cfg(windows)]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if self.handle != 0 {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
        }
    }
}

#[cfg(not(windows))]
struct SingleInstanceGuard {}

use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};
use tauri::Manager;

#[derive(Clone, Serialize)]
struct InstallUiInfo {
    name: String,
    icon_path: Option<String>,
    done_file: Option<String>,
    version: Option<String>,
    mode: String,
    log_file: Option<String>,
    launch_file: Option<String>,
}

#[tauri::command]
fn get_install_ui_info(state: tauri::State<'_, InstallUiInfo>) -> InstallUiInfo {
    state.inner().clone()
}

#[tauri::command]
fn is_install_done(state: tauri::State<'_, InstallUiInfo>) -> bool {
    let Some(path) = state.done_file.as_ref() else {
        return false;
    };
    std::fs::metadata(path).is_ok()
}

#[derive(Serialize)]
struct InstallStatus {
    status: String,
}

#[tauri::command]
fn get_install_status(state: tauri::State<'_, InstallUiInfo>) -> InstallStatus {
    let Some(path) = state.done_file.as_ref() else {
        return InstallStatus {
            status: "running".to_string(),
        };
    };
    let status = match std::fs::read_to_string(path) {
        Ok(contents) => {
            let lowered = contents.trim().to_lowercase();
            if lowered.contains("fail") {
                "fail"
            } else if lowered.contains("ok") || lowered.contains("done") {
                "ok"
            } else {
                "ok"
            }
        }
        Err(_) => "running",
    };
    InstallStatus {
        status: status.to_string(),
    }
}

#[tauri::command]
fn mark_launch_requested(state: tauri::State<'_, InstallUiInfo>) -> Result<(), String> {
    let Some(path) = state.launch_file.as_ref() else {
        return Ok(());
    };
    std::fs::write(path, "launch").map_err(|err| err.to_string())
}

#[derive(Serialize)]
struct LogChunk {
    text: String,
    next_offset: u64,
}

#[tauri::command]
fn read_install_log(
    offset: u64,
    max_bytes: u64,
    state: tauri::State<'_, InstallUiInfo>,
) -> Result<LogChunk, String> {
    let Some(path) = state.log_file.as_ref() else {
        return Ok(LogChunk {
            text: String::new(),
            next_offset: offset,
        });
    };
    let mut file = std::fs::File::open(path).map_err(|err| err.to_string())?;
    let size = file.metadata().map_err(|err| err.to_string())?.len();
    let safe_offset = std::cmp::min(offset, size);
    file.seek(SeekFrom::Start(safe_offset))
        .map_err(|err| err.to_string())?;
    let to_read = std::cmp::min(max_bytes, size.saturating_sub(safe_offset)) as usize;
    let mut buf = vec![0u8; to_read];
    if to_read > 0 {
        file.read_exact(&mut buf).map_err(|err| err.to_string())?;
    }
    let text = String::from_utf8_lossy(&buf).to_string();
    Ok(LogChunk {
        text,
        next_offset: safe_offset + buf.len() as u64,
    })
}

#[tauri::command]
fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("main window not found".to_string());
    };
    window.close().map_err(|err| err.to_string())
}

#[tauri::command]
fn focus_window(app: tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("main window not found".to_string());
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    let _ = window.set_always_on_top(true);
    let _ = window.set_always_on_top(false);
    Ok(())
}

fn parse_args() -> InstallUiInfo {
    let mut name = "Installing...".to_string();
    let mut icon_path = None;
    let mut done_file = None;
    let mut version = None;
    let mut mode = "install".to_string();
    let mut log_file = None;
    let mut launch_file = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--name" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        name = value;
                    }
                }
            }
            "--icon" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        icon_path = Some(value);
                    }
                }
            }
            "--done-file" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        done_file = Some(value);
                    }
                }
            }
            "--version" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        version = Some(value);
                    }
                }
            }
            "--mode" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        mode = value;
                    }
                }
            }
            "--log-file" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        log_file = Some(value);
                    }
                }
            }
            "--launch-file" => {
                if let Some(value) = args.next() {
                    if !value.trim().is_empty() {
                        launch_file = Some(value);
                    }
                }
            }
            _ => {}
        }
    }
    InstallUiInfo {
        name,
        icon_path,
        done_file,
        version,
        mode,
        log_file,
        launch_file,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let info = parse_args();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(info.clone())
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&format!("Installing {}", info.name));
                if let Some(icon_path) = info.icon_path.as_ref() {
                    if let Ok(image) = tauri::image::Image::from_path(icon_path) {
                        let _ = window.set_icon(image);
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_install_ui_info,
            is_install_done,
            get_install_status,
            read_install_log,
            mark_launch_requested,
            focus_window,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use serde::Serialize;
use tauri::Manager;

#[derive(Clone, Serialize)]
struct InstallUiInfo {
    name: String,
    icon_path: Option<String>,
    done_file: Option<String>,
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

#[tauri::command]
fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("main window not found".to_string());
    };
    window.close().map_err(|err| err.to_string())
}

fn parse_args() -> InstallUiInfo {
    let mut name = "Installing...".to_string();
    let mut icon_path = None;
    let mut done_file = None;
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
            _ => {}
        }
    }
    InstallUiInfo {
        name,
        icon_path,
        done_file,
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
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

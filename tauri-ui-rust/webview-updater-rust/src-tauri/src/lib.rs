use serde::Serialize;
use tauri::Manager;

#[derive(Clone, Serialize)]
struct UpdateUiInfo {
    name: String,
    icon_path: Option<String>,
}

#[tauri::command]
fn get_update_ui_info(state: tauri::State<'_, UpdateUiInfo>) -> UpdateUiInfo {
    state.inner().clone()
}

fn parse_args() -> UpdateUiInfo {
    let mut name = "New release".to_string();
    let mut icon_path = None;
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
            _ => {}
        }
    }
    UpdateUiInfo { name, icon_path }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let info = parse_args();
    tauri::Builder::default()
        .manage(info.clone())
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("Updating");
                let _ = window.center();
                if let Some(icon_path) = info.icon_path.as_ref() {
                    if let Ok(image) = tauri::image::Image::from_path(icon_path) {
                        let _ = window.set_icon(image);
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_update_ui_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use crate::APP;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub fn raise_error_dialog(msg: &str) {

    APP.get()
        .unwrap()
        .dialog()
        .message(msg)
        .kind(MessageDialogKind::Error)
        .title("Warning")
        .blocking_show();
}

#[warn(dead_code)]
pub fn get_log_path() -> String {
    APP.get()
        .unwrap()
        .path()
        .app_log_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

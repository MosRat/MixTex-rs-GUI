// modified from https://github.com/pot-app/pot-desktop/blob/master/src-tauri/src/tray.rs

use tauri::CustomMenuItem;
use tauri::GlobalShortcutManager;
use tauri::SystemTrayEvent;
use tauri::SystemTrayMenu;
use tauri::SystemTrayMenuItem;
use tauri::SystemTraySubmenu;
use tauri::{AppHandle, Manager};

pub fn tray_event_handler<'a>(app: &'a AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => on_tray_click(),
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            // "input_translate" => on_input_translate_click(),
            // "copy_source" => on_auto_copy_click(app, "source"),
            // "clipboard_monitor" => on_clipboard_monitor_click(app),
            // "copy_target" => on_auto_copy_click(app, "target"),
            // "copy_source_target" => on_auto_copy_click(app, "source_target"),
            // "copy_disable" => on_auto_copy_click(app, "disable"),
            // "ocr_recognize" => on_ocr_recognize_click(),
            // "ocr_translate" => on_ocr_translate_click(),
            // "config" => on_config_click(),
            // "check_update" => on_check_update_click(),
            // "view_log" => on_view_log_click(app),
            // "restart" => on_restart_click(app),
            "quit" => on_quit_click(app),
            _ => {}
        },
        _ => {}
    }
}



pub fn update_tray(app_handle: tauri::AppHandle){
    let tray_handle = app_handle.tray_handle();
    tray_handle
        .set_menu(tray_menu_zh_cn())
        .unwrap()
}

fn tray_menu_zh_cn() -> tauri::SystemTrayMenu {
    // let input_translate = CustomMenuItem::new("input_translate", "输入翻译");
    // let clipboard_monitor = CustomMenuItem::new("clipboard_monitor", "监听剪切板");
    // let copy_source = CustomMenuItem::new("copy_source", "原文");
    // let copy_target = CustomMenuItem::new("copy_target", "译文");
    //
    // let copy_source_target = CustomMenuItem::new("copy_source_target", "原文+译文");
    // let copy_disable = CustomMenuItem::new("copy_disable", "关闭");
    // let ocr_recognize = CustomMenuItem::new("ocr_recognize", "文字识别");
    // let ocr_translate = CustomMenuItem::new("ocr_translate", "截图翻译");
    // let config = CustomMenuItem::new("config", "偏好设置");
    // let check_update = CustomMenuItem::new("check_update", "检查更新");
    // let restart = CustomMenuItem::new("restart", "重启应用");
    // let view_log = CustomMenuItem::new("view_log", "查看日志");
    let quit = CustomMenuItem::new("quit", "退出");
    SystemTrayMenu::new()
        // .add_item(input_translate)
        // .add_item(clipboard_monitor)
        // .add_submenu(SystemTraySubmenu::new(
        //     "自动复制",
        //     SystemTrayMenu::new()
        //         .add_item(copy_source)
        //         .add_item(copy_target)
        //         .add_item(copy_source_target)
        //         .add_native_item(SystemTrayMenuItem::Separator)
        //         .add_item(copy_disable),
        // ))
        // .add_native_item(SystemTrayMenuItem::Separator)
        // .add_item(ocr_recognize)
        // .add_item(ocr_translate)
        // .add_native_item(SystemTrayMenuItem::Separator)
        // .add_item(config)
        // .add_item(check_update)
        // .add_item(view_log)
        // .add_native_item(SystemTrayMenuItem::Separator)
        // .add_item(restart)
        .add_item(quit)
}

fn on_tray_click() {
    // let event = match get("tray_click_event") {
    //     Some(v) => v.as_str().unwrap().to_string(),
    //     None => {
    //         set("tray_click_event", "config");
    //         "config".to_string()
    //     }
    // };
    // match event.as_str() {
    //     "config" => config_window(),
    //     "translate" => input_translate(),
    //     "ocr_recognize" => ocr_recognize(),
    //     "ocr_translate" => ocr_translate(),
    //     "disable" => {}
    //     _ => config_window(),
    // }
}

fn on_quit_click(app: &AppHandle) {
    app.global_shortcut_manager().unregister_all().unwrap();
    eprint!("============== Quit App ==============");
    app.exit(0);
}
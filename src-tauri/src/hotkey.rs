
use crate::APP;
use tauri::{AppHandle, GlobalShortcutManager};

pub fn register<F>(app_handle: &AppHandle, name: &str, handler: F, key: &str) -> Result<(), String>
where
    F: Fn() + Send + 'static,
{
    let hotkey = key.to_string();
    //     {
    //     if key.is_empty() {
    //         match get(name) {
    //             Some(v) => v.as_str().unwrap().to_string(),
    //             None => {
    //                 set(name, "");
    //                 String::new()
    //             }
    //         }
    //     } else {
    //         key.to_string()
    //     }
    // };

    if !hotkey.is_empty() {
        match app_handle
            .global_shortcut_manager()
            .register(hotkey.as_str(), handler)
        {
            Ok(()) => {
                eprintln!("Registered global shortcut: {} for {}", hotkey, name);
            }
            Err(e) => {
                eprintln!("Failed to register global shortcut: {} {:?}", hotkey, e);
                return Err(e.to_string());
            }
        };
    }
    Ok(())
}
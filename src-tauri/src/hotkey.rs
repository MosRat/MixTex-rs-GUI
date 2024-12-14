use log::warn;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutWrapper};
use tauri_plugin_notification::NotificationExt;

pub fn register_hotkey<F>(app_handle: &AppHandle, handler: F, key: &str) -> Result<(), String>
where
    F: Fn(&AppHandle, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
{
    if !key.is_empty() {
        match app_handle.global_shortcut().on_shortcut(
            ShortcutWrapper::try_from(key).map_err(|e| e.to_string())?,
            handler,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Fail to set key  {key} : {e:?}");
                Err(e.to_string())
            }
        }
    } else {
        Ok(())
    }
}

pub fn handle_screenshot_hotkey(app:&AppHandle, _key:&Shortcut, event:ShortcutEvent) {
    use tauri_plugin_global_shortcut::ShortcutState;
    use log::info;
    use crate::window::screenshot_window;
    let app = app.clone();
    match event.state() {
            ShortcutState::Pressed => {
                tauri::async_runtime::spawn(async move {
                    info!(
                            ">>>>>>>>>>>>>>>>>>>>>>>>>>ShortCur Received!>>>>>>>>>>>>>>>>>>>>>>>"
                        );
                    screenshot_window();

                    app.notification()
                        .builder()
                        .title("Mixtex")
                        .body("screenshot hot key pressed! \nPlease wait...")
                        .show()
                        .unwrap();
                });

            }
            _ => {}
        }
}
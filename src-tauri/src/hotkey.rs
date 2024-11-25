use log::warn;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutWrapper};

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

use crate::hotkey::register_hotkey;
use crate::mixtex::Model;
use crate::window::{build_screenshot_window, screenshot_window};

use crate::screenshot::ScreenshotWrapper;
use anyhow::Result;
use log::{info, warn};
use tauri::{AppHandle, Manager};
use crate::onnx::MixTexOnnx;

pub(crate) async fn setup(app: AppHandle) -> Result<()> {
    app.get_webview_window("main").unwrap().clear_all_browsing_data()?;


    let _ = register_hotkey(
        &app,
        move |_app_handle, _key, event| {
            use tauri_plugin_global_shortcut::ShortcutState;
            match event.state() {
                ShortcutState::Pressed => {
                    tauri::async_runtime::spawn(async {
                        info!(
                            ">>>>>>>>>>>>>>>>>>>>>>>>>>ShortCur Received!>>>>>>>>>>>>>>>>>>>>>>>"
                        );
                        screenshot_window();
                    });
                }
                _ => {}
            }
        },
        "Shift+X",
    )
    .inspect_err(|e| warn!("{e}"));

    let _ = build_screenshot_window();

    tauri::async_runtime::spawn(init_states(app.clone()));

    Ok(())
}

async fn init_states(app: AppHandle) {
    app.manage(Model::<MixTexOnnx>::new());
    app.manage(ScreenshotWrapper::new());
}

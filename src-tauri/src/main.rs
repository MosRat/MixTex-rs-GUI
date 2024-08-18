// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![windows_subsystem="windows"]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{GlobalShortcutManager, Manager, State};
use window_shadows::set_shadow;
use window_vibrancy::{apply_acrylic,apply_vibrancy,};
use mixtex_rs_gui::onnx::MixTexOnnx;
use mixtex_rs_gui::vit_image_processor::preprocess;

use tauri::api::notification::Notification;
use tauri_plugin_log::LogTarget;
use mixtex_rs_gui::{APP, hotkey::register};
use mixtex_rs_gui::tray::{tray_event_handler, update_tray};
use mixtex_rs_gui::screenshot::*;
use mixtex_rs_gui::window::*;

struct Model{
    model:Mutex<MixTexOnnx>,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    token: String,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn inference(path: String,model:State<'_,Model>,window: tauri::Window) -> Result<String,String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        Ok("file not find!".to_string())
    } else {
        let mut stop = Arc::new(AtomicBool::new(false));
        let stop_arc = Arc::clone(&stop);
        window.once("stop", move |_| stop_arc.store(true, Ordering::SeqCst));
        let res = (*model.model.lock().map_err(|err|{ err.to_string() })?)
            .inference_by_step(&preprocess(path)?,|s|{
                window.emit("result",Payload{token:s}).expect("Send result fail!");
                stop.load(Ordering::SeqCst)
            }).map_err(|err|{ err.to_string() })?;
        window.emit("infer_stop",0).unwrap();

        Ok(res)
    }

}




fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, cwd| {
            Notification::new(&app.config().tauri.bundle.identifier)
                .title("The program is already running. Please do not start it again!")
                .body(cwd)
                .icon("pot")
                .show()
                .unwrap();
        }))
        .plugin(tauri_plugin_clipboard::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout])
                .build(),
        )
        .system_tray(tauri::SystemTray::new())
        .setup(|app|{

            // let clipboard = app.app_handle().state::<tauri_plugin_clipboard::ClipboardManager>();
            // clipboard.read_image_binary().unwrap();

            // global app handle
            APP.get_or_init(|| app.handle());

            app.global_shortcut_manager().register("Ctrl+Alt+F12",|| {
                screenshot_window();
            }).unwrap();

            // set window effect
            let window = app.get_window("main").unwrap();
            #[cfg(any(windows, target_os = "macos"))]
            set_shadow(&window, true).unwrap();

            // #[cfg(target_os = "macos")]
            // apply_vibrancy(&window, window_vibrancy::NSVisualEffectMaterial::HudWindow, None, None).expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
            //
            // #[cfg(target_os = "windows")]
            // apply_acrylic(&window, Some((18, 18, 18, 125))).expect("Unsupported platform! 'apply_blur' is only supported on Windows");


            tauri::async_runtime::spawn(async move {
                let app_handle = APP.get().unwrap();
                app_handle.manage(Model{model:Mutex::new(MixTexOnnx::build().expect("Fail load model!"))});
                // app_handle.manage(ImageWrapper(Mutex::new(Vec::new())));

                // register global shortcut
                match  register(app_handle,"call",||{
                    screenshot_window();
                },"Shift+X"){
                    Ok(()) => {}
                    Err(e) => Notification::new(app_handle.config().tauri.bundle.identifier.clone())
                        .title("MixTex fail set short cut")
                        .body(&e)
                        .icon("mixtex-rs-gui")
                        .show()
                        .unwrap(),
                }

                update_tray(app_handle);


            });

            Ok(())
        }
        )
        .invoke_handler(tauri::generate_handler![greet,inference,screenshot])
        .on_system_tray_event(tray_event_handler)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        // 保活
        .run(|_app_handle, event| {
            // _app_handle.global_shortcut_manager().unregister_all().unwrap();
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}

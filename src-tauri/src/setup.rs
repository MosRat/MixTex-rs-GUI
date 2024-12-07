use crate::hotkey::register_hotkey;
use crate::mixtex::Model;
use crate::window::{build_screenshot_window, screenshot_window};

use crate::onnx::MixTexOnnx;
use crate::screenshot::ScreenshotWrapper;
use anyhow::Result;
use image::{EncodableLayout, GenericImageView};
use log::{info, warn};
use serde_json::json;
use tauri::{AppHandle, DragDropEvent, Emitter, Listener, Manager, WindowEvent};
use crate::APP;

pub(crate) async fn setup(app: AppHandle) -> Result<()> {
    // app.get_webview_window("main").unwrap().clear_all_browsing_data()?;

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
        "Alt+X",
    )
    .inspect_err(|e| warn!("{e}"));

    let _ = build_screenshot_window();

    tauri::async_runtime::spawn(init_states(app.clone()));

    Ok(())
}

async fn init_states(app: AppHandle) {
    app.manage(Model::<MixTexOnnx>::new());
    app.manage(ScreenshotWrapper::new());
    init_listeners(app);
}

fn init_listeners(app_handle: AppHandle) {
    app_handle
        .clone()
        .get_webview_window("main")
        .unwrap()
        .on_window_event(|e|  {
            match e {
                WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) => {
                    if let Some(path) = paths.first() {
                        info!("get file drop:{}",path.display());

                        if path.extension().map(|e| e.to_str().unwrap() == "png" || e.to_str().unwrap() == "jpg").unwrap_or(false){
                            info!("get image drop:{}",path.display());

                            let app = APP.get().unwrap();
                            handle_read_image(app, path);
                        }

                    }
                }
                _ => {}
            }
        });
    app_handle
        .clone()
        .get_webview_window("main")
        .unwrap()
        .listen("select_img", |_|{
            use tauri_plugin_dialog::DialogExt;
            let app = APP.get().unwrap();

            let Some(img_path) =  app.dialog().file().blocking_pick_file() else { return; };
            handle_read_image(app,img_path.into_path().unwrap());
        });
}


pub fn handle_read_image<P:AsRef<std::path::Path> + std::fmt::Debug>(app_handle: &AppHandle,path:P){
    match image::open(&path) {
        Ok(img) => {
            let (w, h) = img.dimensions();

            let screenshot = app_handle
                .state::<ScreenshotWrapper>();
            screenshot.set_wh(w,h);
            screenshot.set_data(img.to_rgba8().as_bytes());
            app_handle
                .get_webview_window("main")
                .unwrap()
                .emit("image_arrive", json!({"w":w,"h":h}))
                .unwrap()
        }
        Err(e) => warn!("Fail to open image {:?} due to {:?}", path, e),
    }
}

#![allow(non_snake_case)]


pub mod setup;
pub mod vit_image_processor;
// pub mod winml;

pub mod onnx;

pub mod hotkey;
pub mod screenshot;

pub mod window;

pub mod tray;

pub mod mixtex;
mod model;
mod api;

use log::info;
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

use crate::setup::setup;
use crate::tray::create_tray;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_notification::NotificationExt;
use crate::screenshot::{get_screenshot, screenshot};
use crate::mixtex::generate;

pub static APP: OnceLock<AppHandle> = OnceLock::new();


// #[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, cwd| {
            app.notification()
                .builder()
                .title("The program is already running. Please do not start it again!")
                .body(cwd)
                .icon("pot")
                .show()
                .unwrap();
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Stdout),
                    // Target::new(TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        // .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // global app handle
            APP.get_or_init(|| app.handle().clone());

            // create tray before other setup
            create_tray(app.handle())?;

            // setup app in async runtime
            tauri::async_runtime::spawn(setup(app.handle().clone()));
            // let clipboard = app.app_handle().state::<tauri_plugin_clipboard::ClipboardManager>();
            // clipboard.read_image_binary().unwrap();

            // APP.get_or_init(|| app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![screenshot,generate,get_screenshot])
        // .on_system_tray_event(tray_event_handler)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        // 保活
        .run(|app_handle, event| match event {
            tauri::RunEvent::WindowEvent {label,event, .. } =>{
                if let tauri::WindowEvent::CloseRequested { api, ..  } =  event {
                    if label == "main".to_string() {
                        api.prevent_close();
                        app_handle.get_webview_window("main").unwrap().hide().unwrap()
                    }
                }
            },
            tauri::RunEvent::ExitRequested { api, code, .. } => {
                info!("App requested exit");
                match code {
                    None => api.prevent_exit(),
                    Some(_) => {}
                }
            }
            tauri::RunEvent::Exit => {
                info!("App exit");
            }
            _ => {}
        });
}

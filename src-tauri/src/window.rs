// Modified from Pot App : https://github.com/pot-app/pot-desktop/blob/master/src-tauri/src/window.rs

/*
 * Copyright (c) 2024. MosRat
 * All rights reserved.
 *
 * Project: mixtex-rs-gui
 * File Name: window.rs
 * Author: MosRat (work@whl.moe)
 * Description:
 */

use crate::APP;
use log::{info, warn};
use tauri::utils::config::WindowEffectsConfig;
use tauri::utils::WindowEffect;
use tauri::{Emitter, Manager, Monitor, PhysicalPosition, WebviewWindow};
// Unnecessary in tauri 2.0
// Get daemon window instance
// fn get_daemon_window() -> Window {
//     let app_handle = APP.get().unwrap();
//     match app_handle.get_window("daemon") {
//         Some(v) => v,
//         None => {
//             warn!("Daemon window not found, create new daemon window!");
//             WindowBuilder::new(
//                 app_handle,
//                 "daemon",
//             )
//                 .title("Daemon")
//                 .visible(false)
//                 .skip_taskbar(true)
//                 .build()
//                 .unwrap()
//         }
//     }
// }

// Different in tauri 2.0
// Get monitor where the mouse is currently located
pub fn get_current_monitor() -> Monitor {
    let app = APP.get().unwrap();
    match app.cursor_position() {
        Ok(PhysicalPosition { x, y }) => match app.monitor_from_point(x, y) {
            Ok(Some(m)) => m,
            _ => {
                warn!("Fail get monitor!");
                app.primary_monitor().unwrap().unwrap()
            }
        },
        Err(e) => {
            warn!("Fail to get cursor {e:?}");
            app.primary_monitor().unwrap().unwrap()
        }
    }
}

// Creating a window on the mouse monitor
pub fn build_window(label: &str, title: &str) -> (WebviewWindow, bool) {
    let app_handle = APP.get().unwrap();
    match app_handle.get_webview_window(label) {
        Some(v) => {
            info!("Window existence: {}", label);
            v.set_focus().unwrap();
            (v, true)
        }
        None => {
            info!("Window not existence, Creating new window: {}", label);
            let mut builder = tauri::WebviewWindowBuilder::new(
                app_handle,
                label,
                tauri::WebviewUrl::App("index.html".into()),
            )
            .enable_clipboard_access()
            .title(title)
            .visible(false)
                ;

            #[cfg(target_os = "macos")]
            {
                builder = builder
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .hidden_title(true);
            }
            #[cfg(not(target_os = "macos"))]
            {
                builder = builder.transparent(true);
                // .decorations(false);
            }
            #[cfg(target_os = "windows")]
            {
                builder = builder
            }

            let window = builder.build().unwrap();

            if label != "screenshot" {
                #[cfg(not(target_os = "linux"))]
                window.set_shadow(true).unwrap();
                #[cfg(target_os = "windows")]
                window
                    .set_effects(WindowEffectsConfig {
                        effects: vec![WindowEffect::Tabbed],
                        state: None,
                        radius: None,
                        color: None,
                    })
                    .unwrap();
            }
            (window, false)
        }
    }
}

pub fn config_window() {
    let (window, _exists) = build_window("config", "Fast Writer Config");
    window
        .set_min_size(Some(tauri::LogicalSize::new(800, 400)))
        .unwrap();
    window.set_size(tauri::LogicalSize::new(800, 600)).unwrap();
    window.set_resizable(false).unwrap();
    window.center().unwrap();
    window.show().unwrap();
}

pub fn build_screenshot_window() -> WebviewWindow {
    let (window, _exists) = build_window("screenshot", "Screenshot");
    window.set_skip_taskbar(true).unwrap();

    info!(">>>>>>>>>>>>>>>>>>>>Windows Build!>>>>>>>>>>>>>>>>");

    #[cfg(target_os = "macos")]
    {
        let monitor = window.current_monitor().unwrap().unwrap();
        let size = monitor.size();
        window.set_decorations(false).unwrap();
        window.set_size(*size).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    window.set_fullscreen(true).unwrap();

    window.set_always_on_top(true).unwrap();
    window
}

pub fn screenshot_window() -> WebviewWindow {
    let window = APP.get().unwrap().get_webview_window("screenshot").unwrap();

    info!(">>>>>>>>>>>>>>>>>>>>Windows Return!>>>>>>>>>>>>>>>>");

    window.emit("activate", "").unwrap();

    window
}

pub fn build_formula_window() -> (WebviewWindow, bool) {
    let app_handle = APP.get().unwrap();
    match app_handle.get_webview_window("formula") {
        Some(window) => {
            window.show().unwrap();
            window.set_focus().unwrap();
            (window, true)
        }
        None => {
            info!("Window not existence, Creating new window: {}", "formula");
            let builder = tauri::WebviewWindowBuilder::new(
                app_handle,
                "formula",
                // tauri::WebviewUrl::External("https://www.latexlive.com/".parse().unwrap()),
                tauri::WebviewUrl::App("editor.html".parse().unwrap()),
            )
            .enable_clipboard_access()
            .decorations(false)
            .resizable(false)
            // .initialization_script(include_str!("../scripts/formula_editor.js"))
            .disable_drag_drop_handler()
            .title("Latex Formula Editor")
            .transparent(true)
            .min_inner_size(800f64, 200f64)
            .inner_size(800f64, 600f64)
            .visible(false);
            let window = builder.build().unwrap();
            #[cfg(not(target_os = "linux"))]
            window.set_shadow(true).unwrap();
            #[cfg(target_os = "windows")]
            {
                window
                    .set_effects(WindowEffectsConfig {
                        effects: vec![WindowEffect::Tabbed],
                        state: None,
                        radius: None,
                        color: None,
                    })
                    .unwrap();
            }

            (window, false)
        }
    }
    // let (window, _exists) = build_window("formula", "Formula");
    // window.set_resizable(false).unwrap();
    // window.eval(include_str!("../scripts/formula_editor.js")).unwrap()
}

#[tauri::command(async)]
pub fn formula_window() {
    let _ = build_formula_window();
}

use crate::window::{config_window, formula_window};
use anyhow::Result;
use log::info;
use tauri::menu::{Menu, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{tray::TrayIconBuilder, AppHandle, EventLoopMessage, Manager};

pub fn create_tray(app: &AppHandle) -> Result<()> {
    let menu = create_menu(&app)?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Fast Writer!")
        .menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    info!("Activate main windows");
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {
                // info!("unhandled event {event:?}");
            }
        })
        .on_menu_event(handle_menu_event)
        .menu(&menu)
        .build(app)?;

    Ok(())
}

pub fn create_menu<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<Menu<R>> {
    let menu = Menu::new(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let config_i = MenuItem::with_id(app, "config", "Config", true, None::<&str>)?;
    let formula_i = MenuItem::with_id(app, "formula", "Latex formula editor", true, None::<&str>)?;

    menu.append_items(&[
        &formula_i,
        &config_i,
        &PredefinedMenuItem::separator(app)?,
        &quit_i,
    ])?;

    Ok(menu)
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    info!("{:?} menu item was clicked", event.id);

    match event.id.as_ref() {
        "quit" => {
            app.exit(0);
        }
        "config" => {
            config_window();
        }
        "formula" => {
            formula_window();
        }
        _ => {
            // info!("menu item {:?} not handled", event.id);
        }
    }
}

/*
 * Copyright (c) 2024. MosRat
 * All rights reserved.
 *
 * Project: fast-writer
 * File Name: screenshot.rs
 * Author: MosRat (work@whl.moe)
 * Description:
 */
use log::{info, warn};
// use image::{ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage};
use crate::window::{build_formula_window, build_window, formula_window, get_current_monitor};
use crate::APP;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Listener, Manager, PhysicalPosition, State};
use xcap::{
    image::{ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage},
    Monitor,
};
// use crate::api::simple_latex;

pub struct ScreenshotWrapper(Mutex<(Vec<u8>, u32, u32)>);

impl ScreenshotWrapper {
    pub fn new() -> Self {
        ScreenshotWrapper(Mutex::new((Vec::new(), 0, 0)))
    }
    pub fn set_data(&self, data: &[u8]) {
        (*self.0.lock().unwrap()).0 = Vec::from(data)
    }
    pub fn get_data(&self) -> Vec<u8> {
        self.0.lock().unwrap().0.to_vec()
    }

    pub fn set_wh(&self, w: u32, h: u32) {
        let mut lock = self.0.lock().unwrap();
        lock.1 = w;
        lock.2 = h;
    }
    pub fn get_wh(&self) -> (u32, u32) {
        let lock = self.0.lock().unwrap();
        (lock.1, lock.2)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CropPayload {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

#[tauri::command(async)]
pub fn screenshot(handle: AppHandle) -> tauri::ipc::Response {
    let s = Instant::now();
    let &PhysicalPosition { x, y } = get_current_monitor().position();
    info!("Screenshot screen with monitor position: x={}, y={}", x, y);

    let screen = Monitor::from_point(x, y)
        .inspect_err(|e| {
            warn!("{e:?}");
        })
        .unwrap();

    let img = screen
        .capture_image()
        .map_err(|e| e.to_string())
        .inspect_err(|e| {
            warn!("{e:?}");
        })
        .unwrap();
    info!("{:?}", s.elapsed());
    let buf = img.as_raw().clone();

    let window = handle.get_webview_window("screenshot").unwrap();
    window.clone().once("success", move |event| {
        // recognize_window();
        let handle = APP.get().unwrap();
        let size: CropPayload = serde_json::from_str(&event.payload()).unwrap();
        let img = img
            .view(size.left, size.top, size.width, size.height)
            .to_image();
        let state =  handle.state::<ScreenshotWrapper>();
        state.set_data(img.as_raw());
        state.set_wh(img.width(),img.height());
        let (w, h) = img.dimensions();
        window.emit("image_arrive",
                    json!({"w":w,"h":h})).unwrap();

        // tauri::async_runtime::spawn(async move {
        //     let (w, h) = img.dimensions();
        //     let mut buf = Vec::with_capacity((w * h * 4) as usize);
        //     encode_png(&img, &mut buf);
        //
        //     let latex = simple_latex(buf)
        //         .await.
        //         inspect_err(|e| warn!("Api call fail :{e:?}")).
        //         unwrap_or_default();
        //     let (window, exist) = build_formula_window();
        //     info!("window? {:}",exist);
        //
        //     if exist {
        //         window.emit("latex_arrive",
        //                     json!({
        //                                     "latex":latex,
        //                                     "w":w,
        //                                     "h":h
        //                                     })).unwrap();
        //     } else {
        //         window.clone()
        //             .once("init", move |_| {
        //                 info!("window init!");
        //                 window.emit("latex_arrive",
        //                             json!({
        //                                     "latex":latex,
        //                                     "w":w,
        //                                     "h":h
        //                                     })).unwrap();
        //             });
        //     }
        // });
        info!("Receive from js:{:?}", size);

        // let (w, _) = build_window("main", "fast writer");
        // w.set_resizable(false).unwrap();
        handle
            .get_webview_window("screenshot")
            .unwrap()
            .emit("success_save", "")
            .unwrap();
        info!("emit to js!");
    });

    info!("return js! {:?}", s.elapsed());
    tauri::ipc::Response::new(buf)
}

#[tauri::command(async)]
pub fn get_screenshot(screenshot_wrapper: State<ScreenshotWrapper>) -> tauri::ipc::Response {
    tauri::ipc::Response::new(screenshot_wrapper.get_data())
}

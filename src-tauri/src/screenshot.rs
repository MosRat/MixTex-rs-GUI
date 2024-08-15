// Modified from https://github.com/pot-app/pot-desktop/blob/master/src-tauri/src/screenshot.rs

use crate::ImageWrapper;
use tauri::{Manager, State};
use image::{GenericImageView, RgbaImage};
use log::info;
use serde::{Deserialize, Serialize   };
use crate::window::build_window;

// 负载类型必须实现 `Serialize` 和 `Clone`。
#[derive(Clone, Serialize,Deserialize, Debug)]
struct CropPayload {
    left:u32, top:u32, width:u32, height:u32
}

#[tauri::command]
pub async fn screenshot(x: i32, y: i32) ->Result<(),String> {
    use crate::APP;
    use dirs::cache_dir;
    use xcap::{Monitor,Window};
    use std::fs;
    info!("Screenshot screen with position: x={}, y={}", x, y);
    let screens = Monitor::all().unwrap();
    for screen in screens {
        // let info = screen.display_info;
        info!("Screen: {:?} {:?}", screen.x(),screen.y());
        if screen.x() == x && screen.y() == y {
            let handle = APP.get().unwrap();
            let mut app_cache_dir_path = cache_dir().expect("Get Cache Dir Failed");
            app_cache_dir_path.push(&handle.config().tauri.bundle.identifier);
            if !app_cache_dir_path.exists() {
                // 创建目录
                fs::create_dir_all(&app_cache_dir_path).expect("Create Cache Dir Failed");
            }
            app_cache_dir_path.push("mixtex_screenshot.png");
            info!("Image save: {}",app_cache_dir_path.display());

            let image = screen.capture_image().unwrap();
            image.save(app_cache_dir_path.clone()).unwrap();
            let window = APP.get().unwrap().get_window("screenshot").unwrap();
            let window_ = window.clone();
            window.once("success", move |event| {
                // recognize_window();
                let size:CropPayload = serde_json::from_str(&event.payload().unwrap()).unwrap();
                // (*global_img.0.lock().map_err(|err|{ err.to_string() }).expect("Fail to get global image")) =
                image.view(size.left,size.top,size.width,size.height).to_image().save(app_cache_dir_path).unwrap();
                info!("REceive from js:{:?}",size);

                let (w,_) =  build_window("main","mixtex-rs-gui");
                w.set_resizable(false).unwrap();
                APP.get().unwrap().get_window("screenshot").unwrap().emit("success_save","").unwrap();
                info!("emit to js!");
            });
            // fs::write(app_cache_dir_path, buffer).unwrap();
            break;
        }
    }
    Ok(())
}
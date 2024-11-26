use crate::onnx::MixTexOnnx;
use crate::screenshot::ScreenshotWrapper;
use crate::vit_image_processor::{preprocess_from_memory, preprocess_from_rgb_array};
use crate::{api, APP};
use std::string::ToString;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Listener, State};
use tauri_plugin_dialog::DialogExt;

use anyhow::Result;
use log::info;

pub trait OcrModel: Sized + Send + Sync {
    fn build() -> Result<Self>;
    fn inference(&self, img: &[f32]) -> Result<String>;
    fn generate<F>(&self, img: &[f32], callback: F) -> Result<String>
    where
        F: FnMut(String) -> bool;
}

pub struct Model<M: OcrModel = MixTexOnnx> {
    model: Mutex<M>,
}

impl<M: OcrModel> Model<M> {
    pub fn new() -> Model<M> {
        Model {
            model: Mutex::new(
                M::build()
                    .inspect_err(|e| {
                        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
                        APP.get()
                            .unwrap()
                            .dialog()
                            .message(format!("Model load fail! {:?}", e))
                            .kind(MessageDialogKind::Error)
                            .title("Fatal Error")
                            .blocking_show();
                    })
                    .unwrap(),
            ),
        }
    }

    pub fn inference(&self, img: &[f32]) -> Result<String> {
        self.model
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .inference(img)
    }

    pub fn generate<F>(&self, img: &[f32], callback: F) -> Result<String>
    where
        F: FnMut(String) -> bool,
    {
        self.model
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .generate(img, callback)
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum GenEvent {
    #[serde(rename_all = "camelCase")]
    TokenArrive { token: String },
    #[serde(rename_all = "camelCase")]
    Err { err: String },
    #[serde(rename_all = "camelCase")]
    Stop,
}

#[tauri::command]
pub async fn generate(
    screenshot: State<'_, ScreenshotWrapper>,
    model: State<'_, Model<MixTexOnnx>>,
    window: tauri::WebviewWindow,
    ch: tauri::ipc::Channel<GenEvent>,
    backend: String,
    token: String,
) -> Result<String, String> {
    let img = screenshot.get_data();
    let (w, h) = screenshot.get_wh();
    if img.is_empty() {
        ch.send(GenEvent::Err {
            err: "No Image!".to_string(),
        })
        .unwrap();
        return Err("No Image!".to_string());
    }
    match backend.as_str() {
        "mixtex" => {
            let stop = Arc::new(AtomicBool::new(false));
            let stop_arc = Arc::clone(&stop);
            window.once("stop", move |_| stop_arc.store(true, Ordering::SeqCst));
            let res = model
                .generate(&preprocess_from_rgb_array(&img, w, h)?, |token| {
                    let s = stop.load(Ordering::SeqCst);
                    ch.send(if !s {
                        // info!("token:{:?}",token);
                        GenEvent::TokenArrive { token }
                    } else {
                        GenEvent::Stop
                    })
                    .unwrap();
                    s
                })
                .map_err(|err| err.to_string())?;
            if !stop.load(Ordering::SeqCst) {
                ch.send(GenEvent::Stop).unwrap();
            }
            Ok(res)
        }
        "sl" => {
            let img_png = api::encode_rgba_to_png(&img, w, h).unwrap();
            let res_text = api::simple_latex(img_png, &token)
                .await
                .map_err(|e| e.to_string())?;
            ch.send(GenEvent::TokenArrive { token: res_text }).unwrap();
            ch.send(GenEvent::Stop).unwrap();
            Ok(token)
        }
        "got" => Ok(token),
        _ => {
            ch.send(GenEvent::Err {
                err: "Not support backend type!".to_string(),
            })
            .unwrap();
            Err("Not support backend type!".to_string())
        }
    }
}

#[tauri::command]
pub async fn read_image(path: String, img: State<'_, ScreenshotWrapper>) -> Result<(), String> {
    use xcap::image;
    img.set_data(image::open(path).map_err(|e| e.to_string())?.as_bytes());
    Ok(())
}

pub fn check_repeat(tokens: &[u32]) -> bool {
    if tokens.len() < 16 {
        return false;
    }
    for pattern_length in 2..=(tokens.len() / 12) {
        for start in (0..(tokens.len() - pattern_length * 12)).rev() {
            let rpt = tokens[start..(start + pattern_length)].repeat(12);
            if tokens[start..]
                .windows(pattern_length * 12)
                .rev()
                .any(|x| x.eq(&rpt))
            {
                return true;
            }
        }
    }

    false
}

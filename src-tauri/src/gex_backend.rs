use std::sync::Arc;
use gex::{GexContext, GexOcrModel};
use anyhow::Result;
use log::{info};
use tauri::Manager;
use tauri::path::BaseDirectory;
use crate::APP;

impl crate::mixtex::OcrModel for GexOcrModel {
    fn build() -> Result<Self> {
        let app_handle = APP.get().unwrap();
        let model_path = app_handle.path().resolve("models/mixtex-dec-Q4_K_M.gguf",BaseDirectory::Resource)?;
        let onnx_path = app_handle.path().resolve("models/encoder.onnx",BaseDirectory::Resource)?;
        info!("model_path: {}", model_path.display());
        info!("onnx_path: {}", onnx_path.display());
        if !model_path.exists() || !onnx_path.exists() { 
            return Err(anyhow::anyhow!("{} and {} not found", model_path.display(),onnx_path.display()));
        }

        let ctx = GexContext::new_with_onnx(&model_path, &onnx_path)?;
        Ok(Self::new(Arc::new(ctx)))
    }

    fn inference(&self, img: &[f32]) -> Result<String> {
        Ok(self.ctx.inference_raw(img)?)
    }

    fn generate<F>(&self, img: &[f32], mut callback: F) -> Result<String>
    where
        F: FnMut(String) -> bool,
    {
        Ok(self.ctx.inference_raw_stream(img, move |token| {
            callback(token.to_string())
        })?)
    }
}
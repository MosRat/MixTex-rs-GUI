#![allow(non_snake_case)]

use std::sync::Mutex;
use once_cell::sync::OnceCell;

pub mod vit_image_processor;
// pub mod winml;

pub mod onnx;

pub mod screenshot;
pub mod hotkey;

pub mod window;

pub mod tray;

pub static APP: OnceCell<tauri::AppHandle> = OnceCell::new();
// Text to be translated
pub struct ImageWrapper(pub Mutex<Vec<u8>>);

const ENCODER_BYTES: &[u8] = include_bytes!(r"..\..\models\encoder_model.onnx");
const DECODER_BYTES: &[u8] = include_bytes!(r"..\..\models\decoder_model_merged.onnx");
const TOKENIZER_STR: &str = include_str!(r"..\..\models\tokenizer\tokenizer.json");
// const TOKENIZER_STR: &str = "";


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
                .any(|x| {
                    x.eq(&rpt)
                }) {
                return true;
            }
        }
    }

    false
}
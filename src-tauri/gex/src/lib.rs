use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::Path;
use std::sync::Arc;

use gex_sys::{get_last_error, gex_context, gex_error, gex_free, gex_inference_raw_mem, gex_inference_raw_mem_stream, gex_init_with_onnx, size_t, gex_stream_callback_t};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GexError {
    #[error("GEX error: {0}")]
    GexError(String),
    #[error("Null pointer encountered")]
    NullPointer,
    #[error("UTF-8 conversion error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("CString conversion error")]
    CStringError(#[from] std::ffi::NulError),
}

pub type Result<T> = std::result::Result<T, GexError>;

fn get_last_gex_error() -> GexError {
    let err: gex_error = unsafe { get_last_error() };
    if err.msg.is_null() {
        GexError::GexError("Unknown error".to_string())
    } else {
        unsafe { GexError::GexError(CStr::from_ptr(err.msg).to_str().unwrap_or("").to_string()) }
    }
}

pub struct GexContext {
    ctx: gex_context,
}

impl Drop for GexContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { gex_free(self.ctx) };
        }
    }
}

unsafe impl Send for GexContext {}
unsafe impl Sync for GexContext {}

#[repr(C)]
struct CallbackWrapper<F>
where
    F: FnMut(&str) -> bool,
{
    callback: F,
}

impl GexContext {
    pub fn new_with_onnx(model_path: &Path, onnx_path: &Path) -> Result<Self> {
        let model_path = CString::new(model_path.to_string_lossy().as_bytes())?;
        let onnx_path = CString::new(onnx_path.to_string_lossy().as_bytes())?;

        let ctx = unsafe { gex_init_with_onnx(model_path.as_ptr(), onnx_path.as_ptr()) };

        if ctx.is_null() {
            Err(get_last_gex_error())
        } else {
            Ok(Self { ctx })
        }
    }

    pub fn inference_raw(&self, buf: &[f32]) -> Result<String> {
        let result_ptr = unsafe { gex_inference_raw_mem(self.ctx, buf.as_ptr(), buf.len() as size_t) };

        if result_ptr.is_null() {
            Err(get_last_gex_error())
        } else {
            unsafe { Ok(CStr::from_ptr(result_ptr).to_str()?.to_string()) }
        }
    }

    pub fn inference_raw_stream<F>(&self, buf: &[f32], mut callback: F) -> Result<String>
    where
        F: FnMut(&str) -> bool,
    {
        extern "C" fn callback_wrapper<F>(token: *const c_char, user_data: *mut c_void) -> c_int
        where
            F: FnMut(&str) -> bool,
        {
            if token.is_null() {
                return 0;
            }

            let wrapper = unsafe { &mut *(user_data as *mut CallbackWrapper<F>) };
            let token_str = unsafe { CStr::from_ptr(token) };
            match token_str.to_str() {
                Ok(s) => (wrapper.callback)(s) as c_int,
                Err(_) => 0,
            }
        }

        let wrapper = CallbackWrapper { callback };
        let wrapper_ptr = &wrapper as *const CallbackWrapper<F> as *mut c_void;

        let callback_struct = gex_stream_callback_t {
            callback: Some(callback_wrapper::<F>),
            user_data: wrapper_ptr,
        };

        let result_ptr = unsafe {
            gex_inference_raw_mem_stream(
                self.ctx,
                buf.as_ptr(),
                buf.len() as size_t,
                callback_struct,
            )
        };

        if result_ptr.is_null() {
            Err(get_last_gex_error())
        } else {
            unsafe { Ok(CStr::from_ptr(result_ptr).to_str()?.to_string()) }
        }
    }
}

pub struct GexOcrModel {
    pub ctx: Arc<GexContext>,
}

impl GexOcrModel {
    pub fn new(ctx: Arc<GexContext>) -> Self {
        Self { ctx }
    }
}
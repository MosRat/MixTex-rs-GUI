#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::{c_char, c_float, c_int, c_uchar, c_void,c_ulonglong};
pub type size_t = c_ulonglong;

#[repr(C)]
pub struct gex_error {
    pub msg: *const c_char,
}

#[repr(C)]
pub struct gex_device {
    pub idx: size_t,
    pub des: *mut c_char,
    pub name: *mut c_char,
    pub cap: size_t,
}

#[repr(C)]
pub struct gex_device_list {
    pub devices: [gex_device; 12],
    pub num_devices: size_t,
}

pub type gex_context = *mut c_void;
pub type gex_stream_callback =   *mut c_void;
// pub type gex_stream_callback =  unsafe extern "C" fn (token: *const c_char, user_data: *mut c_void) -> c_int;

extern "C" {
    pub fn gex_error_set(msg: *const c_char);
    pub fn get_last_error() -> gex_error;

    pub fn gex_init_default(model_path: *const c_char, mmproj_path: *const c_char) -> gex_context;
    pub fn gex_init_default_cpu(model_path: *const c_char, mmproj_path: *const c_char) -> gex_context;
    pub fn gex_init_with_param(
        model_path: *const c_char,
        mmproj_path: *const c_char,
        n_ctx: c_int,
        use_gpu: c_int,
        n_thread: c_int,
    ) -> gex_context;
    pub fn gex_init_with_onnx(model_path: *const c_char, onnx_path: *const c_char) -> gex_context;
    pub fn gex_free(ctx: gex_context);

    pub fn gex_inference_path(ctx: gex_context, image_path: *const c_char) -> *const c_char;
    pub fn gex_inference_path_stream(
        ctx: gex_context,
        image_path: *const c_char,
        cb: gex_stream_callback,
    ) -> *const c_char;
    pub fn gex_inference_mem(ctx: gex_context, buf: *const c_uchar, buf_size: size_t) -> *const c_char;
    pub fn gex_inference_raw_mem(ctx: gex_context, buf: *const c_float, buf_size: size_t) -> *const c_char;
    pub fn gex_inference_mem_stream(
        ctx: gex_context,
        buf: *const c_uchar,
        buf_size: size_t,
        cb: gex_stream_callback,
    ) -> *const c_char;
    pub fn gex_inference_raw_mem_stream(
        ctx: gex_context,
        buf: *const c_float,
        buf_size: size_t,
        cb: gex_stream_callback,
    ) -> *const c_char;

    pub fn gex_device_list_get() -> gex_device_list;
}
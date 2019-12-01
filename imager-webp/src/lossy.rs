use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use image::{DynamicImage, GenericImage, GenericImageView};
use webp_dev::sys::webp::{
    self as webp_sys,
    WebPConfig,
    WebPPicture,
    WebPMemoryWriter,
};

pub fn webp_lossy_config(q: f32) -> WebPConfig {
    let mut config: WebPConfig = unsafe {std::mem::zeroed()};
    unsafe {
        webp_sys::webp_config_init(&mut config);
        webp_sys::webp_validate_config(&mut config);
    };
    config.quality = q;
    config.lossless = 0;
    config.method = 6;
    config
}

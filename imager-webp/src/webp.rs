use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use crate::ffi::{
    self,
    WebPConfig,
    WebPPicture,
};


///////////////////////////////////////////////////////////////////////////////
// DECODER UTILS
///////////////////////////////////////////////////////////////////////////////

pub fn load_jpeg(data: Vec<u8>) -> Result<WebPPicture, String> {
    let mut picture: WebPPicture = unsafe {
        std::mem::zeroed()
    };
    let format = ::image::guess_format(&data).map_err(|x| format!("{:?}", x))?;
    match format {
        ::image::ImageFormat::JPEG => {
            unsafe {
                ffi::cbits::webp::webp_picture_from_jpeg(
                    data.as_ptr(),
                    data.len() as libc::size_t,
                    &mut picture
                );
            };
        }
        ::image::ImageFormat::PNG => {
            unsafe {
                ffi::cbits::webp::webp_picture_from_png(
                    data.as_ptr(),
                    data.len() as libc::size_t,
                    &mut picture
                );
            };
        }
        _ => {
            return Err(String::from("unknown format"))
        }
    }
    Ok(picture)
}


pub fn webp_config(q: f32) -> WebPConfig {
    let mut config: WebPConfig = unsafe {
        std::mem::zeroed()
    };
    unsafe {
        ffi::cbits::webp::webp_config_init(&mut config);
        ffi::cbits::webp::webp_validate_config(&mut config);
    };
    config.quality = q;
    config.lossless = 0;
    config.method = 6;
    config
}


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    let input_path = PathBuf::from("assets/test/1.jpeg");
    assert!(input_path.exists());
    let config = webp_config(75.0);
    let source = std::fs::read(input_path).expect("open image failed");
    let picture = load_jpeg(source);
}
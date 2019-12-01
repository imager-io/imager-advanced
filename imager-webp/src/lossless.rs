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


pub fn webp_lossless_config() -> WebPConfig {
    let mut config: WebPConfig = unsafe {std::mem::zeroed()};
    unsafe {
        webp_sys::webp_config_init(&mut config);
        webp_sys::webp_validate_config(&mut config);
    };
    config.lossless = 1;
    config.quality = 100.0;
    config.method = 6;
    config
}

pub fn new_lossless_picture(source: &DynamicImage) -> (WebPPicture, *mut WebPMemoryWriter) {
    let (width, height) = source.dimensions();
    assert!(width < webp_sys::WEBP_MAX_DIMENSION);
    assert!(height < webp_sys::WEBP_MAX_DIMENSION);
    let mut picture: WebPPicture = unsafe {std::mem::zeroed()};
    unsafe {
        assert!(webp_sys::webp_picture_init(&mut picture) != 0);
    };
    let argb_stride = width;
    picture.use_argb = 1;
    picture.width = width as i32;
    picture.height = height as i32;
    picture.argb_stride = argb_stride as i32;
    // FILL ARGB BUFFERS
    unsafe {
        let mut pixel_data = source
            // .to_rgb()
            .pixels()
            .flat_map(|(x, y, px)| {
                let [r, g, b, _] = px.0;
                [r, g, b].to_vec()
            })
            .collect::<Vec<_>>();
        let full_stride = argb_stride * 3;
        let status = webp_sys::webp_picture_import_rgb(
            &mut picture,
            pixel_data.as_mut_ptr(),
            full_stride as i32,
        );
        // CHECKS
        let expected_size = argb_stride * height * 3;
        assert!(pixel_data.len() as u32 == expected_size);
        assert!(status != 0);
        // CLEANUP
        std::mem::forget(pixel_data);
        // std::mem::drop(pixel_data);
    };
    // CHECKS
    assert!(picture.use_argb == 1);
    assert!(picture.y.is_null());
    assert!(!picture.argb.is_null());
    // OUTPUT WRITER
    let mut writer = unsafe {
        let mut writer: WebPMemoryWriter = std::mem::zeroed();
        webp_sys::webp_memory_writer_init(&mut writer);
        Box::into_raw(Box::new(writer))
    };
    unsafe extern "C" fn on_write(
        data: *const u8,
        data_size: usize,
        picture: *const WebPPicture,
    ) -> c_int {
        webp_sys::webp_memory_write(data, data_size, picture)
    }
    picture.writer = Some(on_write);
    unsafe {
        picture.custom_ptr = writer as *mut c_void;
    };
    // DONE
    (picture, writer)
}

pub fn encode_lossless(source: &DynamicImage) -> Vec<u8> {
    let config = webp_lossless_config();
    let (mut picture, writer_ptr) = new_lossless_picture(&source);
    unsafe {
        assert!(webp_sys::webp_encode(&config, &mut picture) != 0);
    };
    // COPY OUTPUT
    let mut writer = unsafe { Box::from_raw(writer_ptr) };
    // assert!(writer.size == writer.max_size);
    let mut output: Vec<u8> = unsafe {
        std::slice::from_raw_parts_mut(writer.mem, writer.size).to_vec()
    };
    // CLEANUP PICTURE & WRITER
    unsafe {
        // webp_sys::webp_picture_free(&mut picture);
        // webp_sys::webp_memory_writer_clear(writer_ptr);
        // std::mem::drop(picture);
        // std::mem::drop(writer_ptr);
        // std::mem::drop(writer);
    };
    // DONE
    output
}

///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    // let input_path = "assets/test/1.jpeg";
    // let input_path = "assets/samples/pexels-photo-3110502.jpeg";
    let input_path = "assets/samples/2yV-pyOxnPw300.jpeg";
    let source = ::image::open(input_path).expect("source image");
    let output = encode_lossless(&source);
    std::fs::write("assets/output/test.webp", output);
}
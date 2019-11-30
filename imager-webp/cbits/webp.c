#include <stdlib.h>
#include <stdio.h>
#include <assert.h>
#include <webp/encode.h>
#include <webp/decode.h>
#include <webp/types.h>
#include <imageio/image_enc.h>
#include <imageio/image_dec.h>


void webp_picture_from_jpeg(const uint8_t* data, size_t data_size, WebPPicture* const pic) {
    assert(ReadJPEG(data, data_size, pic, 1, NULL));
    assert(pic);
}

void webp_picture_from_png(const uint8_t* data, size_t data_size, WebPPicture* const pic) {
    assert(ReadPNG(data, data_size, pic, 1, NULL));
    assert(pic);
}

void webp_config_init(WebPConfig* config) {
    assert(WebPConfigInit(config));
}

void webp_config_preset(WebPConfig* config, WebPPreset preset, float quality) {
    assert(WebPConfigPreset(config, preset, quality));
}

void webp_validate_config(WebPConfig* config) {
    assert(WebPValidateConfig(config));
}
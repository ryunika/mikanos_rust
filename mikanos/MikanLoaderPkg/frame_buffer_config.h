#pragma once

#include <stdint.h>

#define PIXEL_RGB_RESERVED_8BIT_PER_COLOR 1
#define PIXEL_BGR_RESERVED_8BIT_PER_COLOR 2

typedef struct {
  uint8_t* frame_buffer;
  uint32_t pixels_per_scan_line;
  uint32_t horizontal_resolution;
  uint32_t vertical_resolution;
  uint8_t pixel_format;
} FrameBufferConfig;
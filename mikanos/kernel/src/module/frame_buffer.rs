const PIXEL_RGB_RESERVED_8BIT_PER_COLOR:u8 = 1;
const PIXEL_BGR_RESERVED_8BIT_PER_COLOR:u8 = 2;

extern "C" {
    static _binary_hankaku_bin_start: u8;
    static _binary_hankaku_bin_end: u8;
    static _binary_hankaku_bin_size: u8;
}

pub const HANKAKU_FONT_WIDTH: usize = 8;
pub const HANKAKU_FONT_HEIGHT: usize = 16; 

unsafe fn get_font(c: char) -> Option<*mut u8> {
    let index = 16 * c as usize;
    let size = (&_binary_hankaku_bin_size as *const u8) as usize;

    if index < size {
        let start = (&_binary_hankaku_bin_start as *const u8) as *mut u8;
        Some(start.offset(index as isize))
    } else {
        None
    }
}
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Vector2D<T> {
        Self { x, y }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct FrameBuffer {
    pub frame_buffer: *mut u8,
    pub pixels_per_scan_line: u32,
    pub horizotanal_resolution: u32,
    pub vertical_resolution: u32,
    pub format: u8
}

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct PixelWriter {
    frame_buffer: *mut u8,
    pixels_per_scan_line: u32,
    pub horizotanal_resolution: u32,
    pub vertical_resolution: u32,
    pub write: unsafe fn(&PixelWriter, u32, u32, &PixelColor),
}

impl PixelWriter {
    pub fn new(frame_buffer: *mut FrameBuffer) -> Self {
        let fb = unsafe { (*frame_buffer).clone() };
        let write = match fb.format {
            PIXEL_RGB_RESERVED_8BIT_PER_COLOR => Self::write_rgb,
            PIXEL_BGR_RESERVED_8BIT_PER_COLOR => Self::write_bgr,
            _ => Self::write_rgb,
        };
        Self {
            frame_buffer: fb.frame_buffer,
            pixels_per_scan_line: fb.pixels_per_scan_line,
            horizotanal_resolution: fb.horizotanal_resolution,
            vertical_resolution: fb.vertical_resolution,
            write,
        }
    }

    pub fn write(&self, x: u32, y: u32, color: &PixelColor) {
        unsafe {
            (self.write)(self, x as u32, y as u32, color);
        }
    }

    fn write_rgb(&self, x: u32, y: u32, color: &PixelColor) {
        let pixel_position = self.pixels_per_scan_line * y + x;
        let p = unsafe { self.frame_buffer.offset(4 * pixel_position as isize) };
        unsafe {
            *p = color.r;
            *p.offset(1) = color.g;
            *p.offset(2) = color.b;
        }
    }

    fn write_bgr(&self, x: u32, y: u32, color: &PixelColor) {
        let pixel_position = self.pixels_per_scan_line * y + x;
        let p = unsafe { self.frame_buffer.offset(4 * pixel_position as isize) };
        unsafe {
            *p = color.b;
            *p.offset(1) = color.g;
            *p.offset(2) = color.r;
        }
    }

    pub fn write_ascii(&self, x: u32, y: u32, c: char, color: &PixelColor) {
        let font = unsafe{ get_font(c) };
        let font = match font {
            None => return,
            Some(f) => f,
        };

        for dy in 0..HANKAKU_FONT_HEIGHT {
            for dx in 0..HANKAKU_FONT_WIDTH {
                let bits = unsafe{ *font.offset(dy as isize) };
                if ((bits << dx) & 0x80u8) > 0 {
                    self.write(x + dx as u32, y + dy as u32, color);
                }
            }
        }
    }

    pub fn write_string(&self, x: u32, y: u32, s: &str, color: &PixelColor) {
        for (i, c) in s.chars().enumerate() {
            self.write_ascii((x as usize + HANKAKU_FONT_WIDTH * i) as u32, y, c, color);
        }
    }

    pub fn fill_rectangle(&self, pos: Vector2D<u32>, size: Vector2D<u32>, c: &PixelColor) {
        for dy in 0..size.y {
            for dx in 0..size.x {
                self.write(pos.x + dx, pos.y + dy, c);
            }
        }
    }

    pub fn draw_rectangle(&self, pos: Vector2D<u32>, size: Vector2D<u32>, c: &PixelColor) {
        for dx in 0..size.x {
            self.write(pos.x + dx, pos.y, c);
            self.write(pos.x + dx, pos.y + size.y - 1, c);
        }
        for dy in 0..size.y {
            self.write(pos.x, pos.y + dy, c);
            self.write(pos.x + size.x - 1, pos.y + dy, c);
        }
    }
}
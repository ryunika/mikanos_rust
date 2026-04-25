
use super::frame_buffer::PixelWriter;
use super::frame_buffer::PixelColor;
use super::frame_buffer::Vector2D;

use super::frame_buffer::HANKAKU_FONT_WIDTH;
use super::frame_buffer::HANKAKU_FONT_HEIGHT;

use core::fmt;

const ROWS: usize = 25;
const COLUMNS: usize = 80;

pub struct Console {
    /* カーソル位置 */
    cursor_row: usize,
    cursor_column: usize,

    fg_color: PixelColor,
    bg_color: PixelColor,

    // 1 means null character to be written at end of a line
    buffer: [[char; COLUMNS + 1]; ROWS],
}

impl Console {
    pub fn new(fg_color: PixelColor, bg_color: PixelColor) -> Self {
        Self {
            cursor_row: 0,
            cursor_column: 0,
            fg_color,
            bg_color,
            buffer: [[char::from(0); COLUMNS + 1]; ROWS],
        }
    }

    pub fn put_string(&mut self, str: &str) {
        for char in str.chars() {
            match char {
                '\n' => self.new_line(),
                _ => {
                    if self.cursor_column < COLUMNS {
                        self.buffer[self.cursor_row][self.cursor_column] = char;
                        self.cursor_column += 1;
                    }
                }
            }
        }
    }

    pub fn refresh(&mut self, writer: &PixelWriter) {
        writer.fill_rectangle(
            Vector2D::new(0, 0),
            Vector2D::new((HANKAKU_FONT_WIDTH * COLUMNS) as u32, (HANKAKU_FONT_HEIGHT * ROWS) as u32),
            &self.bg_color,
        );

        for (i, row) in self.buffer.iter().enumerate() {
            for (j, c) in row.iter().enumerate() {
                writer.write_ascii(
                    (j * HANKAKU_FONT_WIDTH) as u32,
                    (i * HANKAKU_FONT_HEIGHT) as u32,
                    *c,
                    &self.fg_color,
                );
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_column = 0;

        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            // Scroll up
            for i in 0..ROWS - 1 {
                self.buffer[i] = self.buffer[i + 1];
            }
            self.buffer[ROWS - 1] = ['\0'; COLUMNS + 1]; // Clear the last row
            self.cursor_row = ROWS - 1; // Stay at the last row
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.put_string(s);
        Ok(())
    }
}
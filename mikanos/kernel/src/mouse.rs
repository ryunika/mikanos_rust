use crate::module::frame_buffer::{PixelColor, PixelWriter, Vector2D};

pub const CURSOR_WIDTH: u32 = 15;
pub const CURSOR_HEIGHT: u32 = 24;

/* マウスカーソルのビットマップ形状
 * '@' = 黒枠, '.' = 白塗り, ' ' = 透明（背景色で上書き） */
const CURSOR_SHAPE: [&str; CURSOR_HEIGHT as usize] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

/* カーソルの現在位置（画面座標） */
static mut CURSOR_X: u32 = 100;
static mut CURSOR_Y: u32 = 500;

/* 初期カーソルを現在位置に描画する */
pub fn draw(writer: &PixelWriter) {
    unsafe { draw_at(writer, CURSOR_X, CURSOR_Y); }
}

/* マウスの相対移動量を受け取り、カーソルを消去して新位置に再描画する */
pub fn move_relative(writer: &PixelWriter, bg: &PixelColor, dx: i8, dy: i8) {
    unsafe {
        /* 現在位置を背景色で塗りつぶして消去する */
        writer.fill_rectangle(
            Vector2D { x: CURSOR_X, y: CURSOR_Y },
            Vector2D { x: CURSOR_WIDTH, y: CURSOR_HEIGHT },
            bg,
        );
        /* 画面端でクランプした新しいカーソル位置を計算する */
        let new_x = (CURSOR_X as i32 + dx as i32)
            .max(0)
            .min(writer.horizotanal_resolution as i32 - CURSOR_WIDTH as i32) as u32;
        let new_y = (CURSOR_Y as i32 + dy as i32)
            .max(0)
            .min(writer.vertical_resolution as i32 - CURSOR_HEIGHT as i32) as u32;
        CURSOR_X = new_x;
        CURSOR_Y = new_y;
        /* 新しい位置にカーソルを描画する */
        draw_at(writer, CURSOR_X, CURSOR_Y);
    }
}

/* 指定座標にカーソル形状を描画する */
fn draw_at(writer: &PixelWriter, x: u32, y: u32) {
    for (row, line) in CURSOR_SHAPE.iter().enumerate() {
        for (col, c) in line.chars().enumerate() {
            let color = match c {
                '@' => PixelColor { r: 0, g: 0, b: 0 },
                '.' => PixelColor { r: 255, g: 255, b: 255 },
                _ => continue,
            };
            writer.write(x + col as u32, y + row as u32, &color);
        }
    }
}

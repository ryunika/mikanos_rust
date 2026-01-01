#![no_std]
#![no_main]

use core::arch::asm;
use core::panic;
use core::panic::PanicInfo;

mod module;
use module::frame_buffer::FrameBuffer;
use module::frame_buffer::PixelColor;
use module::frame_buffer::PixelWriter;

use module::console::Console;
use core::fmt::Write;

use crate::module::frame_buffer::Vector2D;
use crate::module::pci::read_bar;

use module::pci::Pci;
use module::pci::Device;

const kMouseCursorWidth: u32 = 15;
const kMouseCursorHeight: u32 = 24;
//char型の2次元配列
const mouse_cursor_shape: [&str; kMouseCursorHeight as usize] = [
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

fn find_xhc_device(pci: &Pci) -> Option<Device> {
    const BASE_CLASS_SERIAL_BUS: u8 = 0x0c;
    const SUB_CLASS_USB: u8 = 0x03;
    const INTERFACE_XHCI: u8 = 0x30;

    for i in 0..pci.num_device {
        let dev = pci.devices[i];
        if dev.class_code.same(BASE_CLASS_SERIAL_BUS, SUB_CLASS_USB, INTERFACE_XHCI) {
            return Some(dev);
        }
    }
    None
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) -> ! {
    let writer = PixelWriter::new(fb);

    let frame_width = writer.horizotanal_resolution;
    let frame_height = writer.vertical_resolution;

    let bg_color = PixelColor{r:45, g:118, b:237};
    let fg_color = PixelColor{r:255, g:255, b:255};

    writer.fill_rectangle(Vector2D{x: 0, y: 0}, Vector2D{x: frame_width, y: frame_height - 50}, &bg_color);

    writer.fill_rectangle(Vector2D{x: 0, y: frame_height - 50}, Vector2D{x: frame_width, y: 50}, &PixelColor { r: 1, g: 8, b: 17 });

    writer.fill_rectangle(Vector2D{x: 0, y: frame_height - 50}, Vector2D{x: frame_width/5, y: 50}, &PixelColor { r: 80, g: 80, b: 80 });

    writer.draw_rectangle(Vector2D{x: 10, y: frame_height - 40}, Vector2D{x: 30, y: 30}, &PixelColor { r: 160, g: 160, b: 160 });

    let mut console = Console::new(fg_color, bg_color);

    for (y, row) in mouse_cursor_shape.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let color = match c {
                '@' => PixelColor{r:0, g:0, b:0},
                '.' => PixelColor{r:255, g:255, b:255},
                _ => continue,
            };
            writer.write(x as u32 + 100, y as u32 + 500, &color);
        }
    }

    write!(&mut console, "Hello, MikanOS Console!\n");
    write!(&mut console, "The current cursor position is at row {}, column {}.\n", 0, 0);
    console.refresh(&writer);

    let mut pci = Pci::new();
    if let Err(_) = pci.scan_all_bus() {
        write!(&mut console, "PCI scan failed.\n");
    }
    pci.show_device(&mut console);

    //xhciデバイスを探す
    let xhc_dev = match find_xhc_device(&pci) {
        Some(dev) => dev,
        None => {
            write!(&mut console, "xHC device not found.\n");
            console.refresh(&writer);
            panic!("xHC device not found.");
        }
    };

    /* bar0を取得する */
    let xhc_bar = match read_bar(&xhc_dev, 0) {
        Ok(bar) => bar,
        Err(_) => {
            write!(&mut console, "ReadBar failed.\n");
            console.refresh(&writer);
            panic!("ReadBar failed.");
        }
    };

    let xhc_mmio_base = xhc_bar & !0xfu64;
    write!(&mut console, "xHC mmio_base = {:08x}\n", xhc_mmio_base);

    console.refresh(&writer);

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}
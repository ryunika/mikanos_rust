#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(inline_const)]  

use core::arch::asm;
use core::panic;
use core::panic::PanicInfo;

mod module;
mod mouse;
mod usb;

use module::frame_buffer::FrameBuffer;
use module::frame_buffer::PixelColor;
use module::frame_buffer::PixelWriter;

use module::console::Console;
use core::fmt::Write;

use crate::module::frame_buffer::Vector2D;
use crate::module::pci::read_bar;

use module::pci::Pci;
use module::pci::Device;
use module::pci;

use module::interrupt::set_idt_entry;
use module::interrupt::load_idt;
use module::interrupt::notify_end_of_interrupt;
use module::interrupt::InterruptVector;
use module::interrupt::InterruptFrame;

use module::queue;


/* マウスオブザーバーコールバック用のグローバル変数
 * extern "C" fn はクロージャを持てないため static mut を使用する */
static mut GLOBAL_WRITER: Option<PixelWriter> = None;
static mut BG_COLOR: PixelColor = PixelColor { r: 45, g: 118, b: 237 };

enum Message {
    InterruptXHCI
}
static mut EVENT_QUEUE:queue::ArrayQueue<Message, 10> = queue::ArrayQueue::new();

extern "C" fn mouse_observer(_buttons: u8, dx: i8, dy: i8) {
    unsafe {
        if let Some(ref w) = GLOBAL_WRITER {
            mouse::move_relative(w, &BG_COLOR, dx, dy);
        }
    }
}

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

    /* 初期マウスカーソルを描画 */
    mouse::draw(&writer);

    write!(&mut console, "Hello, MikanOS Console!\n");
    console.refresh(&writer);

    let mut pci = Pci::new();
    if let Err(_) = pci.scan_all_bus() {
        write!(&mut console, "PCI scan failed.\n");
    }

    /* xHCI デバイスを探す */
    let xhc_dev = match find_xhc_device(&pci) {
        Some(dev) => dev,
        None => {
            write!(&mut console, "xHC device not found.\n");
            console.refresh(&writer);
            panic!("xHC device not found.");
        }
    };

    let xhc_bar = match read_bar(&xhc_dev, 0) {
        Ok(bar) => bar,
        Err(_) => {
            write!(&mut console, "ReadBar failed.\n");
            console.refresh(&writer);
            panic!("ReadBar failed.");
        }
    };

    set_idt_entry(InterruptVector::XHCI, int_handler_xhci as u64);
    load_idt();

    let bsp_local_apic_id: u8 = unsafe {                                                                                                                
        let ptr = 0xfee00020usize as *const u32;                                                                                                        
        (ptr.read_volatile() >> 24) as u8                                                                                                               
    };     

    pci::configure_msi_fixed_destination(
        &xhc_dev,
        bsp_local_apic_id,
        pci::MSITriggerMode::Level,
        pci::MSIDeliveryMode::Fixed,
        InterruptVector::XHCI as u8,
        0);

    let xhc_mmio_base = (xhc_bar & !0xfu64) as usize;

    /* マウスオブザーバー用にライターと背景色を保存 */
    unsafe {
        GLOBAL_WRITER = Some(PixelWriter::new(fb));
    }

    /* USB xHCI コントローラーを初期化 */
    match usb::global::initialize(xhc_mmio_base) {
        Ok(()) => {
            let xhc = usb::global::controller();
            xhc.register_mouse_observer(mouse_observer);
            xhc.configure_ports();
        }
        Err(code) => {
            write!(&mut console, "USB init failed: {}\n", code);
            console.refresh(&writer);
        }
    }

    unsafe {
        loop {
            asm!("cli");
            if EVENT_QUEUE.count() == 0 {
                asm!("sti\n\thlt");
                continue;
            }

            let msg = EVENT_QUEUE.pop().unwrap();
            asm!("sti");

            match msg {
                Message::InterruptXHCI => {
                    while usb::global::controller().has_event() {
                        usb::global::controller().process_event();
                    }
                }
            }
        }
    }
}

extern "x86-interrupt" fn int_handler_xhci(_: *const InterruptFrame) {
    unsafe {
        EVENT_QUEUE.push(Message::InterruptXHCI);
    }
    notify_end_of_interrupt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

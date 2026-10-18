#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(inline_const)]

use core::arch::{asm, global_asm};
use core::panic;
use core::panic::PanicInfo;

mod module;
mod mouse;
mod usb;

use module::frame_buffer::FrameBuffer;
use module::frame_buffer::PixelColor;
use module::frame_buffer::PixelWriter;

use module::console;
use module::console::println;

use crate::module::frame_buffer::Vector2D;
use crate::module::pci::read_bar;

use module::pci;
use module::pci::Device;
use module::pci::Pci;

use module::interrupt::initialize_interrupt;
use module::interrupt::notify_end_of_interrupt;
use module::interrupt::set_idt_entry;
use module::interrupt::InterruptFrame;
use module::interrupt::InterruptVector;

use module::queue;

use module::segment::setup_segments;

use core::ffi::c_void;

/* UEFI BOOT側と同じ定義を使う */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub buffer_size: u64,
    pub buffer: *mut c_void,
    pub map_size: u64,
    pub map_key: u64,
    pub descriptor_size: u64,
    pub descriptor_version: u32,
}

const KERNEL_MAIN_STACK_SIZE: usize = 1024 * 1024;

#[repr(C, align(16))]
struct KernelMainStack([u8; KERNEL_MAIN_STACK_SIZE]);

#[no_mangle]
static mut kernel_main_stack: KernelMainStack = KernelMainStack([0; KERNEL_MAIN_STACK_SIZE]);

/* スタックを付け替えて、Rustのエントリポイントを呼ぶ */
global_asm!(
    r#"
    .global kernel_main
kernel_main:
    lea rsp, [rip + {kernel_main_stack} + {kernel_main_stack_size}]
    call {kernel_main_new_stack}
.Lfin:
    hlt
    jmp .Lfin
"#,
    kernel_main_stack = sym kernel_main_stack,
    kernel_main_stack_size = const KERNEL_MAIN_STACK_SIZE,
    kernel_main_new_stack = sym kernel_main_new_stack,
);

/* UEFI BOOT側と同じ定義を使う */
#[derive(Eq, PartialEq)]
#[warn(dead_code)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: u32,
    pub physical_start: usize,
    pub virtual_start: usize,
    pub number_of_pages: u64,
    pub attribute: u64,
}

/* UEFI BOOT側と同じ定義を使う */
#[warn(dead_code)]
#[derive(Eq, PartialEq, Debug)]
#[repr(C)]
pub enum MemoryType {
    KEfiReservedMemoryType,
    KEfiLoaderCode,
    KEfiLoaderData,
    KEfiBootServicesCode,
    KEfiBootServicesData,
    KEfiRuntimeServicesCode,
    KEfiRuntimeServicesData,
    KEfiConventionalMemory,
    KEfiUnusableMemory,
    KEfiACPIReclaimMemory,
    KEfiACPIMemoryNVS,
    KEfiMemoryMappedIO,
    KEfiMemoryMappedIOPortSpace,
    KEfiPalCode,
    KEfiPersistentMemory,
    KEfiMaxMemoryType,
}

impl TryFrom<u32> for MemoryType {
    type Error = u32;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::KEfiReservedMemoryType),
            1 => Ok(Self::KEfiLoaderCode),
            2 => Ok(Self::KEfiLoaderData),
            3 => Ok(Self::KEfiBootServicesCode),
            4 => Ok(Self::KEfiBootServicesData),
            5 => Ok(Self::KEfiRuntimeServicesCode),
            6 => Ok(Self::KEfiRuntimeServicesData),
            7 => Ok(Self::KEfiConventionalMemory),
            8 => Ok(Self::KEfiUnusableMemory),
            9 => Ok(Self::KEfiACPIReclaimMemory),
            10 => Ok(Self::KEfiACPIMemoryNVS),
            11 => Ok(Self::KEfiMemoryMappedIO),
            12 => Ok(Self::KEfiMemoryMappedIOPortSpace),
            13 => Ok(Self::KEfiPalCode),
            14 => Ok(Self::KEfiPersistentMemory),
            15 => Ok(Self::KEfiMaxMemoryType),
            _ => Err(value),
        }
    }
}

/* マウスオブザーバーコールバック用のグローバル変数
 * extern "C" fn はクロージャを持てないため static mut を使用する */
static mut GLOBAL_WRITER: Option<PixelWriter> = None;
static mut BG_COLOR: PixelColor = PixelColor {
    r: 45,
    g: 118,
    b: 237,
};

enum Message {
    InterruptXHCI,
}
static mut EVENT_QUEUE: queue::ArrayQueue<Message, 10> = queue::ArrayQueue::new();

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
        if dev
            .class_code
            .same(BASE_CLASS_SERIAL_BUS, SUB_CLASS_USB, INTERFACE_XHCI)
        {
            return Some(dev);
        }
    }
    None
}

#[no_mangle]
extern "C" fn kernel_main_new_stack(
    frame_buffer_config_ref: *const FrameBuffer,
    memory_map_ref: *const MemoryMap,
) -> ! {
    let mut frame_buffer_config = unsafe { (*frame_buffer_config_ref).clone() };
    let memory_map = unsafe { *memory_map_ref };
    let fb = &mut frame_buffer_config as *mut FrameBuffer;

    let writer = PixelWriter::new(fb);

    let frame_width = writer.horizotanal_resolution;
    let frame_height = writer.vertical_resolution;

    let bg_color = PixelColor {
        r: 45,
        g: 118,
        b: 237,
    };
    let fg_color = PixelColor {
        r: 255,
        g: 255,
        b: 255,
    };

    writer.fill_rectangle(
        Vector2D { x: 0, y: 0 },
        Vector2D {
            x: frame_width,
            y: frame_height - 50,
        },
        &bg_color,
    );

    writer.fill_rectangle(
        Vector2D {
            x: 0,
            y: frame_height - 50,
        },
        Vector2D {
            x: frame_width,
            y: 50,
        },
        &PixelColor { r: 1, g: 8, b: 17 },
    );

    writer.fill_rectangle(
        Vector2D {
            x: 0,
            y: frame_height - 50,
        },
        Vector2D {
            x: frame_width / 5,
            y: 50,
        },
        &PixelColor {
            r: 80,
            g: 80,
            b: 80,
        },
    );

    writer.draw_rectangle(
        Vector2D {
            x: 10,
            y: frame_height - 40,
        },
        Vector2D { x: 30, y: 30 },
        &PixelColor {
            r: 160,
            g: 160,
            b: 160,
        },
    );

    console::init(fg_color, bg_color);

    /* panicハンドラ用にライターを早期に保存 */
    unsafe {
        GLOBAL_WRITER = Some(PixelWriter::new(fb));
    }

    /* 初期マウスカーソルを描画 */
    mouse::draw(&writer);

    println!("Hello, MikanOS Console!");
    unsafe {
        console::CONSOLE.as_mut().unwrap().refresh(&writer);
    }

    setup_segments();

    let mut pci = Pci::new();
    if let Err(_) = pci.scan_all_bus() {
        println!("PCI scan failed.");
    }

    /* xHCI デバイスを探す */
    let xhc_dev = match find_xhc_device(&pci) {
        Some(dev) => dev,
        None => {
            println!("xHC device not found.");
            unsafe {
                console::CONSOLE.as_mut().unwrap().refresh(&writer);
            }
            panic!("xHC device not found.");
        }
    };

    let xhc_bar = match read_bar(&xhc_dev, 0) {
        Ok(bar) => bar,
        Err(_) => {
            println!("ReadBar failed.");
            unsafe {
                console::CONSOLE.as_mut().unwrap().refresh(&writer);
            }
            panic!("ReadBar failed.");
        }
    };

    set_idt_entry(InterruptVector::XHCI, int_handler_xhci as u64);
    initialize_interrupt();

    let bsp_local_apic_id: u8 = unsafe {
        let ptr = 0xfee00020usize as *const u32;
        (ptr.read_volatile() >> 24) as u8
    };

    let _ = pci::configure_msi_fixed_destination(
        &xhc_dev,
        bsp_local_apic_id,
        pci::MSITriggerMode::Level,
        pci::MSIDeliveryMode::Fixed,
        InterruptVector::XHCI as u8,
        0,
    );

    let xhc_mmio_base = (xhc_bar & !0xfu64) as usize;

    /* USB xHCI コントローラーを初期化 */
    match usb::global::initialize(xhc_mmio_base) {
        Ok(()) => {
            let xhc = usb::global::controller();
            xhc.register_mouse_observer(mouse_observer);
            xhc.configure_ports();
        }
        Err(code) => {
            println!("USB init failed: {}", code);
            unsafe {
                console::CONSOLE.as_mut().unwrap().refresh(&writer);
            }
        }
    }

    println!("memory_map: {:p}", &memory_map);
    let buffer = memory_map.buffer as usize;

    let available_memory_types = [
        MemoryType::KEfiBootServicesCode,
        MemoryType::KEfiBootServicesData,
        MemoryType::KEfiConventionalMemory,
    ];

    let mut iter = buffer;
    while iter < buffer + memory_map.map_size as usize {
        let desc = unsafe { &*(iter as *const MemoryDescriptor) };
        if let Ok(mt) = MemoryType::try_from(desc.memory_type) {
            if available_memory_types.contains(&mt) {
                println!(
                    "type = {}, phys = {:x} - {:x}, pages = {}, attr = {:#08x}",
                    (*desc).memory_type,
                    (*desc).physical_start,
                    (*desc).physical_start + ((*desc).number_of_pages as usize) * 4096 - 1,
                    (*desc).number_of_pages,
                    (*desc).attribute
                );
            }
        }
        iter += memory_map.descriptor_size as usize;
    }

    unsafe {
        console::CONSOLE.as_mut().unwrap().refresh(&writer);
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
        let _ = EVENT_QUEUE.push(Message::InterruptXHCI);
    }
    notify_end_of_interrupt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    unsafe {
        if let (Some(ref mut c), Some(ref w)) = (&mut console::CONSOLE, &GLOBAL_WRITER) {
            c.refresh(w);
        }
        loop {
            asm!("hlt");
        }
    }
}

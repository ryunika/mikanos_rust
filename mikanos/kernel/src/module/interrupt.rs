use core::arch::asm;

static mut IDT: [InterruptDescriptor; 256] = [InterruptDescriptor {
    offset_low: 0,
    segment_selector: 0,
    attr: 0,
    offset_middle: 0,
    offset_high: 0,
    reserved: 0,
}; 256];


pub enum InterruptVector {
    XHCI = 0x40,
}

#[allow(dead_code)]
pub struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

/* 記述子の種類(InterruptGateとkTrapGateのみ設定可能) */
#[allow(dead_code)]
enum DescriptorType {
    Upper8Bytes   = 0,
    LDT           = 2,
    TSSAvailable  = 9,
    TSSBusy       = 11,
    CallGate      = 12,
    InterruptGate = 14,
    TrapGate      = 15,
}


#[derive(Copy, Clone)]
#[repr(C, packed)]
struct InterruptDescriptor {
    offset_low: u16,
    segment_selector: u16,
    attr: u16,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}


pub fn notify_end_of_interrupt() {
    const LOCAL_APIC_BASE: usize = 0xfee00000;
    const LOCAL_APIC_EOI_OFFSET: usize = 0x0b0;  
    unsafe {
        let end_of_interrupt = (LOCAL_APIC_BASE + LOCAL_APIC_EOI_OFFSET) as *mut u32;
        end_of_interrupt.write_volatile(0);
    }
}

/**
*   @brief IDTR（Interrupt Descriptor Table Register）にアドレスとサイズをロードする
*/
fn load_idt_body(limit: u16, offset: u64) {
    #[repr(C, packed)]
    struct Idtr {
        limit: u16,
        offset: u64,
    }
    let idtr = Idtr { limit, offset };
    unsafe {
        asm!("lidt [{}]", in(reg) &idtr, options(nostack, preserves_flags));
    }
}

/**
*   @brief IDTR（Interrupt Descriptor Table Register）に、IDT変数をロードする
*/
pub fn load_idt() {
    unsafe {
        let limit = (core::mem::size_of_val(&IDT) - 1) as u16;
        let offset = IDT.as_ptr() as u64;
        load_idt_body(limit, offset);
    }
}

/**
*   @brief 現在のコードセグメントのセレクタ値
*/
fn get_code_segment() -> u16 {
    let cs: u16;                                                                                                                                       
    unsafe {                                                                                                                                           
        asm!("mov {:x}, cs", out(reg) cs, options(nostack, preserves_flags));                                                                          
    }                                                                                                                                                
    cs 
}

fn make_idt_attr_body(present: u8,desc_type: DescriptorType, descriptor_privilege_level: u8,interrupt_stack_table: u8) -> u16 {
    let mut attr: u16 = 0;

    const PRESENT_FLAG_SHIFT: u16 = 15;
    const DESCRIPTOR_PRIVILEGE_TYPE_SHIFT: u16 = 13;
    const INSTRACTION_STACK_TABLE_SHIFT: u16 = 0;
    const DESCRIPTOR_TYPE_SHIFT: u16 = 8;

    attr |= ((present as u16) << PRESENT_FLAG_SHIFT) as u16;
    attr |= ((descriptor_privilege_level as u16) << DESCRIPTOR_PRIVILEGE_TYPE_SHIFT) as u16;
    attr |= ((desc_type as u16) << DESCRIPTOR_TYPE_SHIFT) as u16;
    attr |= ((interrupt_stack_table as u16) << INSTRACTION_STACK_TABLE_SHIFT) as u16;
    attr
}

pub fn make_idt_attr() -> u16
{
    make_idt_attr_body(1, DescriptorType::InterruptGate, 0, 0)
}

/**
*   @brief IDT(Interrupt Descriptor Table)の割り込み記述子(desc)を設定する
*/
fn set_idt_entry_body(vector :InterruptVector,
                    attr: u16,
                    offset: u64,
                    segment_selector: u16) {
    unsafe {
        let desc = &mut IDT[vector as usize];
        desc.attr = attr;
        desc.offset_low = offset as u16;
        desc.offset_middle = (offset >> 16) as u16;
        desc.offset_high = (offset >> 32) as u32;
        desc.segment_selector = segment_selector;
    }
}

pub fn set_idt_entry(vector :InterruptVector,
                    offset: u64)
{
    let attr = make_idt_attr();
    let cs = get_code_segment();
    set_idt_entry_body(vector, attr, offset, cs);
}
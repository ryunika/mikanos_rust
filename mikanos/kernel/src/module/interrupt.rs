use crate::module::console::println;
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
    ExceptionDE = 0, /* Divide Error */
    ExceptionDB = 1,
    ExceptionBP = 3,  /* Breakpoint */
    ExceptionOF = 4,  /* Overflow */
    ExceptionBR = 5,  /* BOUND Range Exceeded */
    ExceptionUD = 6,  /* Invalid Opcode */
    ExceptionNM = 7,  /* Device Not Available */
    ExceptionDF = 8,  /* Double Fault */
    ExceptionTS = 10, /* Invalid TSS */
    ExceptionNP = 11, /* Segment Not Present */
    ExceptionSS = 12, /* Stack-Segment Fault */
    ExceptionGP = 13, /* General Protection */
    ExceptionPF = 14, /* Page Fault */
    ExceptionMF = 16, /* x87 FPU Floating-Point Error */
    ExceptionAC = 17, /* Alignment Check */
    ExceptionMC = 18, /* Machine Check */
    ExceptionXM = 19, /* SIMD Floating-Point */
    ExceptionVE = 20, /* Virtualization Exception */
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
    Upper8Bytes = 0,
    LDT = 2,
    TSSAvailable = 9,
    TSSBusy = 11,
    CallGate = 12,
    InterruptGate = 14,
    TrapGate = 15,
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
fn load_idt() {
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

fn make_idt_attr_body(
    present: u8,
    desc_type: DescriptorType,
    descriptor_privilege_level: u8,
    interrupt_stack_table: u8,
) -> u16 {
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

pub fn make_idt_attr() -> u16 {
    make_idt_attr_body(1, DescriptorType::InterruptGate, 0, 0)
}

/**
*   @brief IDT(Interrupt Descriptor Table)の割り込み記述子(desc)を設定する
*/
fn set_idt_entry_body(vector: InterruptVector, attr: u16, offset: u64, segment_selector: u16) {
    unsafe {
        let desc = &mut IDT[vector as usize];
        desc.attr = attr;
        desc.offset_low = offset as u16;
        desc.offset_middle = (offset >> 16) as u16;
        desc.offset_high = (offset >> 32) as u32;
        desc.segment_selector = segment_selector;
    }
}

pub fn set_idt_entry(vector: InterruptVector, offset: u64) {
    let attr = make_idt_attr();
    let cs = get_code_segment();
    set_idt_entry_body(vector, attr, offset, cs);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_de(frame: *const InterruptFrame) {
    _fault_handler_no_error("#DE", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_db(frame: *const InterruptFrame) {
    _fault_handler_no_error("#DB", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_bp(frame: *const InterruptFrame) {
    _fault_handler_no_error("#BP", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_of(frame: *const InterruptFrame) {
    _fault_handler_no_error("#OF", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_br(frame: *const InterruptFrame) {
    _fault_handler_no_error("#BR", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_ud(frame: *const InterruptFrame) {
    _fault_handler_no_error("#UD", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_nm(frame: *const InterruptFrame) {
    _fault_handler_no_error("#NM", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_df(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#DF", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_ts(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#TS", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_np(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#NP", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_ss(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#SS", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_gp(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#GP", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_pf(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#PF", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_mf(frame: *const InterruptFrame) {
    _fault_handler_no_error("#MF", frame);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_ac(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#AC", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_mc(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#MC", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_xm(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#XM", frame, error_code);
}

/**
*   @brief 例外ハンドラ
*/
extern "x86-interrupt" fn int_handler_ve(frame: *const InterruptFrame, error_code: u64) {
    _fault_handler_with_error("#VE", frame, error_code);
}

pub fn print_frame(f: &InterruptFrame, exp_name: &str) {
    println!("{}", exp_name);
    println!("CS:RIP {:x}:{:x}", f.cs, f.rip);
    println!("RFLAGS {:x}", f.rflags);
    println!("SS:RSP {:x}:{:x}", f.ss, f.rsp);
}

fn _fault_handler_with_error(name: &str, frame: *const InterruptFrame, error_code: u64) {
    let f = unsafe { frame.as_ref() }.unwrap();
    print_frame(f, name);
    println!("ERR {:x}", error_code);

    panic!("ReadBar failed.");
}

fn _fault_handler_no_error(name: &str, frame: *const InterruptFrame) {
    let f = unsafe { frame.as_ref() }.unwrap();
    print_frame(f, name);
    panic!("ReadBar failed.");
}

pub fn initialize_interrupt() {
    set_idt_entry(InterruptVector::ExceptionDE, int_handler_de as u64);
    set_idt_entry(InterruptVector::ExceptionDB, int_handler_db as u64);
    set_idt_entry(InterruptVector::ExceptionBP, int_handler_bp as u64);
    set_idt_entry(InterruptVector::ExceptionOF, int_handler_of as u64);
    set_idt_entry(InterruptVector::ExceptionBR, int_handler_br as u64);
    set_idt_entry(InterruptVector::ExceptionUD, int_handler_ud as u64);
    set_idt_entry(InterruptVector::ExceptionNM, int_handler_nm as u64);
    set_idt_entry(InterruptVector::ExceptionDF, int_handler_df as u64);
    set_idt_entry(InterruptVector::ExceptionTS, int_handler_ts as u64);
    set_idt_entry(InterruptVector::ExceptionNP, int_handler_np as u64);
    set_idt_entry(InterruptVector::ExceptionSS, int_handler_ss as u64);
    set_idt_entry(InterruptVector::ExceptionGP, int_handler_gp as u64);
    set_idt_entry(InterruptVector::ExceptionPF, int_handler_pf as u64);
    set_idt_entry(InterruptVector::ExceptionMF, int_handler_mf as u64);
    set_idt_entry(InterruptVector::ExceptionAC, int_handler_ac as u64);
    set_idt_entry(InterruptVector::ExceptionMC, int_handler_mc as u64);
    set_idt_entry(InterruptVector::ExceptionXM, int_handler_xm as u64);
    set_idt_entry(InterruptVector::ExceptionVE, int_handler_ve as u64);

    load_idt();
}

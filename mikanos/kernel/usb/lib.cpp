#include "usb/driver/xhci/xhci.hpp"
#include "usb/driver/classdriver/mouse.hpp"
#include "usb/driver/memory.hpp"

// Provide exit() for freestanding kernel: called from usb::xhci::Initialize()
// which we bypass, but the symbol must be resolvable.
extern "C" __attribute__((weak)) void exit(int) {
    while (true) { __asm__ volatile("hlt"); }
}

// Stubs for C/C++ runtime functions required by libc++ and newlib.
extern "C" {
    __attribute__((weak)) void abort() {
        while (true) { __asm__ volatile("hlt"); }
    }
    // malloc/free/posix_memalign backed by the USB memory pool.
    __attribute__((weak)) void* malloc(size_t size) {
        return usb::AllocMem(size, 0, 0);
    }
    __attribute__((weak)) void free(void*) {}
    __attribute__((weak)) int posix_memalign(void** ptr, size_t align, size_t size) {
        *ptr = usb::AllocMem(size, static_cast<unsigned int>(align), 0);
        return *ptr ? 0 : 12; // 12 = ENOMEM
    }
    // __cxa_atexit: register static destructor — no-op in a kernel.
    __attribute__((weak)) int __cxa_atexit(void (*)(void*), void*, void*) { return 0; }

    // newlib syscall stubs (required by libc.a linking)
    __attribute__((weak)) void* sbrk(intptr_t) { return (void*)-1; }
    __attribute__((weak)) int close(int) { return -1; }
    __attribute__((weak)) long lseek(int, long, int) { return -1; }
    __attribute__((weak)) int read(int, void*, unsigned) { return -1; }
    __attribute__((weak)) int write(int, const void*, unsigned) { return -1; }
    __attribute__((weak)) int fstat(int, void*) { return -1; }
    __attribute__((weak)) int isatty(int) { return 0; }
}

// Global operator new/delete backed by the USB memory pool.
// Required because std::deque (used in cdc.hpp) calls ::operator new.
void* operator new(size_t size) {
    return usb::AllocMem(size, 0, 0);
}
void* operator new[](size_t size) {
    return usb::AllocMem(size, 0, 0);
}
void operator delete(void*) noexcept {}
void operator delete[](void*) noexcept {}
void operator delete(void*, size_t) noexcept {}
void operator delete[](void*, size_t) noexcept {}

extern "C" {

void* usb_xhci_controller_new(uintptr_t mmio_base) {
    return new usb::xhci::Controller(mmio_base);
}

int usb_xhci_controller_initialize(void* xhc) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    auto err = c->Initialize();
    return static_cast<int>(err.code);
}

int usb_xhci_controller_run(void* xhc) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    auto err = c->Run();
    return static_cast<int>(err.code);
}

int usb_xhci_controller_configure_port(void* xhc, uint8_t port_num) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    auto port = c->PortAt(port_num);
    auto err = usb::xhci::ConfigurePort(*c, port);
    return static_cast<int>(err.code);
}

int usb_xhci_controller_process_event(void* xhc) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    auto err = usb::xhci::ProcessEvent(*c);
    return static_cast<int>(err.code);
}

bool usb_xhci_controller_primary_event_ring_has_front(void* xhc) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    return c->PrimaryEventRing()->HasFront();
}

uint8_t usb_xhci_controller_max_ports(void* xhc) {
    auto* c = reinterpret_cast<usb::xhci::Controller*>(xhc);
    return c->MaxPorts();
}

void usb_xhci_controller_register_mouse_observer(
    void* /* xhc */,
    void (*cb)(uint8_t buttons, int8_t dx, int8_t dy))
{
    usb::HIDMouseDriver::default_observer =
        [cb](uint8_t b, int8_t x, int8_t y) { cb(b, x, y); };
}

} // extern "C"

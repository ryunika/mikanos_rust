# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

MikanOS — a custom x86-64 OS built from scratch, following the book "ゼロからのOS自作入門". The project has two components:

1. **UEFI Bootloader** (`mikanos/MikanLoaderPkg/`) — C, built with EDK2/CLANG38
2. **Kernel** (`mikanos/kernel/`) — Rust (`#![no_std]`, `#![no_main]`), targets `x86_64-unknown-none-elf`

## Build Commands

- **Bootloader**: use the `build-bootloader` skill
- **Kernel**: use the `build-kernel` skill
- **Run with QEMU**: use the `run-qemu` skill

## Architecture

### Boot Flow

1. UEFI firmware loads `Loader.efi`
2. Bootloader (`Main.c: UefiMain`) gets the memory map, opens the GOP framebuffer, reads `kernel.elf` from the EFI partition
3. Bootloader parses the ELF64 headers, allocates pages at the kernel's requested virtual addresses, and copies LOAD segments (`CopyLoadSegments`)
4. Bootloader builds a `FrameBufferConfig` struct (framebuffer pointer, resolution, pixel format) and calls the kernel entry point directly: `entry_point(&config)`
5. UEFI boot services are exited before jumping to the kernel

### Kernel Entry & Modules

- Entry point: `kernel_main(fb: *mut FrameBuffer)` in `src/main.rs` (exported as `extern "C"`)
- `src/module/frame_buffer.rs` — `FrameBuffer` (repr(C), matches bootloader struct), `PixelWriter` (dispatches RGB/BGR writes at runtime), `Vector2D<T>`, font rendering via the embedded hankaku bitmap font
- `src/module/console.rs` — text console backed by `PixelWriter`; call `console.refresh(&writer)` to flush to screen
- `src/module/pci.rs` — PCI bus scan via port I/O, `Pci::scan_all_bus()`, `read_bar()`
- `src/module/err.rs` — shared error types

### Shared Data Structures

`FrameBufferConfig` / `FrameBuffer` is defined in both C (`frame_buffer_config.h`) and Rust (`module/frame_buffer.rs`) as `#[repr(C)]`. Any change to field order or types must be kept in sync across both files.

### Font

`hankaku.txt` encodes an 8x16 bitmap font. `build.rs` runs `tools/makefont.py` to produce `hankaku.bin`, then uses `objcopy` + `ar` to link it into the kernel as `libhankaku.a`. The font is accessed in Rust via the extern symbols `_binary_hankaku_bin_start/end/size`.

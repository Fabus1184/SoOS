[![SoOS.iso](https://github.com/Fabus1184/SoOS/actions/workflows/iso.yml/badge.svg)](https://github.com/Fabus1184/SoOS/actions/workflows/iso.yml)

# SoOS

x86_64 operating system written in Rust

The kernel is written in Rust, small parts in assembly.
Userspace library & applications written in Zig.  
Booting with [Limine](https://github.com/limine-bootloader/limine).

## 'inspirational' screenshots

![image](https://github.com/user-attachments/assets/829adf15-2d85-406d-90e0-620af28c65b8)



## Done

- [x] booting with limine
- [x] interrupts / exceptions & IRQs
- [x] basic framebuffer text output
- [x] i8253 PIT
- [x] RTC (CMOS)
- [x] Paging (4-level)
- [x] Kernel heap allocation
- [x] ELF64 binary loader
- [x] Basic syscall functionality
- [x] Fancy logging with [`log`](https://crates.io/crates/log)
- [x] Basic Preemptive multitasking & Process management
- [x] Basic Virtual File System

## Work in progress
- [ ] Fix bugs & undefined behaviors that I'm sure are lurking somewhere
- [ ] IRQ workers, stop disabling interrupts for long periods

### Userland
- [x] Shell
- [ ] `init` process
- [ ] Implement libc
- [ ] Dynamic linking

### Hardware
- [ ] IOAPIC
- [ ] Serial Port
- [ ] Mass Storage Drivers
- [ ] File System Drivers
- [ ] Networking
- [ ] ACPI
- [ ] PCI (discovering devices working a little bit)
- [ ] USB

### Meta
- [ ] Revise structure / modules
- [ ] Real logging
- [ ] CI
- [ ] More careful / isolated use of `unsafe`
- [ ] real cross-compilation

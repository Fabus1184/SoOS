# SoOS

x86_64 operating system written in Rust.

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

## Work in progress
- [ ] Preemptive multitasking & Process management

## Roadmap
- [ ] Fix bugs & undefined behaviors that I'm sure are lurking somewhere
- [ ] Stop disabling interrupts at all

### Kernel
- [ ] Virtual File System
- [ ] Process Management

### Userland
- [ ] `init` process
- [ ] Shell
- [ ] Implement libc
- [ ] Dynamic linking

### Hardware
- [ ] IOAPIC
- [ ] Serial Port
- [ ] Mass Storage Drivers
- [ ] File System Drivers
- [ ] Networking
- [ ] ACPI
- [ ] PCI
- [ ] USB

### Meta
- [ ] Revise structure / modules
- [ ] CI
- [ ] More careful / isolated use of `unsafe`
- [ ] GCC real cross-compilation

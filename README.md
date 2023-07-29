# SoOS

x86_64 operating system written in Rust.

## Done

- [x] booting with limine
- [x] interrupts / exceptions & IRQs
- [x] basic framebuffer text output
- [x] i8253 PIT
- [x] RTC (CMOS)
- [x] paging (4-level)
- [x] kernel heap allocation
- [x] ELF64 binary loader
- [x] basic syscall functionality
- [x] fancy logging with [`log`](https://crates.io/crates/log)

## Work in progress
- [ ] Preemptive multitasking & Process management

## Roadmap

### Kernel
- [ ] Virtual File System
- [ ] Process Management

### Userland
- [ ] `init` process
- [ ] shell
- [ ] implement libc
- [ ] dynamic linking

### Hardware
- [ ] IOAPIC
- [ ] Serial Port
- [ ] Mass Storage Drivers
- [ ] File System Drivers
- [ ] Networking
- [ ] ACPI
- [ ] PCI
- [ ] USB

use alloc::{format, string::String};
use ringbuffer::RingBuffer as _;

use crate::{
    process::{MappedPage, PROCESSES},
    vfs::{Directory, File},
};

pub fn init_fs(fs: &mut Directory) {
    log::debug!("VFS: ");

    fs.create_file("/home/test", File::regular(b"Hello World!"));

    fs.create_file(
        "/bin/sosh",
        File::regular(include_bytes_aligned::include_bytes_aligned!(
            32,
            "../../../build/userspace/bin/sosh"
        )),
    );

    fs.create_file(
        "/bin/cat",
        File::regular(include_bytes_aligned::include_bytes_aligned!(
            32,
            "../../../build/userspace/bin/cat"
        )),
    );

    fs.create_file(
        "/var/log",
        File::special(|_, offset, writer| {
            let mut written = 0;
            let ringbuffer = crate::kernel::logger::KERNEL_LOGGER.lock_ringbuffer();
            for byte in ringbuffer.iter().skip(offset) {
                written += writer.write(&[*byte])?;
            }

            Ok(written)
        }),
    );

    fs.create_special_directory("/proc", |files, directories| {
        for process in PROCESSES.processes().iter() {
            let pid = process.pid();
            files
                .entry(alloc::format!("{pid}"))
                .or_insert(File::special(move |_self, offset, writer| {
                    if offset != 0 {
                        return Err(crate::io::WriterError::InvalidOffset);
                    }

                    let mut written = 0;

                    let line = alloc::format!("pid: {pid}\n");

                    written += writer.write(line.as_bytes())?;

                    Ok(written)
                }));

            directories
                .entry(alloc::format!("{pid}"))
                .or_insert_with(|| {
                    let mut files = alloc::collections::BTreeMap::new();

                    files.insert(
                        String::from("stdin"),
                        File::stream2(crate::process::ForeignStreamType::Process {
                            pid,
                            file_descriptor: 0,
                        }),
                    );

                    files.insert(
                        String::from("stdout"),
                        File::stream2(crate::process::ForeignStreamType::Process {
                            pid,
                            file_descriptor: 1,
                        }),
                    );

                    files.insert(
                        String::from("memmap"),
                        File::special(move |_self, _offset, writer| {
                            let mut written = 0;

                            let process = PROCESSES.process(pid);

                            written += writer.write(
                                format!("{:<16}{:<16}{:<16}{}\n", "start", "end", "name", "flags")
                                    .as_bytes(),
                            )?;

                            let mut acc = Option::<(MappedPage, u64)>::None;

                            for &mapped_page in &process.mapped_pages {
                                match acc {
                                    None => {
                                        acc = Some((mapped_page, mapped_page.page.size()));
                                    }
                                    Some((page, size))
                                        if page.page.start_address() + size
                                            == mapped_page.page.start_address()
                                            && page.flags == mapped_page.flags
                                            && page.name == mapped_page.name =>
                                    {
                                        acc = Some((page, size + mapped_page.page.size()));
                                    }
                                    Some((page, size)) => {
                                        written += writer.write(
                                            alloc::format!(
                                                "{:<#16x}{:<#16x}{:<16}{:?}\n",
                                                page.page.start_address(),
                                                size,
                                                page.name,
                                                page.flags
                                            )
                                            .as_bytes(),
                                        )?;
                                        acc = Some((mapped_page, mapped_page.page.size()));
                                    }
                                }
                            }

                            if let Some((page, size)) = acc {
                                written += writer.write(
                                    alloc::format!(
                                        "{:<#16x}{:<#16x}{:<16}{:?}\n",
                                        page.page.start_address(),
                                        size,
                                        page.name,
                                        page.flags
                                    )
                                    .as_bytes(),
                                )?;
                            }

                            Ok(written)
                        }),
                    );

                    Directory::Regular {
                        files,
                        directories: alloc::collections::BTreeMap::new(),
                    }
                });
        }
    });

    fs.create_file(
        "/sys/pci/devices",
        File::special(|_self, _offset, writer| {
            let mut written = 0;

            for dev in crate::driver::pci::scan().expect("Failed to scan PCI devices!") {
                written += writer.write(
                    alloc::format!(
                        "bus {} device {} function {} class {:?}\n",
                        dev.bus,
                        dev.device,
                        dev.function,
                        dev.header.class
                    )
                    .as_bytes(),
                )?;
            }

            Ok(written)
        }),
    );

    fs.create_file(
        "/sys/memory",
        File::special(|_self, _offset, writer| {
            let mut written = 0;

            let kernel_paging = crate::kernel_paging();

            written += writer.write(
                alloc::format!("{:<#16} {:<#16} {}\n", "Base Address", "Length", "Type").as_bytes(),
            )?;

            for entry in kernel_paging.frame_allocator().memmap.iter() {
                let line = alloc::format!(
                    "{:<#16x} {:<#16x} {:?}\n",
                    entry.base,
                    entry.len,
                    entry.type_,
                );
                written += writer.write(line.as_bytes())?;
            }

            let (allocated, used, total) = kernel_paging.frame_allocator().stats();
            written += writer.write(
                alloc::format!("\nused frames: {used}/{total} ({allocated} allocated)\n")
                    .as_bytes(),
            )?;

            Ok(written)
        }),
    );

    fs.create_file(
        "/dev/mouse",
        File::stream1(crate::process::OwnedStreamType::Mouse),
    );

    fs.create_file(
        "/dev/keyboard",
        File::stream1(crate::process::OwnedStreamType::Keyboard),
    );
}

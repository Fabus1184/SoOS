#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u64)]
#[allow(dead_code)]
pub enum MemmapEntryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    KernelAndModules = 6,
    Framebuffer = 7,
}

#[derive(Debug, Copy, Clone)]
pub struct MemmapEntry {
    pub base: u64,
    pub len: u64,
    pub type_: MemmapEntryType,
}

#[derive(Debug, Copy, Clone)]
pub struct SoosMemmap([Option<MemmapEntry>; 128]);

impl SoosMemmap {
    pub fn iter_usable_addresses(&self) -> impl Iterator<Item = u64> + '_ {
        self.0
            .iter()
            .flatten()
            .filter(|e| e.type_ == MemmapEntryType::Usable)
            .flat_map(|e| e.base..e.base + e.len)
    }

    pub fn iter(&self) -> impl Iterator<Item = &MemmapEntry> {
        self.0.iter().flatten()
    }
}

impl From<&limine::response::MemoryMapResponse> for SoosMemmap {
    fn from(limine_memmap: &limine::response::MemoryMapResponse) -> Self {
        let mut memmap = [None; 128];

        for (i, entry) in limine_memmap.entries().iter().enumerate() {
            memmap[i] = Some(MemmapEntry {
                base: entry.base,
                len: entry.length,
                type_: unsafe {
                    core::mem::transmute::<limine::memory_map::EntryType, MemmapEntryType>(
                        entry.entry_type,
                    )
                },
            });
        }

        Self(memmap)
    }
}

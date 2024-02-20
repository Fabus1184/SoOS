use limine::MemmapResponse;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
#[allow(dead_code)]
enum MemmapEntryType {
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
struct MemmapEntry {
    base: u64,
    len: u64,
    typ: MemmapEntryType,
}

#[derive(Debug, Copy, Clone)]
pub struct SoosMemmap([Option<MemmapEntry>; 128]);

impl SoosMemmap {
    pub fn iter_usable_addresses(&self) -> impl Iterator<Item = u64> + '_ {
        self.0
            .iter()
            .flatten()
            .filter(|e| e.typ == MemmapEntryType::Usable)
            .flat_map(|e| e.base..e.base + e.len)
    }
}

impl From<&MemmapResponse> for SoosMemmap {
    fn from(limine_memmap: &MemmapResponse) -> Self {
        let mut memmap = [None; 128];

        for (i, entry) in limine_memmap.memmap().iter().enumerate() {
            memmap[i] = Some(MemmapEntry {
                base: entry.base,
                len: entry.len,
                typ: unsafe { core::mem::transmute(entry.typ) },
            });
        }

        Self(memmap)
    }
}

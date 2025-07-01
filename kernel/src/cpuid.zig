pub const Cpuid = struct {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
};

pub fn cpuid(leaf: u8, subleaf: u8) Cpuid {
    var eax: u32 = 0;
    var ebx: u32 = 0;
    var ecx: u32 = 0;
    var edx: u32 = 0;

    asm volatile (
        \\ cpuid
        : [eax] "={eax}" (eax),
          [ebx] "={ebx}" (ebx),
          [ecx] "={ecx}" (ecx),
          [edx] "={edx}" (edx),
        : [leaf] "{eax}" (leaf),
          [subleaf] "{ecx}" (subleaf),
        : "memory"
    );

    return Cpuid{ .eax = eax, .ebx = ebx, .ecx = ecx, .edx = edx };
}

fn main() {
    let outdir = std::env::var("OUT_DIR").expect("failed to get OUT_DIR");

    // nasm -f elf64 -o $OUTDIR/syscall_handler_asm_stub.o src/syscall_handler_asm_stub.s
    std::process::Command::new("nasm")
        .args(&["-f", "elf64", "-o"])
        .arg(format!("{}/syscall_handler_asm_stub.o", outdir))
        .arg("src/syscall_handler_asm_stub.s")
        .status()
        .expect("failed to execute nasm");
    println!("cargo:rerun-if-changed=src/syscall_handler_asm_stub.s");

    // ar rcs $OUTDIR/libsyscall_handler_asm_stub.a $OUTDIR/syscall_handler_asm_stub.o
    std::process::Command::new("ar")
        .args(&["rcs"])
        .arg(format!("{}/libsyscall_handler_asm_stub.a", outdir))
        .arg(format!("{}/syscall_handler_asm_stub.o", outdir))
        .status()
        .expect("failed to execute ar");

    // link the syscall_handler_asm_stub.o file
    println!("cargo:rustc-link-search={}", outdir);
    println!("cargo:rustc-link-lib=static=syscall_handler_asm_stub");

    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}

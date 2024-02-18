use std::ops::Not;

fn main() {
    let outdir = std::env::var("OUT_DIR").expect("failed to get OUT_DIR");

    // nasm -f elf64 -o $OUTDIR/syscall_handler_asm_stub.o src/syscall_handler_asm_stub.s
    std::process::Command::new("nasm")
        .args(["-f", "elf64", "-o"])
        .arg(format!("{outdir}/syscall_handler_asm_stub.o"))
        .arg("src/syscall_handler_asm_stub.s")
        .status()
        .expect("failed to execute nasm")
        .success()
        .not()
        .then(|| panic!("nasm failed"));

    println!("cargo:rerun-if-changed=src/syscall_handler_asm_stub.s");

    // ar rcs $OUTDIR/libsyscall_handler_asm_stub.a $OUTDIR/syscall_handler_asm_stub.o
    std::process::Command::new("ar")
        .args(["rcs"])
        .arg(format!("{outdir}/libsyscall_handler_asm_stub.a"))
        .arg(format!("{outdir}/syscall_handler_asm_stub.o"))
        .status()
        .expect("failed to execute ar")
        .success()
        .not()
        .then(|| panic!("ar failed"));

    // link the syscall_handler_asm_stub.o file
    println!("cargo:rustc-link-search={outdir}");
    println!("cargo:rustc-link-lib=static=syscall_handler_asm_stub");

    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}

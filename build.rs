use std::ops::Not;

fn main() {
    let outdir = std::env::var("OUT_DIR").expect("failed to get OUT_DIR");

    // nasm -f elf64 -o $OUTDIR/assembly.o src/assembly.s
    std::process::Command::new("nasm")
        .args(["-f", "elf64", "-o"])
        .arg(format!("{outdir}/assembly.o"))
        .arg("src/assembly.s")
        .status()
        .expect("failed to execute nasm")
        .success()
        .not()
        .then(|| panic!("nasm failed"));

    println!("cargo:rerun-if-changed=src/assembly.s");

    // ar rcs $OUTDIR/assembly.a $OUTDIR/assembly.o
    std::process::Command::new("ar")
        .args(["rcs"])
        .arg(format!("{outdir}/libassembly.a"))
        .arg(format!("{outdir}/assembly.o"))
        .status()
        .expect("failed to execute ar")
        .success()
        .not()
        .then(|| panic!("ar failed"));

    // link the libassembly.o file
    println!("cargo:rustc-link-search={outdir}");
    println!("cargo:rustc-link-lib=static=assembly");

    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}

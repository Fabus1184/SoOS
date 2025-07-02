fn main() {
    loop {
        println!("Hello World!");

        for i in 0..10_000_000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
}

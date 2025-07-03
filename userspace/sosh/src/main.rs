use ansi_term::{Color, Style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        Style::new()
            .fg(Color::Cyan)
            .paint("Welcome to sosh - the SoOS shell!")
    );

    for i in 0.. {
        println!(
            "{}",
            Style::new().fg(Color::Green).paint(format!("sosh> {i}"))
        );

        for _ in 0..10_000_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }

    Ok(())
}

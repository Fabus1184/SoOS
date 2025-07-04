use std::io::Read;

use anstyle::{AnsiColor, Color, Style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}Welcome to sosh - the SoOS shell!{}",
        Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        anstyle::Reset
    );

    let mut stdin = std::io::stdin();

    loop {
        println!(
            "{}sosh>{} ",
            Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            anstyle::Reset,
        );

        'read_loop: loop {
            let mut buffer: Vec<u8> = vec![0; 64];
            let count = stdin.read(&mut buffer)?;
            for &byte in &buffer[..count] {
                match byte {
                    b'\n' => {
                        println!("You entered a newline character.");
                        break 'read_loop;
                    }
                    _ => {
                        print!("{}", byte as char);
                    }
                }
            }
        }
    }
}

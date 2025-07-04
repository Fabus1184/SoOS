use std::io::{Read, Write};

use anstyle::{AnsiColor, Color, Style};

const BANNER: &str = r#"

  /$$$$$$             /$$$$$$  /$$   /$$
 /$$__  $$           /$$__  $$| $$  | $$
| $$  \__/  /$$$$$$ | $$  \__/| $$  | $$
|  $$$$$$  /$$__  $$|  $$$$$$ | $$$$$$$$
 \____  $$| $$  \ $$ \____  $$| $$__  $$
 /$$  \ $$| $$  | $$ /$$  \ $$| $$  | $$
|  $$$$$$/|  $$$$$$/|  $$$$$$/| $$  | $$
 \______/  \______/  \______/ |__/  |__/

          - the SoOS shell -
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}{BANNER}{}",
        Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        anstyle::Reset
    );

    let mut stdin = std::io::stdin();

    loop {
        print!(
            "{}sosh>{} ",
            Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            anstyle::Reset,
        );
        std::io::stdout().flush()?;

        let mut command = Vec::<u8>::with_capacity(64);

        'read_loop: loop {
            let mut buffer: Vec<u8> = vec![0; 64];
            let count = stdin.read(&mut buffer)?;
            for &byte in &buffer[..count] {
                match byte {
                    b'\n' => break 'read_loop,
                    b'\x08' => {
                        command.pop();
                    }
                    _ => command.push(byte),
                }
            }
        }

        let command = str::from_utf8(&command).expect("Invalid UTF-8 sequence");

        let command = command.split(" ").collect::<Vec<&str>>();
        if command.is_empty() {
            continue;
        }

        match BUILTINS.iter().find(|c| c.name == command[0]) {
            Some(builtin) => {
                (builtin.handler)(&command).expect("Failed to execute builtin command");
            }
            None => {
                println!(
                    "{}Error: Command '{}' not found. Type 'help' for a list of commands.{}",
                    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
                    command[0],
                    anstyle::Reset
                );
            }
        }
    }
}

struct BuiltinCommand {
    name: &'static str,
    description: &'static str,
    handler: fn(args: &[&str]) -> anyhow::Result<()>,
}

const BUILTINS: &[BuiltinCommand] = &[
    BuiltinCommand {
        name: "help",
        description: "display help message",
        handler: |_| {
            for cmd in BUILTINS {
                println!("{}: {}", cmd.name, cmd.description);
            }
            Ok(())
        },
    },
    BuiltinCommand {
        name: "exit",
        description: "Exit the shell",
        handler: |_| {
            std::process::exit(0);
        },
    },
];

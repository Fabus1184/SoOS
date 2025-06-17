use crate::term::{Color, Term};

pub struct Writer<'a> {
    term: &'a Term,
}

impl<'a> Writer<'a> {
    pub fn new(term: &'a Term) -> Self {
        Self { term }
    }
}

impl core::fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut state_machine = vte::Parser::new();
        state_machine.advance(self, s.as_bytes());

        Ok(())
    }
}

impl vte::Perform for Writer<'_> {
    fn print(&mut self, c: char) {
        self.term.putchar(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // new line
            b'\n' => {
                self.term.x.set(0);
                self.term.y.update(|x| x + 1);
            }
            // carriage return
            b'\r' => {
                self.term.x.set(0);
            }
            // backspace
            b'\x08' => {
                self.term.x.update(|x| x.saturating_sub(1));
            }
            c => panic!("Unhandled control character: {}", c),
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let params = params.iter().collect::<heapless::Vec<&[u16], 4>>();

        match (action, params.as_slice()) {
            // Clear screen
            ('J', [[2]]) => {
                self.term.clear();
                self.term.x.set(0);
                self.term.y.set(0);
            }
            // Move cursor to position
            ('H', [[0]]) => {
                self.term.x.set(0);
                self.term.y.set(0);
            }
            ('H', &[&[y], &[x]]) if x > 0 && y > 0 => {
                self.term.x.set((x - 1) as usize); // Convert to zero-based index
                self.term.y.set((y - 1) as usize); // Convert to zero-based index
            }
            // Text attributes
            ('m', [[0]]) => {
                self.term.fg.set(Color::WHITE); // Reset foreground color to white
                self.term.bg.set(Color::BLACK); // Reset background color to black
            }
            ('m', &[&[fg]]) if (30..=37).contains(&fg) => {
                self.term.fg.set(match fg {
                    30 => Color::BLACK,
                    31 => Color::RED,
                    32 => Color::GREEN,
                    33 => Color::YELLOW,
                    34 => Color::BLUE,
                    35 => Color::MAGENTA,
                    36 => Color::CYAN,
                    37 => Color::WHITE,
                    _ => unreachable!(),
                });
            }
            ('m', &[&[bg]]) if (40..=47).contains(&bg) => {
                self.term.bg.set(match bg {
                    40 => Color::BLACK,
                    41 => Color::RED,
                    42 => Color::GREEN,
                    43 => Color::YELLOW,
                    44 => Color::BLUE,
                    45 => Color::MAGENTA,
                    46 => Color::CYAN,
                    47 => Color::WHITE,
                    _ => unreachable!(),
                });
            }
            _ => panic!("Unhandled CSI action: {} with params: {:?}", action, params),
        }
    }
}

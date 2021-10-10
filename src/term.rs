use std::io::{Stderr, Stdin, Stdout, Write};

use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

#[derive(Debug)]
pub struct Term<R, W> {
    stdin: R,
    stdout: W,
}

impl Term<Stdin, RawTerminal<Stdout>> {
    pub fn stdout() -> std::io::Result<Self> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout().into_raw_mode()?;
        Ok(Self { stdin, stdout })
    }
}

impl Term<Stdin, RawTerminal<Stderr>> {
    pub fn stderr() -> std::io::Result<Self> {
        let stdin = std::io::stdin();
        let stdout = std::io::stderr().into_raw_mode()?;
        Ok(Self { stdin, stdout })
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    items: Vec<String>,
    default: usize,
}

impl Menu {
    pub fn new(items: Vec<String>) -> Self {
        Self { items, default: 0 }
    }

    pub fn default(&mut self, default: usize) -> &mut Self {
        self.default = default;
        self
    }

    pub fn interact(&self) -> std::io::Result<usize> {
        let mut term = Term::stderr()?;
        write!(term.stdout, "{}", cursor::Hide)?;
        let mut selected = self.default.min(self.items.len());
        let mut keys = term.stdin.keys();
        loop {
            self.draw(&mut term.stdout, selected)?;
            let key = match keys.next() {
                Some(key) => key?,
                None => break,
            };
            write!(
                term.stdout,
                "{}{}",
                cursor::Up(self.items.len() as _),
                clear::AfterCursor
            )?;
            match key {
                Key::Up => selected = selected.saturating_sub(1),
                Key::Down => selected = (selected + 1).min(self.items.len() - 1),
                Key::Char('\n') => break,
                _ => {}
            }
        }
        write!(term.stdout, "{}", cursor::Show)?;
        Ok(selected)
    }

    fn draw<W: Write>(&self, mut stdout: W, selected: usize) -> std::io::Result<()> {
        let mut output = vec![];
        for (i, item) in self.items.iter().enumerate() {
            if i == selected {
                writeln!(
                    output,
                    "{}❯ {}{}\r",
                    color::Fg(color::LightCyan),
                    item,
                    termion::style::Reset
                )?;
            } else {
                writeln!(output, "  {}\r", item)?;
            }
        }
        stdout.write_all(&output)?;
        stdout.flush()?;
        Ok(())
    }
}

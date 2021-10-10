use std::io::{Read, Write};

use termion::{clear, color, cursor, event::Key, input::TermRead};

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

    pub fn interact<R, W>(&self, stdin: R, mut stdout: W) -> std::io::Result<usize>
    where
        R: Read,
        W: Write,
    {
        write!(stdout, "{}", cursor::Hide)?;
        let mut selected = self.default.min(self.items.len());
        let mut keys = stdin.keys();
        loop {
            self.draw(&mut stdout, selected)?;
            let key = match keys.next() {
                Some(key) => key?,
                None => break,
            };
            write!(
                stdout,
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
        write!(stdout, "{}", cursor::Show)?;
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

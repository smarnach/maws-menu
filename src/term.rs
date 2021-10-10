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
pub struct MenuItem {
    label: String,
    shortcut: Option<char>,
}

impl MenuItem {
    fn new(label: impl Into<String>, shortcut: Option<char>) -> Self {
        let label = label.into();
        Self { label, shortcut }
    }

    fn draw<W: Write>(&self, mut stdout: W, selected: bool) -> std::io::Result<()> {
        if selected {
            write!(stdout, "{}❯ ", color::Fg(color::LightCyan))?;
        } else {
            write!(stdout, "  ")?;
        }
        if let Some(c) = self.shortcut {
            write!(stdout, "[{}] ", c)?;
        } else {
            write!(stdout, "    ")?;
        }
        write!(stdout, "{}\r\n", self.label)?;
        if selected {
            write!(stdout, "{}", termion::style::Reset)?;
        }
        Ok(())
    }
}

impl<T: Into<String>> From<(T, Option<char>)> for MenuItem {
    fn from((label, shortcut): (T, Option<char>)) -> Self {
        Self::new(label, shortcut)
    }
}

#[derive(Clone, Debug)]
pub struct Menu {
    items: Vec<MenuItem>,
    default: usize,
}

impl Menu {
    pub fn new<I>(items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<MenuItem>,
    {
        let items = items.into_iter().map(Into::into).collect();
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
        'outer: loop {
            self.draw(&mut term.stdout, selected)?;
            let key = match keys.next() {
                Some(key) => key?,
                None => break,
            };
            self.clear(&mut term.stdout)?;
            match key {
                Key::Up => selected = selected.saturating_sub(1),
                Key::Down => selected = (selected + 1).min(self.items.len() - 1),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    for (i, item) in self.items.iter().enumerate() {
                        if item.shortcut == Some(c) {
                            selected = i;
                            break 'outer;
                        }
                    }
                }
                _ => {}
            }
        }
        write!(term.stdout, "{}", cursor::Show)?;
        Ok(selected)
    }

    fn draw<W: Write>(&self, mut stdout: W, selected: usize) -> std::io::Result<()> {
        let mut output = vec![];
        for (i, item) in self.items.iter().enumerate() {
            item.draw(&mut output, i == selected)?;
        }
        stdout.write_all(&output)?;
        stdout.flush()?;
        Ok(())
    }

    fn clear<W: Write>(&self, mut stdout: W) -> std::io::Result<()> {
        write!(
            stdout,
            "{}{}",
            cursor::Up(self.items.len() as _),
            clear::AfterCursor
        )
    }
}

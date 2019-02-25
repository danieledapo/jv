use crate::widgets::ascii_line::AsciiLine;
use crate::widgets::view::Line;
use crate::widgets::Renderable;

use std::io;

use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;

pub struct StatusLine {
    // 0-based
    cursor_row: u16,
    cursor_col: u16,

    width: u16,

    buffer: AsciiLine<String>,
}

impl StatusLine {
    pub fn new(cursor_row: u16, width: u16) -> StatusLine {
        StatusLine {
            cursor_row,
            cursor_col: 0,
            width,
            buffer: AsciiLine { l: String::new() },
        }
    }

    pub fn activate(&mut self) {
        self.clear();
        self.insert(':');
    }

    pub fn insert(&mut self, c: char) {
        if !c.is_ascii() {
            return;
        }

        self.buffer.l.insert(usize::from(self.cursor_col), c);
        self.cursor_col += 1;
    }

    pub fn remove(&mut self) {
        self.cursor_col -= 1;
        self.buffer.l.remove(usize::from(self.cursor_col));
    }

    pub fn clear(&mut self) {
        self.buffer.l.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn left(&mut self) {
        // TODO: handle page movement
        self.cursor_col = self.cursor_col.saturating_sub(1).max(1);
    }

    pub fn right(&mut self) {
        // TODO: handle page movement
        self.cursor_col = usize::from(self.cursor_col + 1).min(self.buffer.len()) as u16;
    }
}

impl Renderable for StatusLine {
    fn render(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        write!(
            term,
            "{}{}{}{}",
            cursor::Goto(1, self.cursor_row + 1),
            color::Bg(color::AnsiValue::grayscale(4)),
            clear::CurrentLine,
            self.buffer.render(0, usize::from(self.width)),
        )?;

        term.flush()
    }

    fn focus(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        write!(
            term,
            "{}",
            cursor::Goto(self.cursor_col + 1, self.cursor_row + 1)
        )?;
        term.flush()
    }
}

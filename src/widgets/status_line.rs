use crate::widgets::ascii_line::AsciiLine;
use crate::widgets::view::Line;
use crate::widgets::Widget;

use std::io;

use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;

// TODO: handle tabs
pub struct StatusLine {
    frame_start_col: usize,

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
            frame_start_col: 0,
            width,
            buffer: AsciiLine { l: String::new() },
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer.l[1..]
    }

    pub fn activate(&mut self) {
        self.clear();
        self.insert(':');
    }

    pub fn insert(&mut self, c: char) {
        if !c.is_ascii() {
            return;
        }

        self.buffer
            .l
            .insert(self.frame_start_col + usize::from(self.cursor_col), c);
        self.right();
    }

    pub fn remove(&mut self) {
        self.buffer
            .l
            .remove(self.frame_start_col + usize::from(self.cursor_col) - 1);
        self.left();
    }

    pub fn clear(&mut self) {
        self.buffer.l.clear();
        self.cursor_col = 0;
        self.frame_start_col = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn left(&mut self) {
        if self.cursor_col == 0 {
            let text_width = self.width;

            if self.frame_start_col != 0 {
                self.cursor_col = text_width / 2;
            }

            self.frame_start_col = self
                .frame_start_col
                .saturating_sub(usize::from(text_width) / 2 + 1);
        } else {
            self.cursor_col -= 1;
        }

        if self.frame_start_col == 0 && self.cursor_col == 0 {
            self.cursor_col = 1;
        }
    }

    pub fn right(&mut self) {
        let row_len = self.buffer.chars_count();

        if self.frame_start_col + usize::from(self.cursor_col) < row_len {
            self.cursor_col += 1;
        }

        let text_width = self.width;
        if self.cursor_col >= text_width {
            self.frame_start_col += usize::from(text_width) / 2 + 1;
            self.cursor_col /= 2;
        }
    }
}

impl Widget for StatusLine {
    fn render(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        write!(
            term,
            "{}{}{}{}",
            cursor::Goto(1, self.cursor_row + 1),
            color::Bg(color::AnsiValue::grayscale(4)),
            clear::CurrentLine,
            self.buffer
                .render(self.frame_start_col, usize::from(self.width)),
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

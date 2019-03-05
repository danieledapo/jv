use crate::widgets::ascii_line::AsciiLine;
use crate::widgets::view::Line;
use crate::widgets::Widget;

use std::io;

use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;

#[derive(Debug, Clone, Copy)]
pub enum StatusLineMode {
    Query,
    Command,
}

#[derive(Debug)]
pub struct StatusLine {
    frame_start_col: usize,
    col_char_ix: usize,

    // 0-based
    cursor_row: u16,
    cursor_col: u16,

    width: u16,

    buffer: AsciiLine<String>,
    mode: StatusLineMode,
}

impl StatusLine {
    pub fn new(cursor_row: u16, width: u16) -> StatusLine {
        StatusLine {
            cursor_row,
            cursor_col: 0,
            frame_start_col: 0,
            col_char_ix: 0,
            mode: StatusLineMode::Command,
            width,
            buffer: AsciiLine::new(String::new()).unwrap(),
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer.line()[1..]
    }

    pub fn mode(&self) -> StatusLineMode {
        self.mode
    }

    pub fn activate(&mut self, mode: StatusLineMode) {
        self.clear();

        self.mode = mode;
        match self.mode {
            StatusLineMode::Command => self.insert(':'),
            StatusLineMode::Query => self.insert('#'),
        }
    }

    pub fn insert(&mut self, c: char) {
        if !c.is_ascii() {
            return;
        }

        self.buffer.insert(self.col_char_ix, c);
        self.right();
    }

    pub fn remove(&mut self) {
        self.buffer.remove(self.col_char_ix - 1);
        self.left();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_col = 0;
        self.frame_start_col = 0;
        self.col_char_ix = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.chars_count() == 0
    }

    pub fn left(&mut self) {
        if self.col_char_ix <= 1 {
            return;
        }

        self.col_char_ix -= 1;

        self.center_horizontally();
    }

    pub fn right(&mut self) {
        let row_len = self.buffer.chars_count();

        if self.col_char_ix >= row_len {
            return;
        }

        self.col_char_ix += 1;

        self.center_horizontally();
    }

    fn center_horizontally(&mut self) {
        self.frame_start_col = self.col_char_ix;

        let mut w = 0;
        while self.frame_start_col > 0 {
            let cw = self.buffer.char_width(self.frame_start_col);

            if w + cw >= self.width {
                break;
            }

            w += cw;
            self.frame_start_col -= 1;
        }

        self.cursor_col = w + self.buffer.char_width(self.frame_start_col) - 1;
    }
}

impl Widget for StatusLine {
    fn render(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        let mode_line = match self.mode {
            StatusLineMode::Command => AsciiLine::new(" NORMAL ").unwrap(),
            StatusLineMode::Query => AsciiLine::new(" QUERY ").unwrap(),
        };

        writeln!(
            term,
            "{}{}{}{}{}{}{}{}",
            cursor::Goto(1, self.cursor_row + 1),
            color::Bg(color::AnsiValue::grayscale(6)),
            color::Fg(color::Black),
            clear::CurrentLine,
            color::Bg(color::LightBlue),
            mode_line.render(0, usize::from(self.width)),
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        )?;
        write!(
            term,
            "{}{}{}{}{}",
            cursor::Goto(1, self.cursor_row + 2),
            color::Bg(color::AnsiValue::grayscale(4)),
            color::Fg(color::Reset),
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
            cursor::Goto(self.cursor_col + 1, self.cursor_row + 2)
        )?;
        term.flush()
    }
}

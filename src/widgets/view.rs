use std::io;
use std::io::Write;

use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;

use crate::widgets::Widget;

/// `Line` is a line that can be rendered by a `View`.
pub trait Line {
    /// Render this line starting from the given column spanning for a given
    /// width. If the rendered line is shorter than `start_col` then it must
    /// return the empty string.
    fn render(&self, start_col: usize, width: usize) -> String;

    /// Return the number of the visible characters that compose the string.
    /// This function must not take into account the markup that's added into
    /// the rendered string nor the character width. As of now, only ASCII
    /// characters are supported because Unicode is hard to get right.
    fn chars_count(&self) -> usize;

    /// Return the number of columns the char at the given positions spans.
    fn char_width(&self, idx: usize) -> u16;
}

/// A read-only view over some lines.
pub struct View<L> {
    lines: Vec<L>,

    width: u16,
    height: u16,
    num_lines_padding: usize,

    line_char_ix: usize,
    max_line_char_ix: usize,

    frame_start_row: usize,
    frame_start_char_ix: usize,

    // these are 0-based even though the terminal uses 1-based coordinates
    cursor_row: u16,
    cursor_col: u16,
}

impl<L> View<L>
where
    L: Line,
{
    /// Create a new `View` to print the given lines over the raw terminal
    /// with the given size.
    pub fn new(size: (u16, u16), lines: impl IntoIterator<Item = L>) -> Self {
        let lines = lines.into_iter().collect::<Vec<L>>();
        let num_lines_padding = lines.len().to_string().len();

        View {
            lines,
            num_lines_padding,
            cursor_col: 0,
            cursor_row: 0,
            line_char_ix: 0,
            frame_start_char_ix: 0,
            frame_start_row: 0,
            height: size.1,
            max_line_char_ix: 0,
            width: size.0,
        }
    }

    // Move the cursor one character to the right.
    pub fn move_right(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        let row = &self.lines[self.frame_start_row + usize::from(self.cursor_row)];

        if self.line_char_ix + 1 >= row.chars_count() {
            return;
        }

        self.line_char_ix += 1;
        self.max_line_char_ix = self.line_char_ix;

        self.center_horizontally();
    }

    // Move the cursor one character to the left.
    pub fn move_left(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        if self.line_char_ix == 0 {
            return;
        }

        self.line_char_ix -= 1;
        self.max_line_char_ix = self.line_char_ix;

        self.center_horizontally();
    }

    // Move the cursor up one row.
    pub fn move_up(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        if self.cursor_row == 0 {
            self.frame_start_row = self.frame_start_row.saturating_sub(1);
        } else {
            self.cursor_row -= 1;
        }

        self.cap_line_char_ix();
        self.center_horizontally();
    }

    // Move the cursor down one row.
    pub fn move_down(&mut self) {
        if self.frame_start_row + usize::from(self.cursor_row) + 1 >= self.lines.len() {
            return;
        }

        self.cursor_row =
            (usize::from(self.cursor_row + 1)).min(self.lines.len().saturating_sub(1)) as u16;

        if self.cursor_row >= self.height {
            self.cursor_row = self.height - 1;
            self.frame_start_row =
                (self.frame_start_row + 1).min(self.lines.len().saturating_sub(1));
        }

        self.cap_line_char_ix();
        self.center_horizontally();
    }

    /// Move to beginning of current line.
    pub fn move_to_sol(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        self.max_line_char_ix = 0;
        self.line_char_ix = 0;

        self.center_horizontally();
    }

    /// Move to end of current line.
    pub fn move_to_eol(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        self.line_char_ix = self.lines[self.frame_start_row + usize::from(self.cursor_row)]
            .chars_count()
            .saturating_sub(1);
        self.max_line_char_ix = self.line_char_ix;

        self.center_horizontally();
    }

    /// Move one page up.
    pub fn page_up(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        if self.frame_start_row == 0 {
            self.cursor_row = 0;
        } else {
            self.frame_start_row = self
                .frame_start_row
                .saturating_sub(usize::from(self.height));
        }

        self.cap_line_char_ix();
        self.center_horizontally();
    }

    /// Move one page down.
    pub fn page_down(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        self.frame_start_row += usize::from(self.height);
        if self.frame_start_row + usize::from(self.cursor_row) >= self.lines.len() {
            self.frame_start_row = self.lines.len() - 1;
            self.cursor_row = 0;
        }

        self.cap_line_char_ix();
        self.center_horizontally();
    }

    /// Goto 0 based row and column.
    pub fn goto(&mut self, r: usize, c: usize) {
        if self.lines.is_empty() {
            return;
        }

        let r = r.min(self.lines.len().saturating_sub(1));
        if r < self.frame_start_row || r >= self.frame_start_row + usize::from(self.height) {
            self.frame_start_row = r.saturating_sub(usize::from(self.height) / 2 - 1);
        }

        self.cursor_row = r.saturating_sub(self.frame_start_row) as u16;

        let c = c.min(
            self.lines[self.frame_start_row + usize::from(self.cursor_row)]
                .chars_count()
                .saturating_sub(1),
        );
        self.max_line_char_ix = c;

        self.cap_line_char_ix();
        self.center_horizontally();
    }

    fn cap_line_char_ix(&mut self) {
        self.line_char_ix = self.max_line_char_ix.min(
            self.lines[self.frame_start_row + usize::from(self.cursor_row)]
                .chars_count()
                .saturating_sub(1),
        );
    }

    fn center_horizontally(&mut self) {
        let text_width = usize::from(self.width) - self.num_column_width();

        let row = &self.lines[self.frame_start_row + usize::from(self.cursor_row)];
        let row_len = row.chars_count();

        let c = self.max_line_char_ix.min(row_len.saturating_sub(1));

        let min_start_char_ix = if c < self.frame_start_char_ix + usize::from(self.width) {
            self.frame_start_char_ix
        } else {
            0
        };

        self.frame_start_char_ix = c;

        let mut w = 0;
        while self.frame_start_char_ix > min_start_char_ix {
            let cw = row.char_width(self.frame_start_char_ix);

            if usize::from(w) + usize::from(cw) >= text_width {
                break;
            }

            w += cw;
            self.frame_start_char_ix -= 1;
        }

        self.cursor_col = w;
    }

    fn num_column_width(&self) -> usize {
        // +3 is because after the line number we show " | "
        self.num_lines_padding + 3
    }
}

impl<L> Widget for View<L>
where
    L: Line,
{
    fn render(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        let fg = color::Fg(color::AnsiValue::grayscale(4));
        let bg = color::Bg(color::AnsiValue::grayscale(4));
        let highlighted_bg = color::Bg(color::AnsiValue::grayscale(6));
        let num_fg = color::Fg(color::AnsiValue::grayscale(7));
        let highlighted_num_fg = color::Fg(color::LightCyan);

        write!(term, "{}{}", cursor::Hide, cursor::Goto(1, 1))?;

        let text_width = usize::from(self.width) - self.num_column_width();

        // always redraw all the lines possibly clearing them
        for i in 0..self.height {
            let r = self.frame_start_row + usize::from(i);

            match self.lines.get(r) {
                None => write!(
                    term,
                    "{}{}{}{:nlp$} │",
                    bg,
                    clear::CurrentLine,
                    num_fg,
                    '~',
                    nlp = self.num_lines_padding
                )?,
                Some(l) => {
                    if self.cursor_row == i {
                        write!(
                            term,
                            "{}{}{}{:>nlp$}{} │ {}{}",
                            highlighted_bg,
                            clear::CurrentLine,
                            highlighted_num_fg,
                            r + 1,
                            fg,
                            color::Fg(color::Reset),
                            l.render(self.frame_start_char_ix, text_width),
                            nlp = self.num_lines_padding,
                        )?
                    } else {
                        write!(
                            term,
                            "{}{}{}{:>nlp$} │ {}{}",
                            bg,
                            clear::CurrentLine,
                            num_fg,
                            r + 1,
                            color::Fg(color::Reset),
                            l.render(self.frame_start_char_ix, text_width),
                            nlp = self.num_lines_padding,
                        )?
                    }
                }
            }

            if i < self.height - 1 {
                write!(term, "\n\r")?;
            }
        }

        write!(term, "{}", cursor::Show)?;
        term.flush()?;

        Ok(())
    }

    fn focus(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
        let c = self.cursor_col + 1 + self.num_column_width() as u16;
        let r = self.cursor_row + 1;

        write!(term, "{}", cursor::Goto(c, r))?;

        term.flush()
    }
}

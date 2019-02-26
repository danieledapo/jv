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

    /// Return the length of the visible characters that compose the string.
    /// This function must not take into account the markup that's added into
    /// the rendered string. As of now, only ASCII characters are supported
    /// because Unicode is hard to get right.
    fn unstyled_chars_len(&self) -> usize;
}

/// A read-only view over some lines.
pub struct View<L> {
    lines: Vec<L>,

    width: u16,
    height: u16,

    frame_start_row: usize,
    frame_start_col: usize,
    num_lines_padding: usize,
    max_col: usize,

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
            frame_start_col: 0,
            frame_start_row: 0,
            height: size.1,
            max_col: 0,
            width: size.0,
        }
    }

    // Move the cursor one character to the right.
    pub fn move_right(&mut self) {
        let row_len =
            self.lines[self.frame_start_row + usize::from(self.cursor_row)].unstyled_chars_len();

        if self.frame_start_col + usize::from(self.cursor_col) + 1 < row_len {
            self.cursor_col += 1;
        }

        let text_width = self.width - self.num_column_width() as u16;
        if self.cursor_col >= text_width {
            self.frame_start_col += usize::from(text_width) / 2 + 1;
            self.cursor_col /= 2;
        }

        self.max_col = self.frame_start_col + usize::from(self.cursor_col);
    }

    // Move the cursor one character to the left.
    pub fn move_left(&mut self) {
        if self.cursor_col == 0 {
            let text_width = self.width - self.num_column_width() as u16;

            if self.frame_start_col != 0 {
                self.cursor_col = text_width / 2;
            }

            self.frame_start_col = self
                .frame_start_col
                .saturating_sub(usize::from(text_width) / 2 + 1);
        } else {
            self.cursor_col -= 1;
        }

        self.max_col = self.frame_start_col + usize::from(self.cursor_col);
    }

    // Move the cursor up one row.
    pub fn move_up(&mut self) {
        if self.cursor_row == 0 {
            self.frame_start_row = self.frame_start_row.saturating_sub(1);
        } else {
            self.cursor_row -= 1;
        }

        self.fix_cursor_col_after_vertical_move();
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

        self.fix_cursor_col_after_vertical_move();
    }

    /// Move to beginning of current line.
    pub fn move_to_sol(&mut self) {
        self.max_col = 0;
        self.fix_cursor_col_after_vertical_move();
    }

    /// Move to end of current line.
    pub fn move_to_eol(&mut self) {
        self.max_col =
            self.lines[self.frame_start_row + usize::from(self.cursor_row)].unstyled_chars_len();

        self.fix_cursor_col_after_vertical_move();
    }

    /// Move one page up.
    pub fn page_up(&mut self) {
        if self.frame_start_row == 0 {
            self.cursor_row = 0;
        } else {
            self.frame_start_row = self
                .frame_start_row
                .saturating_sub(usize::from(self.height));
        }

        self.fix_cursor_col_after_vertical_move();
    }

    /// Move one page down.
    pub fn page_down(&mut self) {
        self.frame_start_row += usize::from(self.height);
        if self.frame_start_row + usize::from(self.cursor_row) >= self.lines.len() {
            self.frame_start_row = self.lines.len() - 1;
            self.cursor_row = 0;
        }

        self.fix_cursor_col_after_vertical_move();
    }

    fn fix_cursor_col_after_vertical_move(&mut self) {
        let row_len =
            self.lines[self.frame_start_row + usize::from(self.cursor_row)].unstyled_chars_len();

        let text_width = usize::from(self.width) - self.num_column_width();
        let c = self.max_col.min(row_len.saturating_sub(1));

        self.frame_start_col = c / text_width * text_width;
        self.cursor_col = c.saturating_sub(self.frame_start_col) as u16;
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
                None => write!(term, "{}{}", bg, clear::CurrentLine)?,
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
                            l.render(self.frame_start_col, text_width),
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
                            l.render(self.frame_start_col, text_width),
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

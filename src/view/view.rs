use std::io;
use std::io::Write;

use termion::clear;
use termion::cursor;
use termion::raw::RawTerminal;

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
pub struct View<W, L>
where
    W: io::Write,
{
    lines: Vec<L>,
    term: RawTerminal<W>,

    width: u16,
    height: u16,

    frame_start_row: usize,
    frame_start_col: usize,

    // these are 0-based even though the terminal uses 1-based coordinates
    cursor_row: u16,
    cursor_col: u16,
}

impl<W, L> View<W, L>
where
    W: io::Write,
    L: Line,
{
    /// Create a new `View` to print the given lines over the raw terminal
    /// with the given size.
    pub fn new(term: RawTerminal<W>, size: (u16, u16), lines: impl IntoIterator<Item = L>) -> Self {
        View {
            term,
            cursor_col: 0,
            cursor_row: 0,
            frame_start_col: 0,
            frame_start_row: 0,
            height: size.1,
            lines: lines.into_iter().collect(),
            width: size.0,
        }
    }

    // TODO
    // pub fn resize(&mut self, size: (u16, u16)) {
    //     self.width = size.0;
    //     self.height = size.1;
    // }

    /// Clear the view.
    pub fn clear(&mut self) -> io::Result<()> {
        write!(self.term, "{}", clear::All)
    }

    /// Redraw all the screen.
    pub fn display(&mut self) -> io::Result<()> {
        write!(self.term, "{}{}", cursor::Hide, cursor::Goto(1, 1))?;

        // always redraw all the lines possibly clearing them
        for i in 0..self.height {
            match self.lines.get(self.frame_start_row + usize::from(i)) {
                None => write!(self.term, "{}", clear::CurrentLine)?,
                Some(l) => {
                    write!(
                        self.term,
                        "{}{}",
                        clear::CurrentLine,
                        l.render(self.frame_start_col, usize::from(self.width))
                    )?;
                }
            }

            if i < self.height - 1 {
                write!(self.term, "\n\r")?;
            }
        }

        write!(self.term, "{}", cursor::Show)?;
        self.show_cursor()?;

        Ok(())
    }

    // Move the cursor one character to the right.
    pub fn move_right(&mut self) -> io::Result<()> {
        if self.frame_start_col + usize::from(self.cursor_col) + 1
            < self.lines[self.frame_start_row + usize::from(self.cursor_row)].unstyled_chars_len()
        {
            self.cursor_col += 1;
        }

        if self.cursor_col >= self.width {
            self.frame_start_col += usize::from(self.width / 2) + 1;
            self.cursor_col /= 2;
        }

        self.display()
    }

    // Move the cursor one character to the left.
    pub fn move_left(&mut self) -> io::Result<()> {
        if self.cursor_col == 0 {
            if self.frame_start_col == 0 {
                self.cursor_col = 0;
            } else {
                self.cursor_col = self.width / 2;
            }

            self.frame_start_col = self
                .frame_start_col
                .saturating_sub(usize::from(self.width / 2) + 1);
        } else {
            self.cursor_col -= 1;
        }

        self.display()
    }

    // Move the cursor up one row.
    pub fn move_up(&mut self) -> io::Result<()> {
        if self.cursor_row == 0 {
            self.cursor_row = 0;
            self.frame_start_row = self.frame_start_row.saturating_sub(1);
        } else {
            self.cursor_row -= 1;
        }

        self.cursor_col = self.lines[self.frame_start_row + usize::from(self.cursor_row)]
            .unstyled_chars_len()
            .saturating_sub(1)
            .min(usize::from(self.cursor_col)) as u16;

        self.display()
    }

    // Move the cursor down one row.
    pub fn move_down(&mut self) -> io::Result<()> {
        if self.frame_start_row + usize::from(self.cursor_row) + 1 >= self.lines.len() {
            return Ok(());
        }

        self.cursor_row =
            (usize::from(self.cursor_row + 1)).min(self.lines.len().saturating_sub(1)) as u16;

        if self.cursor_row >= self.height {
            self.cursor_row = self.height - 1;
            self.frame_start_row =
                (self.frame_start_row + 1).min(self.lines.len().saturating_sub(1));
        }

        self.cursor_col = self.lines[self.frame_start_row + usize::from(self.cursor_row)]
            .unstyled_chars_len()
            .saturating_sub(1)
            .min(usize::from(self.cursor_col)) as u16;

        self.display()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(
            self.term,
            "{}",
            cursor::Goto(self.cursor_col + 1, self.cursor_row + 1)
        )?;
        self.term.flush()?;

        Ok(())
    }
}

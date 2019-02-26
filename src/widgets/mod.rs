pub mod ascii_line;
pub mod status_line;
pub mod view;

use std::io;
use termion::raw::RawTerminal;

pub trait Widget {
    fn render(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()>;
    fn focus(&self, term: &mut RawTerminal<impl io::Write>) -> io::Result<()>;
}

use std::env;
use std::fs;
use std::io;
use std::io::Read;

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use jv::view::ascii_line::AsciiLine;
use jv::view::View;

fn main() -> io::Result<()> {
    let mut f = fs::File::open(env::args().nth(1).unwrap())?;

    let mut input = String::new();
    f.read_to_string(&mut input)?;

    let stdout = io::stdout().into_raw_mode()?;
    let size = termion::terminal_size()?;

    let mut view = View::new(
        stdout,
        size,
        input.lines().map(|l| AsciiLine::new(l).unwrap()),
    );

    view.clear()?;
    view.display()?;

    for ev in io::stdin().events() {
        match ev? {
            Event::Key(Key::Char('q')) => break,
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) => view.move_right()?,
            Event::Key(Key::Left) | Event::Key(Key::Char('h')) => view.move_left()?,
            Event::Key(Key::Up) | Event::Key(Key::Char('k')) => view.move_up()?,
            Event::Key(Key::Down) | Event::Key(Key::Char('j')) => view.move_down()?,
            _ => {}
        }
    }

    view.clear()?;

    Ok(())
}

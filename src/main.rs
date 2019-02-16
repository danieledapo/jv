use std::env;
use std::fs;
use std::io;
use std::io::Read;

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use jv::json::parse_json;
use jv::view::ascii_line::AsciiLine;
use jv::view::{Line, View};

fn main() -> io::Result<()> {
    let file_path = env::args().nth(1).unwrap();
    let mut f = fs::File::open(&file_path)?;

    if file_path.ends_with("json") {
        run(parse_json(f).unwrap())?;
    } else {
        let mut input = String::new();
        f.read_to_string(&mut input)?;

        run(input.lines().map(|l| AsciiLine::new(l).unwrap()))?;
    }

    Ok(())
}

fn run(lines: impl IntoIterator<Item = impl Line>) -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let size = termion::terminal_size()?;

    let mut view = View::new(stdout, size, lines);

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

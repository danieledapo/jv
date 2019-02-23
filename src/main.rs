use std::env;
use std::fs;
use std::io;
use std::io::Read;

use termion::event::Key;
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

    for ev in io::stdin().keys() {
        match ev? {
            Key::Char('q') => break,
            Key::Right | Key::Char('l') => view.move_right()?,
            Key::Left | Key::Char('h') => view.move_left()?,
            Key::Up | Key::Char('k') => view.move_up()?,
            Key::Down | Key::Char('j') => view.move_down()?,
            Key::Char('0') => view.move_to_sol()?,
            Key::Char('$') => view.move_to_eol()?,
            Key::PageUp => view.page_up()?,
            Key::PageDown => view.page_down()?,
            _ => {}
        }
    }

    view.clear()?;

    Ok(())
}

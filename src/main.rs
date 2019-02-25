use std::env;
use std::fs;
use std::io;
use std::io::Read;

use termion::clear;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use jv::json::parse_json;
use jv::widgets::ascii_line::AsciiLine;
use jv::widgets::view::{Line, View};
use jv::widgets::Renderable;

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
    let mut stdout = io::stdout().into_raw_mode()?;
    let (width, height) = termion::terminal_size()?;

    let mut view = View::new((width, height), lines);

    clear(&mut stdout)?;
    view.render(&mut stdout)?;

    for ev in io::stdin().keys() {
        match ev? {
            Key::Char('q') => break,
            Key::Right | Key::Char('l') => view.move_right(),
            Key::Left | Key::Char('h') => view.move_left(),
            Key::Up | Key::Char('k') => view.move_up(),
            Key::Down | Key::Char('j') => view.move_down(),
            Key::Char('0') => view.move_to_sol(),
            Key::Char('$') => view.move_to_eol(),
            Key::PageUp => view.page_up(),
            Key::PageDown => view.page_down(),
            _ => {}
        }

        view.render(&mut stdout)?;
    }

    clear(&mut stdout)?;

    Ok(())
}

fn clear(term: &mut RawTerminal<impl io::Write>) -> io::Result<()> {
    write!(
        term,
        "{}{}{}",
        color::Fg(color::Reset),
        color::Bg(color::Reset),
        clear::All
    )
}

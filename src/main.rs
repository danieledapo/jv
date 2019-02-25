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
use jv::widgets::status_line::StatusLine;
use jv::widgets::view::{Line, View};
use jv::widgets::Renderable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Input,
}

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

    let mut mode = Mode::Normal;

    let mut view = View::new((width, height - 1), lines);
    let mut status_line = StatusLine::new(height - 1, width);

    clear(&mut stdout)?;
    status_line.render(&mut stdout)?;
    view.render(&mut stdout)?;
    view.focus(&mut stdout)?;

    for ev in io::stdin().keys() {
        match mode {
            Mode::Normal => match ev? {
                Key::Char('q') => break,
                Key::Right | Key::Char('l') => view.move_right(),
                Key::Left | Key::Char('h') => view.move_left(),
                Key::Up | Key::Char('k') => view.move_up(),
                Key::Down | Key::Char('j') => view.move_down(),
                Key::Char('0') => view.move_to_sol(),
                Key::Char('$') => view.move_to_eol(),
                Key::PageUp => view.page_up(),
                Key::PageDown => view.page_down(),
                Key::Char(':') => {
                    mode = Mode::Input;
                    status_line.activate();
                }
                _ => {}
            },

            Mode::Input => match ev? {
                Key::Esc => {
                    status_line.clear();
                    mode = Mode::Normal;
                }
                Key::Char('\n') => {
                    // TODO: something
                }
                Key::Char(c) => status_line.insert(c),
                Key::Backspace => {
                    status_line.remove();
                    if status_line.is_empty() {
                        mode = Mode::Normal;
                    }
                }
                Key::Left => status_line.left(),
                Key::Right => status_line.right(),
                _ => {}
            },
        }

        status_line.render(&mut stdout)?;
        view.render(&mut stdout)?;

        match mode {
            Mode::Normal => view.focus(&mut stdout)?,
            Mode::Input => status_line.focus(&mut stdout)?,
        }
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

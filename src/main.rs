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
use jv::widgets::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    View,
    StatusLine,
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

    let mut focus = Focus::View;

    let mut view = View::new((width, height - 1), lines);
    let mut status_line = StatusLine::new(height - 1, width);

    clear(&mut stdout)?;
    status_line.render(&mut stdout)?;
    view.render(&mut stdout)?;
    view.focus(&mut stdout)?;

    for ev in io::stdin().keys() {
        match focus {
            Focus::View => match ev? {
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
                    focus = Focus::StatusLine;
                    mode = Mode::Input;
                    status_line.activate();
                }
                _ => {}
            },

            Focus::StatusLine => match ev? {
                Key::Esc => {
                    status_line.clear();
                    focus = Focus::View;
                }
                Key::Char('\n') => {
                    if let Some((r, c)) = parse_goto(&status_line.text()) {
                        view.goto(r, c.unwrap_or(0));

                        status_line.clear();
                        focus = Focus::View;
                    }
                }
                Key::Char(c) => status_line.insert(c),
                Key::Backspace => {
                    status_line.remove();
                    if status_line.is_empty() {
                        status_line.clear();
                        focus = Focus::View;
                    }
                }
                Key::Left => status_line.left(),
                Key::Right => status_line.right(),
                _ => {}
            },
        }

        status_line.render(&mut stdout)?;
        view.render(&mut stdout)?;

        match focus {
            Focus::View => view.focus(&mut stdout)?,
            Focus::StatusLine => status_line.focus(&mut stdout)?,
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

fn parse_goto(input: &str) -> Option<(usize, Option<usize>)> {
    let mut parts = input.split(':').fuse();

    let r = parts.next()?.parse::<usize>().ok()?.saturating_sub(1);

    match parts.next() {
        None => Some((r, None)),
        Some(cs) => match cs.parse::<usize>().ok() {
            Some(c) => Some((r, Some(c.saturating_sub(1)))),
            None => None,
        },
    }
}

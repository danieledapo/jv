use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;

use termion::clear;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use jv::json::index::{index, Index};
use jv::json::{parse_json, JsonTokenTag};
use jv::widgets::ascii_line::AsciiLine;
use jv::widgets::status_line::{StatusLine, StatusLineMode};
use jv::widgets::view::{Line, View};
use jv::widgets::Widget;

/// Simple json viewer that allows querying and jumping to json values via
/// jq-like queries with format "#/<objectkey>/<arrayix>". An example query is
/// `#/authors/1` or `#/dependencies/react`.
///
/// You can write a query by entering query mode with `#` and writing the
/// desired query. Moreover, if the cursor is under a valid query text then you
/// can automatically jump to it with `ENTER`. If the input filename doesn't end
/// with ".json" then it's not treated as such and `jv` will simply work as a
/// viewer.
#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

struct Ui<L, W, Q>
where
    W: io::Write,
    Q: Fn(&View<L>) -> Option<String>,
{
    stdout: RawTerminal<W>,

    view: View<L>,
    status_line: StatusLine,

    focus: Focus,

    index: Index,
    get_current_query: Q,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    View,
    StatusLine,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    NotUnicode(String),
    Json(serde_json::Error),
}

fn main() {
    fn _main() -> Result<()> {
        let opts = Opts::from_args();

        let mut f = fs::File::open(&opts.input)?;

        if opts.input.ends_with("json") {
            let lines = parse_json(serde_json::from_reader(f)?).map_err(Error::NotUnicode)?;
            let index = index(&lines);
            // dbg!(&index);

            let mut ui = Ui::new(lines, index, |v| {
                if let Some(jt) = v.current_line().and_then(|r| r.token_at(v.col())) {
                    if jt.tag() == JsonTokenTag::Ref {
                        let mut q = jt.text().to_string();

                        // remove ""
                        q.pop();
                        q.remove(0);

                        return Some(q);
                    }
                }

                None
            })?;

            ui.run()?;
        } else {
            let mut input = String::new();
            f.read_to_string(&mut input)?;

            let lines = input
                .lines()
                .map(|l| AsciiLine::new(l).map_err(|e| Error::NotUnicode(e.to_string())))
                .collect::<Result<Vec<_>>>();

            let mut ui = Ui::new(lines?, Index::new(), |_| None)?;
            ui.run()?;
        }

        Ok(())
    }

    if let Err(err) = _main() {
        println!("{}", err);
        std::process::exit(1);
    }
}

impl<L, Q> Ui<L, io::Stdout, Q>
where
    L: Line,
    Q: Fn(&View<L>) -> Option<String>,
{
    fn new(lines: Vec<L>, index: Index, get_current_query: Q) -> Result<Self> {
        let stdout = io::stdout().into_raw_mode()?;
        let (width, height) = termion::terminal_size()?;

        Ok(Ui {
            focus: Focus::View,
            status_line: StatusLine::new(height - 2, width),
            view: View::new((width, height - 2), lines),
            get_current_query,
            index,
            stdout,
        })
    }
}

impl<L, W, Q> Ui<L, W, Q>
where
    L: Line,
    W: io::Write,
    Q: Fn(&View<L>) -> Option<String>,
{
    fn clear(&mut self) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            clear::All
        )
    }

    fn run(&mut self) -> Result<()> {
        self.clear()?;

        self.status_line.render(&mut self.stdout)?;
        self.view.render(&mut self.stdout)?;
        self.view.focus(&mut self.stdout)?;

        for ev in io::stdin().keys() {
            match self.focus {
                Focus::View => {
                    let quit = self.update_view(ev?)?;
                    if quit {
                        break;
                    }
                }

                Focus::StatusLine => self.update_status_line(ev?)?,
            }

            self.status_line.render(&mut self.stdout)?;
            self.view.render(&mut self.stdout)?;

            match self.focus {
                Focus::View => self.view.focus(&mut self.stdout)?,
                Focus::StatusLine => self.status_line.focus(&mut self.stdout)?,
            }

            self.status_line.no_error();
        }

        self.clear()?;

        Ok(())
    }

    fn update_view(&mut self, ev: Key) -> Result<bool> {
        match ev {
            Key::Char('q') => return Ok(true),
            Key::Right | Key::Char('l') => self.view.move_right(),
            Key::Left | Key::Char('h') => self.view.move_left(),
            Key::Up | Key::Char('k') => self.view.move_up(),
            Key::Down | Key::Char('j') => self.view.move_down(),
            Key::Char('0') => self.view.move_to_sol(),
            Key::Char('$') => self.view.move_to_eol(),
            Key::PageUp => self.view.page_up(),
            Key::PageDown => self.view.page_down(),
            Key::Char(':') => {
                self.focus = Focus::StatusLine;
                self.status_line.activate(StatusLineMode::Command);
            }
            Key::Char('#') => {
                self.focus = Focus::StatusLine;
                self.status_line.activate(StatusLineMode::Query);
            }
            Key::Char('\n') => {
                if let Some(q) = (self.get_current_query)(&mut self.view) {
                    self.goto_ref(&q)?;
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn update_status_line(&mut self, ev: Key) -> Result<()> {
        match ev {
            Key::Esc => {
                self.status_line.clear();
                self.focus = Focus::View;
            }
            Key::Char('\n') => match self.status_line.mode() {
                StatusLineMode::Command => {
                    if let Some((r, c)) = parse_goto(&self.status_line.text()) {
                        self.view.goto(r, c.unwrap_or(0));

                        self.status_line.clear();
                        self.focus = Focus::View;
                    }
                }
                StatusLineMode::Query => {
                    let q = format!("#{}", self.status_line.text());
                    self.goto_ref(&q)?;
                }
            },
            Key::Char(c) => self.status_line.insert(c),
            Key::Backspace => {
                self.status_line.remove();
                if self.status_line.is_empty() {
                    self.status_line.clear();
                    self.focus = Focus::View;
                }
            }
            Key::Left => self.status_line.left(),
            Key::Right => self.status_line.right(),
            _ => {}
        }

        Ok(())
    }

    fn goto_ref(&mut self, q: &str) -> Result<()> {
        match self.index.get(q.trim_end_matches('/')) {
            Some((r, c)) => {
                self.view.goto(*r, *c);

                self.status_line.clear();
                self.focus = Focus::View;
            }
            None => self
                .status_line
                .set_error(AsciiLine::new(format!("{} not found", q)).map_err(Error::NotUnicode)?),
        }

        Ok(())
    }
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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Json(err) => err.fmt(f),
            Error::NotUnicode(s) => write!(f, "{} is not ascii", s),
        }
    }
}

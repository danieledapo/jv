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

const HELP_TEXT: &str = r##"
          ___      __
         | \ \    / /
         | |\ \  / /
     _   | | \ \/ /
    | |__| |  \  /
     \____/    \/


JV is a simple JSON viewer that supports jq-like queries. Incidentally it works
also as a basic viewer for plain text files.

# Manual

JV has three "modes": COMMAND, QUERY and HELP. You can enter COMMAND mode with
":", QUERY with "#" and help with h from COMMAND mode.

To quit JV hit q while focusing the buffer or :q in COMMAND mode.

JV uses vim-like navigation, you can move around with the arrow keys or with J,
H, K, L. Use 0 and $ to go to the beginning and at the end of the current line
respectively.

Go to a given line and or column by typing in COMMAND mode the line and column
number separated by a ":". It's possible to omit either the row or column
numbers in which case its value won't be changed. Valid examples: "1:20", ":20"
and "1".

Use a jq-like query to quickly jump to an element of a JSON document. First,
enter query mode with "#" and then enter object keys or array indices separated
by "/" . Example queries: "#/", "#/array/23/name", "#/23".

To exit this help page hit q.
"##;

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
    help_view: View<HelpLine>,
    status_line: StatusLine,

    focus: Focus,

    index: Index,
    get_current_query: Q,
}

#[derive(Debug)]
struct HelpLine {
    line: AsciiLine<&'static str>,
    logo: bool,
}

impl Line for HelpLine {
    fn render(&self, start_col: usize, width: usize) -> String {
        if self.logo {
            format!(
                "{}{}{}",
                color::Fg(color::Yellow),
                self.line.render(start_col, width),
                color::Fg(color::Reset)
            )
        } else {
            self.line.render(start_col, width)
        }
    }

    fn indent(&mut self, first_col: usize) {
        self.line.indent(first_col)
    }

    fn chars_count(&self) -> usize {
        self.line.chars_count()
    }

    fn char_width(&self, idx: usize) -> u16 {
        self.line.char_width(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    View,
    StatusLine,
    Help,
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

        match opts.input.extension() {
            Some(e) if e == "json" => {
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
            }
            _ => {
                let mut input = String::new();
                f.read_to_string(&mut input)?;

                let lines = input
                    .lines()
                    .map(|l| AsciiLine::new(l).map_err(|e| Error::NotUnicode(e.to_string())))
                    .collect::<Result<Vec<_>>>();

                let mut ui = Ui::new(lines?, Index::new(), |_| None)?;
                ui.run()?;
            }
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

        let help_view = View::new(
            (width, height),
            HELP_TEXT
                .lines()
                .skip(1)
                .enumerate()
                .map(|(i, l)| HelpLine {
                    line: AsciiLine::new(l).unwrap(),
                    logo: i < 8,
                }),
        );

        Ok(Ui {
            focus: Focus::View,
            status_line: StatusLine::new(height - 2, width),
            view: View::new((width, height - 2), lines),
            get_current_query,
            index,
            stdout,
            help_view,
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
            let quit = match self.focus {
                Focus::View => self.update_view(ev?)?,
                Focus::StatusLine => self.update_status_line(ev?)?,
                Focus::Help => self.update_help_view(ev?)?,
            };

            if quit {
                break;
            }

            if self.focus == Focus::Help {
                self.help_view.render(&mut self.stdout)?;
            } else {
                self.status_line.render(&mut self.stdout)?;
                self.view.render(&mut self.stdout)?;
            }

            match self.focus {
                Focus::View => self.view.focus(&mut self.stdout)?,
                Focus::StatusLine => self.status_line.focus(&mut self.stdout)?,
                Focus::Help => self.help_view.focus(&mut self.stdout)?,
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

    fn update_status_line(&mut self, ev: Key) -> Result<bool> {
        match ev {
            Key::Esc => {
                self.status_line.clear();
                self.focus = Focus::View;
            }
            Key::Up => {
                self.status_line.history_up();
            }
            Key::Down => {
                self.status_line.history_down();
            }
            Key::Char('\n') => match self.status_line.mode() {
                StatusLineMode::Command => {
                    if self.status_line.text() == "q" {
                        return Ok(true);
                    }

                    if self.status_line.text() == "h" {
                        self.status_line.clear();
                        self.focus = Focus::Help;
                        return Ok(false);
                    }

                    match parse_goto(&self.status_line.text()) {
                        None => self.status_line.set_error(
                            AsciiLine::new(format!(
                                "invalid goto line and column ref: {} ",
                                self.status_line.text()
                            ))
                            .map_err(Error::NotUnicode)?,
                        ),

                        Some((r, c)) => {
                            self.view
                                .goto(r.unwrap_or_else(|| self.view.current_row()), c.unwrap_or(0));

                            self.status_line.save_history();
                            self.status_line.clear();
                            self.focus = Focus::View;
                        }
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

        Ok(false)
    }

    fn update_help_view(&mut self, ev: Key) -> Result<bool> {
        match ev {
            Key::Char('q') | Key::Esc => {
                self.focus = Focus::View;
            }
            Key::Right | Key::Char('l') => self.help_view.move_right(),
            Key::Left | Key::Char('h') => self.help_view.move_left(),
            Key::Up | Key::Char('k') => self.help_view.move_up(),
            Key::Down | Key::Char('j') => self.help_view.move_down(),
            Key::Char('0') => self.help_view.move_to_sol(),
            Key::Char('$') => self.help_view.move_to_eol(),
            Key::PageUp => self.help_view.page_up(),
            Key::PageDown => self.help_view.page_down(),
            _ => {}
        }

        Ok(false)
    }

    fn goto_ref(&mut self, q: &str) -> Result<()> {
        match self.index.get(q.trim_end_matches('/')) {
            Some((r, c)) => {
                self.view.goto(*r, *c);

                self.status_line.save_history();
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

fn parse_goto(input: &str) -> Option<(Option<usize>, Option<usize>)> {
    let mut parts = input.split(':').fuse();

    let r = {
        let rs = parts.next()?;
        match rs.parse::<usize>().ok() {
            Some(d) => Some(d.saturating_sub(1)),
            None if rs.is_empty() => None,
            None => return None,
        }
    };

    match parts.next() {
        None => Some((r, None)),
        Some(cs) => match cs.parse::<usize>().ok() {
            Some(c) => {
                if parts.next().is_none() {
                    Some((r, Some(c.saturating_sub(1))))
                } else {
                    None
                }
            }
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

#[cfg(test)]
mod tests {
    use super::parse_goto;

    #[test]
    fn test_parse_goto() {
        assert_eq!(parse_goto("1:100"), Some((Some(0), Some(99))));
        assert_eq!(parse_goto("0:50"), Some((Some(0), Some(49))));

        assert_eq!(parse_goto("42"), Some((Some(41), None)));
        assert_eq!(parse_goto(":42"), Some((None, Some(41))));

        assert_eq!(parse_goto("fuffa:"), None);
        assert_eq!(parse_goto(":yeyo"), None);
        assert_eq!(parse_goto("yoyo"), None);
        assert_eq!(parse_goto("1:yoyo"), None);
        assert_eq!(parse_goto("1:2:"), None);
    }
}

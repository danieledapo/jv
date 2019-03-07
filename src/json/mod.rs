use termion::color;
use termion::style;

use crate::widgets::ascii_line::AsciiLine;
use crate::widgets::view::Line;

pub mod index;
mod parser;

#[derive(Debug)]
pub struct JsonLine {
    tokens: Vec<JsonToken>,
}

#[derive(Debug, Clone)]
pub struct JsonToken {
    tag: JsonTokenTag,
    text: AsciiLine<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonTokenTag {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Null,
    Bool,
    Number,
    String,
    ObjectKey,
    Whitespace,
    Ref,
}

pub fn parse_json(json: serde_json::Value) -> Result<Vec<JsonLine>, String> {
    parser::parse_json_lines(json, 0)
}

impl JsonLine {
    pub fn token_at(&self, idx: usize) -> Option<&JsonToken> {
        let mut col = 0;

        for t in &self.tokens {
            let c = t.chars_count();

            if idx < col + c {
                return Some(t);
            }

            col += c;
        }

        None
    }
}

impl JsonToken {
    pub fn tag(&self) -> JsonTokenTag {
        self.tag
    }

    pub fn text(&self) -> &str {
        self.text.line()
    }
}

impl Line for JsonLine {
    fn chars_count(&self) -> usize {
        self.tokens.iter().map(|l| l.chars_count()).sum::<usize>()
    }

    fn char_width(&self, idx: usize) -> u16 {
        let mut col = 0;

        for t in &self.tokens {
            let c = t.chars_count();

            if idx < col + c {
                return t.char_width(idx - col);
            }

            col += c;
        }

        panic!("bug: shouldn't happen")
    }

    fn indent(&mut self, mut first_col: usize) {
        for t in &mut self.tokens {
            t.indent(first_col);
            first_col += (0..t.chars_count())
                .map(|i| usize::from(t.char_width(i)))
                .sum::<usize>();
        }
    }

    fn render(&self, start_col: usize, width: usize) -> String {
        let mut l = String::new();
        let mut col = 0;

        for t in &self.tokens {
            let c = t.chars_count();

            if start_col < col + c {
                let s = start_col.saturating_sub(col);
                let w = start_col + width - col;

                l.push_str(&t.render(s, w));
            }

            col += c;

            if col >= start_col + width {
                break;
            }
        }

        l
    }
}

impl Line for JsonToken {
    fn chars_count(&self) -> usize {
        self.text.chars_count()
    }

    fn char_width(&self, idx: usize) -> u16 {
        self.text.char_width(idx)
    }

    fn indent(&mut self, width: usize) {
        self.text.indent(width);
    }

    fn render(&self, start_col: usize, width: usize) -> String {
        // termion colors are different types, that's annoying...
        match self.tag {
            JsonTokenTag::Whitespace => format!(
                "{}{}",
                color::Fg(color::Reset),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::ObjectStart => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::ObjectEnd => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::ArrayStart => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::ArrayEnd => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Colon => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Comma => format!(
                "{}{}",
                color::Fg(color::White),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Null => format!(
                "{}{}",
                color::Fg(color::Magenta),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Bool => format!(
                "{}{}",
                color::Fg(color::Magenta),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Number => format!(
                "{}{}",
                color::Fg(color::LightGreen),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::String => format!(
                "{}{}",
                color::Fg(color::Yellow),
                self.text.render(start_col, width)
            ),
            JsonTokenTag::Ref => format!(
                "{}{}{}{}",
                color::Fg(color::Yellow),
                style::Underline,
                self.text.render(start_col, width),
                style::NoUnderline,
            ),
            JsonTokenTag::ObjectKey => format!(
                "{}{}",
                color::Fg(color::Cyan),
                self.text.render(start_col, width)
            ),
        }
    }
}

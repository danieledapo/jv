use std::io;

use termion::color;

use crate::widgets::ascii_line::AsciiLine;
use crate::widgets::view::Line;

#[derive(Debug)]
pub struct JsonLine {
    tokens: Vec<JsonToken>,
}

#[derive(Debug, Clone)]
struct JsonToken {
    tag: JsonTokenTag,
    text: AsciiLine<String>,
}

#[derive(Debug, Clone)]
enum JsonTokenTag {
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
}

pub fn parse_json(rdr: impl io::Read) -> Option<Vec<JsonLine>> {
    let json = serde_json::from_reader(rdr).ok()?;

    parse_json_lines(json, 0)
}

fn parse_json_lines(json: serde_json::Value, indent: usize) -> Option<Vec<JsonLine>> {
    use serde_json::Value;
    use JsonTokenTag::*;

    let mut lines = vec![];

    let new_tok = |tag, t| {
        Some(JsonToken {
            tag,
            text: AsciiLine::new(t)?,
        })
    };

    let new_indent_tok = |s| new_tok(Whitespace, (0..s).map(|_| ' ').collect());

    match json {
        Value::Bool(b) => lines.push(JsonLine {
            tokens: vec![new_tok(Bool, b.to_string())?],
        }),
        Value::Null => lines.push(JsonLine {
            tokens: vec![new_tok(Null, "null".to_string())?],
        }),
        Value::Number(n) => lines.push(JsonLine {
            tokens: vec![new_tok(Number, n.to_string())?],
        }),
        Value::String(mut s) => {
            s.insert(0, '"');
            s.push('"');

            lines.push(JsonLine {
                tokens: vec![new_tok(String, s)?],
            });
        }
        Value::Array(arr) => {
            lines.push(JsonLine {
                tokens: vec![new_tok(ArrayStart, '['.to_string())?],
            });

            let arr_len = arr.len();
            for (i, v) in arr.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                if i < arr_len - 1 {
                    children
                        .last_mut()
                        .unwrap()
                        .tokens
                        .push(new_tok(Comma, ','.to_string())?);
                }

                children[0].tokens.insert(0, new_indent_tok(indent + 4)?);
                lines.extend(children);
            }

            lines.push(JsonLine {
                tokens: vec![new_indent_tok(indent)?, new_tok(ArrayEnd, ']'.to_string())?],
            });
        }
        Value::Object(obj) => {
            lines.push(JsonLine {
                tokens: vec![new_tok(ObjectStart, '{'.to_string())?],
            });

            let obj_len = obj.len();
            for (i, (mut k, v)) in obj.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                children[0].tokens.insert(0, new_indent_tok(1)?);
                children[0]
                    .tokens
                    .insert(0, new_tok(Colon, ":".to_string())?);

                k.insert(0, '"');
                k.push('"');
                children[0].tokens.insert(0, new_tok(ObjectKey, k)?);

                children[0].tokens.insert(0, new_indent_tok(indent + 4)?);

                if i < obj_len - 1 {
                    children
                        .last_mut()
                        .unwrap()
                        .tokens
                        .push(new_tok(Comma, ",".to_string())?);
                }

                lines.extend(children);
            }

            lines.push(JsonLine {
                tokens: vec![
                    new_indent_tok(indent)?,
                    new_tok(ObjectEnd, '}'.to_string())?,
                ],
            });
        }
    };

    Some(lines)
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
        self.text.len()
    }

    fn char_width(&self, idx: usize) -> u16 {
        self.text.char_width(idx)
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
            JsonTokenTag::ObjectKey => format!(
                "{}{}",
                color::Fg(color::Cyan),
                self.text.render(start_col, width)
            ),
        }
    }
}

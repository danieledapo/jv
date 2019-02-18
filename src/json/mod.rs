use std::io;

use termion::color;

use crate::view::ascii_line::AsciiLine;
use crate::view::Line;

#[derive(Debug)]
pub struct JsonLine {
    tokens: Vec<JsonToken>,
    indent: usize,
}

#[derive(Debug)]
struct JsonToken {
    tag: JsonTokenTag,
    text: AsciiLine<String>,
}

#[derive(Debug)]
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

    match json {
        Value::Bool(b) => lines.push(JsonLine {
            indent,
            tokens: vec![new_tok(Bool, b.to_string())?],
        }),
        Value::Null => lines.push(JsonLine {
            indent,
            tokens: vec![new_tok(Null, "null".to_string())?],
        }),
        Value::Number(n) => lines.push(JsonLine {
            indent,
            tokens: vec![new_tok(Number, n.to_string())?],
        }),
        Value::String(mut s) => {
            s.insert(0, '"');
            s.push('"');

            lines.push(JsonLine {
                indent,
                tokens: vec![new_tok(String, s)?],
            });
        }
        Value::Array(arr) => {
            lines.push(JsonLine {
                indent: 0,
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

                lines.extend(children);
            }

            lines.push(JsonLine {
                indent,
                tokens: vec![new_tok(ArrayEnd, ']'.to_string())?],
            });
        }
        Value::Object(obj) => {
            lines.push(JsonLine {
                indent,
                tokens: vec![new_tok(ObjectStart, '{'.to_string())?],
            });

            let obj_len = obj.len();
            for (i, (mut k, v)) in obj.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                children[0]
                    .tokens
                    .insert(0, new_tok(Colon, ": ".to_string())?);

                k.insert(0, '"');
                k.push('"');
                children[0].indent = indent + 4;
                children[0].tokens.insert(0, new_tok(ObjectKey, k)?);

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
                indent,
                tokens: vec![new_tok(ObjectEnd, '}'.to_string())?],
            });
        }
    };

    Some(lines)
}

impl Line for JsonLine {
    fn unstyled_chars_len(&self) -> usize {
        self.indent
            + self
                .tokens
                .iter()
                .map(|l| l.unstyled_chars_len())
                .sum::<usize>()
    }

    fn render(&self, start_col: usize, width: usize) -> String {
        let mut l = String::new();

        let mut col = 0;
        if start_col < self.indent {
            l.extend((0..self.indent - start_col).map(|_| ' '));
            col += self.indent - start_col;
        }

        for t in &self.tokens {
            if col >= start_col + width {
                break;
            }

            l.push_str(&t.render(0, start_col + width - col));
            col += t.unstyled_chars_len();
        }

        l
    }
}

impl Line for JsonToken {
    fn unstyled_chars_len(&self) -> usize {
        self.text.len()
    }

    fn render(&self, start_col: usize, width: usize) -> String {
        // termion colors are different types, that's annoying...
        match self.tag {
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

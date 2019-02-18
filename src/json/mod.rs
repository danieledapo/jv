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
enum JsonToken {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Null,
    Bool(bool),
    Number(f64),
    String(AsciiLine<String>),
    ObjectKey(AsciiLine<String>),
}

pub fn parse_json(rdr: impl io::Read) -> Option<Vec<JsonLine>> {
    let json = serde_json::from_reader(rdr).ok()?;

    parse_json_lines(json, 0)
}

fn parse_json_lines(json: serde_json::Value, indent: usize) -> Option<Vec<JsonLine>> {
    use serde_json::Value;

    let mut lines = vec![];

    match json {
        Value::Bool(b) => lines.push(JsonLine {
            indent,
            tokens: vec![JsonToken::Bool(b)],
        }),
        Value::Null => lines.push(JsonLine {
            indent,
            tokens: vec![JsonToken::Null],
        }),
        Value::Number(n) => lines.push(JsonLine {
            indent,
            tokens: vec![JsonToken::Number(n.as_f64()?)],
        }),
        Value::String(mut s) => {
            s.insert(0, '"');
            s.push('"');

            lines.push(JsonLine {
                indent,
                tokens: vec![JsonToken::String(AsciiLine::new(s)?)],
            });
        }
        Value::Array(arr) => {
            lines.push(JsonLine {
                indent: 0,
                tokens: vec![JsonToken::ArrayStart],
            });

            let arr_len = arr.len();
            for (i, v) in arr.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                if i < arr_len - 1 {
                    children.last_mut().unwrap().tokens.push(JsonToken::Comma);
                }

                lines.extend(children);
            }

            lines.push(JsonLine {
                indent,
                tokens: vec![JsonToken::ArrayEnd],
            });
        }
        Value::Object(obj) => {
            lines.push(JsonLine {
                indent,
                tokens: vec![JsonToken::ObjectStart],
            });

            let obj_len = obj.len();
            for (i, (mut k, v)) in obj.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                children[0].tokens.insert(0, JsonToken::Colon);

                k.insert(0, '"');
                k.push('"');
                children[0].indent = indent + 4;
                children[0]
                    .tokens
                    .insert(0, JsonToken::ObjectKey(AsciiLine::new(k)?));

                if i < obj_len - 1 {
                    children.last_mut().unwrap().tokens.push(JsonToken::Comma);
                }

                lines.extend(children);
            }

            lines.push(JsonLine {
                indent,
                tokens: vec![JsonToken::ObjectEnd],
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
        match self {
            JsonToken::ObjectStart => 1,
            JsonToken::ObjectEnd => 1,
            JsonToken::ArrayStart => 1,
            JsonToken::ArrayEnd => 1,

            // after a colon there's a space
            JsonToken::Colon => 2,

            JsonToken::Comma => 1,
            JsonToken::Null => "null".len(),
            JsonToken::Bool(true) => "true".len(),
            JsonToken::Bool(false) => "false".len(),
            JsonToken::Number(n) => n.to_string().len(),
            JsonToken::String(c) => c.len(),
            JsonToken::ObjectKey(c) => c.len(),
        }
    }

    fn render(&self, start_col: usize, width: usize) -> String {
        match self {
            JsonToken::ObjectStart => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: "{" }.render(start_col, width)
            ),
            JsonToken::ObjectEnd => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: "}" }.render(start_col, width)
            ),
            JsonToken::ArrayStart => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: "[" }.render(start_col, width)
            ),
            JsonToken::ArrayEnd => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: "]" }.render(start_col, width)
            ),
            JsonToken::Colon => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: ": " }.render(start_col, width)
            ),
            JsonToken::Comma => format!(
                "{}{}",
                color::Fg(color::White),
                AsciiLine { l: "," }.render(start_col, width)
            ),
            JsonToken::Null => format!(
                "{}{}",
                color::Fg(color::Magenta),
                AsciiLine { l: "null" }.render(start_col, width)
            ),
            JsonToken::Bool(true) => format!(
                "{}{}",
                color::Fg(color::Magenta),
                AsciiLine { l: "true" }.render(start_col, width)
            ),
            JsonToken::Bool(false) => format!(
                "{}{}",
                color::Fg(color::Magenta),
                AsciiLine { l: "false" }.render(start_col, width)
            ),
            JsonToken::Number(n) => format!(
                "{}{}",
                color::Fg(color::LightGreen),
                AsciiLine { l: &n.to_string() }.render(start_col, width)
            ),
            JsonToken::String(c) => {
                format!("{}{}", color::Fg(color::Yellow), c.render(start_col, width))
            }
            JsonToken::ObjectKey(c) => {
                format!("{}{}", color::Fg(color::Cyan), c.render(start_col, width))
            }
        }
    }
}

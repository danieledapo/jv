use serde_json;

use crate::json::{JsonLine, JsonToken};
use crate::widgets::ascii_line::AsciiLine;

pub fn parse_json_lines(json: serde_json::Value, indent: usize) -> Result<Vec<JsonLine>, String> {
    use crate::json::JsonTokenTag::*;
    use serde_json::Value;

    let mut lines = vec![];

    let new_tok = |tag, t| -> Result<JsonToken, std::string::String> {
        Ok(JsonToken {
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
            let tag = if s.starts_with("#/") { Ref } else { String };

            s.insert(0, '"');
            s.push('"');

            lines.push(JsonLine {
                tokens: vec![new_tok(tag, s)?],
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

    Ok(lines)
}

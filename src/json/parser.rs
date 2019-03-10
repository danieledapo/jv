use serde_json;

use crate::json::{JsonLine, JsonToken};

pub fn parse_json_lines(json: serde_json::Value, indent: usize) -> Result<Vec<JsonLine>, String> {
    use serde_json::Value;

    let mut lines = vec![];

    match json {
        Value::Bool(b) => lines.push(JsonLine {
            tokens: vec![JsonToken::bool(b)],
        }),
        Value::Null => lines.push(JsonLine {
            tokens: vec![JsonToken::null()],
        }),
        Value::Number(n) => lines.push(JsonLine {
            tokens: vec![JsonToken::number(n)],
        }),
        Value::String(s) => {
            lines.push(JsonLine {
                tokens: vec![JsonToken::string(s)?],
            });
        }
        Value::Array(ref arr) if arr.is_empty() => {
            lines.push(JsonLine {
                tokens: vec![JsonToken::array_start(), JsonToken::array_end()],
            });
        }
        Value::Array(arr) => {
            lines.push(JsonLine {
                tokens: vec![JsonToken::array_start()],
            });

            let arr_len = arr.len();
            for (i, v) in arr.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                if i < arr_len - 1 {
                    children.last_mut().unwrap().tokens.push(JsonToken::comma());
                }

                children[0].tokens.insert(0, JsonToken::ws(indent + 4));
                lines.extend(children);
            }

            lines.push(JsonLine {
                tokens: vec![JsonToken::ws(indent), JsonToken::array_end()],
            });
        }
        Value::Object(ref obj) if obj.is_empty() => {
            lines.push(JsonLine {
                tokens: vec![JsonToken::object_start(), JsonToken::object_end()],
            });
        }
        Value::Object(obj) => {
            lines.push(JsonLine {
                tokens: vec![JsonToken::object_start()],
            });

            let obj_len = obj.len();
            for (i, (mut k, v)) in obj.into_iter().enumerate() {
                let mut children = parse_json_lines(v, indent + 4)?;

                children[0].tokens.insert(0, JsonToken::ws(1));
                children[0].tokens.insert(0, JsonToken::colon());

                children[0].tokens.insert(0, JsonToken::object_key(k)?);

                children[0].tokens.insert(0, JsonToken::ws(indent + 4));

                if i < obj_len - 1 {
                    children.last_mut().unwrap().tokens.push(JsonToken::comma());
                }

                lines.extend(children);
            }

            lines.push(JsonLine {
                tokens: vec![JsonToken::ws(indent), JsonToken::object_end()],
            });
        }
    };

    Ok(lines)
}

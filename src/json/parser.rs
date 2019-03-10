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

            // this is potentially inefficient for large objects but it's pretty
            // useful
            let mut items = obj.into_iter().collect::<Vec<_>>();
            items.sort_by(|o1, o2| (o1.0).cmp(&o2.0));

            for (i, (k, v)) in items.into_iter().enumerate() {
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

#[cfg(test)]
mod tests {
    use crate::json::{JsonLine, JsonToken};

    #[test]
    fn test_parse_json_primitive() {
        let input_json = r##"{
    "empty-array": [],
    "empty-object": {},
    "name": "mattors",
    "ref1": "#/ciaomondo/23",
    "private": true
}
"##;

        let expected = vec![
            JsonLine::new(vec![JsonToken::object_start()]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("empty-array".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::array_start(),
                JsonToken::array_end(),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("empty-object".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::object_start(),
                JsonToken::object_end(),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("name".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::string("mattors".to_string()).unwrap(),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("private".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::bool(true),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("ref1".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::string("#/ciaomondo/23".to_string()).unwrap(),
            ]),
            JsonLine::new(vec![JsonToken::object_end()]),
        ];

        let value = serde_json::from_str(input_json).unwrap();
        let lines = super::parse_json_lines(value, 0).unwrap();

        assert_eq!(lines.len(), expected.len());
        for (i, (g, e)) in lines.into_iter().zip(expected.into_iter()).enumerate() {
            assert_eq!(g, e, "line #{} differ", i);
        }
    }

    #[test]
    fn test_parse_simple_array() {
        let input_json = "[1,null,true]";

        let expected = vec![
            JsonLine::new(vec![JsonToken::array_start()]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::number(1.into()),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::null(),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![JsonToken::ws(4), JsonToken::bool(true)]),
            JsonLine::new(vec![JsonToken::array_end()]),
        ];

        let value = serde_json::from_str(input_json).unwrap();
        let lines = super::parse_json_lines(value, 0).unwrap();

        assert_eq!(lines.len(), expected.len());
        for (i, (g, e)) in lines.into_iter().zip(expected.into_iter()).enumerate() {
            assert_eq!(g, e, "line #{} differ", i);
        }
    }

    #[test]
    fn test_parse_nested_json() {
        let input_json = r##"{"a" : [1,2,3, {"hello-world": null}]}"##;

        let expected = vec![
            JsonLine::new(vec![JsonToken::object_start()]),
            JsonLine::new(vec![
                JsonToken::ws(4),
                JsonToken::object_key("a".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::array_start(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(8),
                JsonToken::number(1.into()),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(8),
                JsonToken::number(2.into()),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![
                JsonToken::ws(8),
                JsonToken::number(3.into()),
                JsonToken::comma(),
            ]),
            JsonLine::new(vec![JsonToken::ws(8), JsonToken::object_start()]),
            JsonLine::new(vec![
                JsonToken::ws(12),
                JsonToken::object_key("hello-world".to_string()).unwrap(),
                JsonToken::colon(),
                JsonToken::ws(1),
                JsonToken::null(),
            ]),
            JsonLine::new(vec![JsonToken::ws(8), JsonToken::object_end()]),
            JsonLine::new(vec![JsonToken::ws(4), JsonToken::array_end()]),
            JsonLine::new(vec![JsonToken::object_end()]),
        ];

        let value = serde_json::from_str(input_json).unwrap();
        let lines = super::parse_json_lines(value, 0).unwrap();

        assert_eq!(lines.len(), expected.len());
        for (i, (g, e)) in lines.into_iter().zip(expected.into_iter()).enumerate() {
            assert_eq!(g, e, "line #{} differ", i);
        }
    }
}

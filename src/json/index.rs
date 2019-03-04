use std::collections::HashMap;

use crate::json::{JsonLine, JsonTokenTag};
use crate::widgets::view::Line;

/// Index type from reference to (row, col)
pub type Index = HashMap<String, (usize, usize)>;

/// Create an index over the given json lines.
pub fn index(lines: &[JsonLine]) -> Index {
    let mut refs = HashMap::new();
    let mut path = vec!["#".to_string()];

    // stack of the current array index if inside an array and whether the
    // current collection has at least one entry.
    let mut stack: Vec<(Option<usize>, bool)> = vec![];

    for (r, line) in lines.iter().enumerate() {
        let mut c = 0;

        for tok in &line.tokens {
            match tok.tag {
                JsonTokenTag::ObjectStart | JsonTokenTag::ArrayStart => {
                    if let Some((Some(ix), _)) = stack.last() {
                        path.push(ix.to_string());
                    }

                    refs.insert(path.join(""), (r, c));
                    path.push("/".to_string());

                    if tok.tag == JsonTokenTag::ArrayStart {
                        stack.push((Some(0), false));
                    } else {
                        stack.push((None, false));
                    }
                }
                JsonTokenTag::ArrayEnd | JsonTokenTag::ObjectEnd => {
                    let has_entry = stack.pop().unwrap().1;
                    if has_entry {
                        path.pop();
                    }
                    path.pop();
                }
                JsonTokenTag::Comma => {
                    let (array_ix, has_entry) = stack.last_mut().unwrap();
                    *has_entry = true;

                    if let Some(array_ix) = array_ix {
                        *array_ix += 1;
                    }

                    path.pop();
                }
                JsonTokenTag::ObjectKey => {
                    let mut k = tok.text.line().to_string();

                    // remove ""
                    k.remove(0);
                    k.pop();

                    path.push(k);
                }
                JsonTokenTag::Null
                | JsonTokenTag::Number
                | JsonTokenTag::Bool
                | JsonTokenTag::String
                | JsonTokenTag::Ref => {
                    let (array_ix, has_entry) = stack.last_mut().unwrap();
                    *has_entry = true;

                    if let Some(ix) = array_ix {
                        path.push(ix.to_string());
                    }

                    refs.insert(path.join(""), (r, c));
                }
                _ => {}
            }

            c += tok.chars_count();
        }
    }

    refs
}

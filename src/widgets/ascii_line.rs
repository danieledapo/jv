use std::collections::HashMap;

use crate::widgets::view::Line;

pub const TAB_WIDTH: usize = 8;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<S> {
    pub(crate) l: S,
    tabs: HashMap<usize, u8>,
    size: usize,
}

impl<'a, S> AsciiLine<S>
where
    S: AsRef<str>,
{
    pub fn new(l: S) -> Option<Self> {
        // FIXME: start_col should probably be passed, since the line doesn't always
        // start from col 0 and therefore tab handling is different...
        let start_col = 4;

        let mut p = 0;
        let mut tabs = HashMap::new();

        for c in l.as_ref().chars() {
            if !c.is_ascii() {
                return None;
            }

            if c == '\t' {
                let s = TAB_WIDTH - (p + start_col) % TAB_WIDTH;
                tabs.insert(p, s as u8);

                p += s;
            } else {
                p += 1;
            }
        }

        dbg!(&tabs);

        Some(AsciiLine { l, tabs, size: p })
    }

    pub fn line(&self) -> &S {
        &self.l
    }

    pub fn is_empty(&self) -> bool {
        self.l.as_ref().is_empty()
    }
}

impl<S> Line for AsciiLine<S>
where
    S: AsRef<str>,
{
    fn render(&self, start_col: usize, width: usize) -> String {
        let l = self.l.as_ref();
        if start_col < l.len() {
            l[start_col..l.len().min(start_col + width)].to_string()
        } else {
            String::new()
        }
    }

    fn unstyled_chars_len(&self) -> usize {
        self.size
    }

    fn char_width(&self, idx: usize) -> u16 {
        u16::from(*self.tabs.get(&idx).unwrap_or(&1))
    }

    fn len(&self) -> usize {
        self.l.as_ref().len()
    }
}

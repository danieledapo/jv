use std::collections::BTreeMap;

use crate::widgets::view::Line;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<S> {
    l: S,
    char_widths: BTreeMap<usize, u8>,
}

impl<S> AsciiLine<S>
where
    S: AsRef<str>,
{
    pub fn new(l: S) -> Option<Self> {
        if l.as_ref().is_ascii() {
            Some(AsciiLine {
                l,
                char_widths: BTreeMap::new(),
            })
        } else {
            None
        }
    }

    pub fn line(&self) -> &S {
        &self.l
    }
}

impl AsciiLine<String> {
    pub fn clear(&mut self) {
        self.l.clear();
        self.char_widths.clear();
    }

    pub fn insert(&mut self, ix: usize, c: char) {
        self.l.insert(ix, c);
    }

    pub fn remove(&mut self, ix: usize) {
        self.l.remove(ix);
    }
}

impl<S> Line for AsciiLine<S>
where
    S: AsRef<str>,
{
    fn render(&self, start_col: usize, width: usize) -> String {
        let mut w = 0;
        let mut rendered = String::new();

        for (i, c) in self.l.as_ref().chars().enumerate().skip(start_col) {
            w += usize::from(self.char_width(i));

            if w > width {
                break;
            }

            rendered.push(c);
        }

        rendered
    }

    fn chars_count(&self) -> usize {
        self.l.as_ref().len()
    }

    fn char_width(&self, idx: usize) -> u16 {
        u16::from(*self.char_widths.get(&idx).unwrap_or(&1))
    }
}

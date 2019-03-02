use std::collections::BTreeMap;

use crate::widgets::view::Line;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<S> {
    l: S,
    char_widths: BTreeMap<usize, u8>,
    first_col: usize,
}

impl<S> AsciiLine<S>
where
    S: AsRef<str> + std::fmt::Debug,
{
    pub fn new(l: S) -> Option<Self> {
        if l.as_ref().is_ascii() {
            let mut line = AsciiLine {
                l,
                char_widths: BTreeMap::new(),
                first_col: 0,
            };

            line.indent(0);

            Some(line)
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

        // there's no need to recalculate tab widths if we're not inserting any
        // tabs or we haven't any already because we're certain that nothing
        // would change.
        if c == '\t' || !self.char_widths.is_empty() {
            self.indent(self.first_col);
        }
    }

    pub fn remove(&mut self, ix: usize) {
        self.l.remove(ix);

        // if the line doesn't have any characters with width >1 then there's no
        // need to recalculate anything.
        if !self.char_widths.is_empty() {
            self.indent(self.first_col);
        }
    }
}

impl<S> Line for AsciiLine<S>
where
    S: AsRef<str> + std::fmt::Debug,
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

    fn indent(&mut self, first_col: usize) {
        // This is inefficient, but hopefully this hasn't to called too often
        // because characters with widths >1 are not that common...

        self.char_widths.clear();
        self.first_col = first_col;

        let mut col = first_col;
        for (i, c) in self.l.as_ref().chars().enumerate() {
            if c == '\t' {
                let tw = 8 - (col % 8) as u8;

                self.char_widths.insert(i, tw);
                col += usize::from(tw);
            } else {
                col += 1;
            }
        }
    }

    fn chars_count(&self) -> usize {
        self.l.as_ref().len()
    }

    fn char_width(&self, idx: usize) -> u16 {
        u16::from(*self.char_widths.get(&idx).unwrap_or(&1))
    }
}

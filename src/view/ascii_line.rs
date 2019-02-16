use crate::view::Line;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<'a> {
    pub(crate) l: &'a str,
}

impl<'a> AsciiLine<'a> {
    pub fn new(l: &'a str) -> Option<Self> {
        if l.is_ascii() {
            Some(AsciiLine { l })
        } else {
            None
        }
    }

    pub fn line(&self) -> &'a str {
        self.l
    }

    pub fn len(&self) -> usize {
        self.l.len()
    }

    pub fn is_empty(&self) -> bool {
        self.l.is_empty()
    }
}

impl Line for AsciiLine<'_> {
    fn render(&self, start_col: usize, width: usize) -> String {
        if start_col < self.l.len() {
            let row = &self.l[start_col..self.l.len().min(start_col + width)];

            row.to_string()
        } else {
            String::new()
        }
    }

    fn unstyled_chars_len(&self) -> usize {
        self.l.len()
    }
}

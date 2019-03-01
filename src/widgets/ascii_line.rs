use crate::widgets::view::Line;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<S> {
    l: S,
}

impl<S> AsciiLine<S>
where
    S: AsRef<str>,
{
    pub fn new(l: S) -> Option<Self> {
        if l.as_ref().is_ascii() {
            Some(AsciiLine { l })
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
        // TODO: chop string at width handling variable length characters like
        // tabs

        if start_col < self.chars_count() {
            let row = &self.l.as_ref()[start_col..self.chars_count().min(start_col + width)];

            row.to_string()
        } else {
            String::new()
        }
    }

    fn chars_count(&self) -> usize {
        self.l.as_ref().len()
    }

    fn char_width(&self, idx: usize) -> u16 {
        // TODO: check if char at idx has a custom width and return it

        1
    }
}

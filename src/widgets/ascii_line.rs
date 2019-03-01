use crate::widgets::view::Line;

/// Simple ascii line that can be used to create a simple viewer over ascii
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiLine<S> {
    pub(crate) l: S,
}

impl<'a, S> AsciiLine<S>
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

    pub fn len(&self) -> usize {
        self.l.as_ref().len()
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
        // TODO: chop string at width handling variable length characters like
        // tabs

        if start_col < self.len() {
            let row = &self.l.as_ref()[start_col..self.len().min(start_col + width)];

            row.to_string()
        } else {
            String::new()
        }
    }

    fn chars_count(&self) -> usize {
        self.len()
    }

    fn char_width(&self, idx: usize) -> u16 {
        // TODO: check if char at idx has a custom width and return it

        1
    }
}

impl<S> AsRef<str> for AsciiLine<S>
where
    S: AsRef<str>,
{
    fn as_ref(&self) -> &str {
        self.l.as_ref()
    }
}

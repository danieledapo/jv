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
    /// Create a new AsciiLine from the given string. Returns the raw line on
    /// error if it contains non ascii characters.
    pub fn new(l: S) -> Result<Self, S> {
        if l.as_ref().is_ascii() {
            let mut line = AsciiLine {
                l,
                char_widths: BTreeMap::new(),
                first_col: 0,
            };

            line.indent(0);

            Ok(line)
        } else {
            Err(l)
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
        // This is inefficient, but hopefully this hasn't to be called too often
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

#[cfg(test)]
mod tests {
    use super::AsciiLine;
    use crate::widgets::view::Line;

    #[test]
    fn test_returns_error_for_non_ascii_string() {
        assert_eq!(AsciiLine::new("la vita è bella"), Err("la vita è bella"));
        assert_eq!(
            AsciiLine::new("❤️ pugs ❤️"),
            Err("❤️ pugs ❤️")
        );
    }

    #[test]
    fn test_renders_correctly() {
        let line = AsciiLine::new("42 is the answer to life, the universe and everything").unwrap();

        assert_eq!(line.chars_count(), 53);

        for c in 0..line.chars_count() {
            assert_eq!(
                line.char_width(c),
                1,
                "character at #{} is not 1 unit wide",
                c
            );
        }

        assert_eq!(&line.render(0, 53), line.l);

        assert_eq!(&line.render(0, 80), line.l);
        assert_eq!(&line.render(0, 10), "42 is the ");

        assert_eq!(&line.render(100, 80), "");

        assert_eq!(
            &line.render(10, 80),
            "answer to life, the universe and everything"
        );
        assert_eq!(&line.render(10, 14), "answer to life");
    }

    #[test]
    fn test_renders_tabs_correctly() {
        let mut line = AsciiLine::new("\tA\tBB\tCCC\tDDDD\tEEEEE\tFFFFFF\tGGGGGGG\tH").unwrap();

        assert_eq!(line.chars_count(), 37);

        for c in 0..line.chars_count() {
            let w = match c {
                0 => 8,
                2 => 7,
                5 => 6,
                9 => 5,
                14 => 4,
                20 => 3,
                27 => 2,
                35 => 1,
                _ => 1,
            };

            assert_eq!(
                line.char_width(c),
                w,
                "character at #{} is not {} unit wide",
                c,
                w
            );
        }

        assert_eq!(&line.render(0, 66), line.l);

        assert_eq!(&line.render(0, 80), line.l);
        assert_eq!(&line.render(0, 10), "\tA");

        assert_eq!(&line.render(100, 80), "");

        assert_eq!(&line.render(1, 10), "A\tBB");
        assert_eq!(&line.render(5, 7), "\tC");

        assert_eq!(&line.render(2, 3), "");

        line.indent(3);
        for c in 0..line.chars_count() {
            let w = match c {
                0 => 5,
                2 => 7,
                5 => 6,
                9 => 5,
                14 => 4,
                20 => 3,
                27 => 2,
                35 => 1,
                _ => 1,
            };

            assert_eq!(
                line.char_width(c),
                w,
                "character at #{} is not {} unit wide",
                c,
                w
            );
        }
    }

    #[test]
    fn test_insert() {
        let mut line = AsciiLine::new("".to_string()).unwrap();

        line.insert(0, 'h');
        line.insert(1, 'i');
        line.insert(2, 'g');
        line.insert(3, 'r');
        line.insert(4, 'u');
        line.insert(5, 'n');
        line.insert(6, 'd');

        assert_eq!(line.render(0, 80), "higrund");

        line.insert(2, ',');
        line.insert(3, ' ');
        line.insert(6, 'o');
        line.insert(10, '!');

        assert_eq!(line.render(0, 80), "hi, ground!");
    }

    #[test]
    fn test_remove() {
        let mut line = AsciiLine::new("hi, ground!".to_string()).unwrap();

        line.remove(2);
        line.remove(2);
        line.remove(4);
        line.remove(7);

        assert_eq!(line.render(0, 80), "higrund");
    }

    #[test]
    fn test_edit() {
        let mut line = AsciiLine::new("".to_string()).unwrap();

        line.insert(0, '/');
        line.remove(0);

        assert_eq!(line.render(0, 80), "");

        line.insert(0, 'm');
        line.insert(0, 'a');
        line.insert(0, 'y');

        assert_eq!(line.render(0, 80), "yam");

        line.remove(1);
        line.insert(1, 'u');

        assert_eq!(line.render(0, 80), "yum");

        line.remove(2);
        line.remove(1);
        line.remove(0);

        assert_eq!(line.render(0, 80), "");
    }

    #[test]
    fn test_edit_tabs() {
        let mut line = AsciiLine::new("".to_string()).unwrap();

        line.insert(0, '0');
        line.insert(1, '\t');
        line.insert(2, '$');

        assert_eq!(line.char_width(1), 7);

        line.insert(1, '1');
        assert_eq!(line.char_width(2), 6);

        line.insert(2, '2');
        assert_eq!(line.char_width(3), 5);

        line.remove(1);
        assert_eq!(line.char_width(2), 6);

        assert_eq!(line.render(0, 80), "02\t$");
        assert_eq!(line.render(0, 3), "02");
    }
}

use std::iter::Peekable;
use std::str::Chars;

pub trait EqualIgnoreWhitespace {
    fn eq_ignore_whitespace(&self, other: &str) -> bool;
}

impl EqualIgnoreWhitespace for &str {
    fn eq_ignore_whitespace(&self, other: &str) -> bool {
        let mut self_iter = self.chars().into_iter().peekable();
        let mut other_iter = other.chars().into_iter().peekable();

        let mut self_char: Option<char> = None;
        let mut other_char: Option<char> = None;

        fn skip_whitespace(iter: &mut Peekable<Chars>) {
            while iter.peek().map(|c| c.is_whitespace()).unwrap_or_default() {
                iter.next();
            }
        }

        let same = loop {
            if self_char.map(|c| c.is_whitespace()).unwrap_or(true) {
                skip_whitespace(&mut self_iter);
            }

            if other_char.map(|c| c.is_whitespace()).unwrap_or(true) {
                skip_whitespace(&mut other_iter);
            }

            self_char = self_iter.next();
            other_char = other_iter.next();

            if self_char.is_none() && other_char.is_none() {
                break true;
            }

            if self_char.map(|c| c.is_whitespace()).unwrap_or(true) &&
                other_char.map(|c| c.is_whitespace()).unwrap_or(true)
            {
                continue
            }

            if self_char != other_char {
                break false;
            }
        };

        same
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_ignore_whitespace() {
        let left = "this is a test";
        let right = "  this\tis \t a      test  \t";

        let actual = left.eq_ignore_whitespace(right);

        assert_eq!(true, actual);
    }
}
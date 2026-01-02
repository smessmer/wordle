use crate::constants::WORD_LENGTH;
use std::fmt;

/// A single letter in a word (always lowercase internally)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Letter(char);

impl Letter {
    /// Create from a character, normalizing to lowercase.
    /// Returns None if the character is not alphabetic.
    pub fn new(c: char) -> Option<Self> {
        if c.is_alphabetic() {
            Some(Self(c.to_lowercase().next().unwrap_or(c)))
        } else {
            None
        }
    }

    /// Get the character
    pub fn char(&self) -> char {
        self.0
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A word of WORD_LENGTH letters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Word([Letter; WORD_LENGTH]);

impl Word {
    /// Parse from string, returns None if not exactly WORD_LENGTH alphabetic chars
    pub fn parse(s: &str) -> Option<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != WORD_LENGTH {
            return None;
        }

        let mut letters = [Letter('a'); WORD_LENGTH];
        for (i, c) in chars.into_iter().enumerate() {
            letters[i] = Letter::new(c)?;
        }
        Some(Self(letters))
    }

    /// Get letter at position (0..WORD_LENGTH)
    pub fn letter(&self, index: usize) -> Letter {
        self.0[index]
    }

    /// Iterate over letters
    pub fn letters(&self) -> impl Iterator<Item = Letter> + '_ {
        self.0.iter().copied()
    }

    /// Convert to lowercase string
    pub fn as_str(&self) -> String {
        self.0.iter().map(|l| l.char()).collect()
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_new() {
        assert_eq!(Letter::new('a').map(|l| l.char()), Some('a'));
        assert_eq!(Letter::new('A').map(|l| l.char()), Some('a'));
        assert_eq!(Letter::new('1'), None);
        assert_eq!(Letter::new(' '), None);
    }

    #[test]
    fn test_word_parse() {
        let word = Word::parse("hello").unwrap();
        assert_eq!(word.as_str(), "hello");

        let word = Word::parse("HELLO").unwrap();
        assert_eq!(word.as_str(), "hello");

        assert!(Word::parse("hi").is_none());
        assert!(Word::parse("toolong").is_none());
        assert!(Word::parse("hell0").is_none());
    }

    #[test]
    fn test_word_letters() {
        let word = Word::parse("hello").unwrap();
        let letters: Vec<char> = word.letters().map(|l| l.char()).collect();
        assert_eq!(letters, vec!['h', 'e', 'l', 'l', 'o']);
    }
}

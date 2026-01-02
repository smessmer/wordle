//! A string newtype with case-fold ordering.

use std::cmp::Ordering;
use std::fmt;

use super::ordering::case_fold_cmp;

/// A word with case-fold ordering.
///
/// This is a newtype around `String` that implements `Ord` using case-fold
/// comparison, where lowercase letters come before uppercase:
/// `"apple" < "Apple" < "APPLE" < "banana"`.
/// 
/// This ordering is important because otherwise [WordStream::to_lowercase]
/// could break the sorted invariant of a WordStream.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Word(pub String);

impl Ord for Word {
    fn cmp(&self, other: &Self) -> Ordering {
        case_fold_cmp(&self.0, &other.0)
    }
}

impl PartialOrd for Word {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<String> for Word {
    fn from(s: String) -> Self {
        Word(s)
    }
}

impl From<Word> for String {
    fn from(w: Word) -> Self {
        w.0
    }
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ord_case_fold() {
        let apple = Word("apple".to_string());
        let apple_cap = Word("Apple".to_string());
        let apple_upper = Word("APPLE".to_string());
        let banana = Word("banana".to_string());

        assert!(apple < apple_cap);
        assert!(apple_cap < apple_upper);
        assert!(apple_upper < banana);
    }

    #[test]
    fn test_from_string() {
        let w: Word = "hello".to_string().into();
        assert_eq!(w.0, "hello");
    }

    #[test]
    fn test_into_string() {
        let w = Word("hello".to_string());
        let s: String = w.into();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_as_ref() {
        let w = Word("hello".to_string());
        let s: &str = w.as_ref();
        assert_eq!(s, "hello");
    }
}

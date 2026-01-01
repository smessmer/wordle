//! Merge transform for combining two sorted WordStreams.

use std::cmp::Ordering;
use std::io;
use std::iter::Peekable;

use crate::wordlist::stream::ordering::case_fold_cmp;

/// An iterator that merges two sorted streams into one sorted stream.
///
/// Both input streams must be sorted in case-fold order. The output maintains
/// this ordering by comparing the heads of both streams and emitting the smaller one.
pub struct MergeStream<I1: Iterator, I2: Iterator> {
    left: Peekable<I1>,
    right: Peekable<I2>,
}

impl<I1, I2> MergeStream<I1, I2>
where
    I1: Iterator,
    I2: Iterator,
{
    pub fn new(left: Peekable<I1>, right: Peekable<I2>) -> Self {
        Self { left, right }
    }
}

impl<I1, I2> Iterator for MergeStream<I1, I2>
where
    I1: Iterator<Item = io::Result<String>>,
    I2: Iterator<Item = io::Result<String>>,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.peek(), self.right.peek()) {
            (None, None) => None,
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (Some(Ok(l)), Some(Ok(r))) => {
                if case_fold_cmp(l, r) != Ordering::Greater {
                    self.left.next()
                } else {
                    self.right.next()
                }
            }
            // Errors: emit left errors first
            (Some(Err(_)), _) => self.left.next(),
            (_, Some(Err(_))) => self.right.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(
        items: I,
    ) -> impl Iterator<Item = io::Result<String>> {
        items.into_iter().map(|s| Ok(s.to_string()))
    }

    #[test]
    fn test_merge_disjoint() {
        let left = ok_iter(["apple", "banana"]).peekable();
        let right = ok_iter(["cherry", "date"]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry", "date"]);
    }

    #[test]
    fn test_merge_interleaved() {
        let left = ok_iter(["apple", "cherry"]).peekable();
        let right = ok_iter(["banana", "date"]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry", "date"]);
    }

    #[test]
    fn test_merge_with_duplicates() {
        let left = ok_iter(["apple", "banana"]).peekable();
        let right = ok_iter(["apple", "cherry"]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        // Both "apple"s are emitted (left first due to <=)
        assert_eq!(collected, vec!["apple", "apple", "banana", "cherry"]);
    }

    #[test]
    fn test_merge_case_fold_order() {
        // "apple" < "Apple" < "APPLE" in case-fold order
        let left = ok_iter(["apple", "APPLE"]).peekable();
        let right = ok_iter(["Apple", "banana"]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "Apple", "APPLE", "banana"]);
    }

    #[test]
    fn test_merge_left_empty() {
        let left = ok_iter([]).peekable();
        let right = ok_iter(["apple", "banana"]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "banana"]);
    }

    #[test]
    fn test_merge_right_empty() {
        let left = ok_iter(["apple", "banana"]).peekable();
        let right = ok_iter([]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "banana"]);
    }

    #[test]
    fn test_merge_both_empty() {
        let left = ok_iter([]).peekable();
        let right = ok_iter([]).peekable();
        let merged = MergeStream::new(left, right);
        let collected: Vec<String> = merged.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_merge_preserves_errors() {
        let left: Vec<io::Result<String>> = vec![
            Ok("apple".to_string()),
            Err(io::Error::new(io::ErrorKind::Other, "left error")),
            Ok("cherry".to_string()),
        ];
        let right: Vec<io::Result<String>> = vec![
            Ok("banana".to_string()),
            Ok("date".to_string()),
        ];
        let merged = MergeStream::new(left.into_iter().peekable(), right.into_iter().peekable());
        let results: Vec<_> = merged.collect();

        // Error is emitted immediately when encountered (after apple)
        assert_eq!(results.len(), 5);
        assert_eq!(results[0].as_ref().unwrap(), "apple");
        assert!(results[1].is_err()); // left error emitted immediately
        assert_eq!(results[2].as_ref().unwrap(), "banana");
        assert_eq!(results[3].as_ref().unwrap(), "cherry");
        assert_eq!(results[4].as_ref().unwrap(), "date");
    }
}

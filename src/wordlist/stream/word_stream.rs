//! Core WordStream type for sorted word processing.

use std::cmp::Ordering;
use std::io;
use std::iter::Peekable;

use super::ordering::case_fold_cmp;

/// A stream of words, guaranteed to be sorted in case-fold order.
///
/// Panics during iteration if the underlying data is not sorted.
/// This ensures that any `WordStream` can be safely used for operations
/// that require sorted input (like deduplication or writing to sorted files).
///
/// Uses `Peekable` internally to validate sortedness by comparing current
/// with next item, eliminating the need to store the previous item.
pub struct WordStream<I: Iterator> {
    inner: Peekable<I>,
}

impl<I: Iterator> WordStream<I> {
    /// Creates a new WordStream wrapping the given iterator.
    ///
    /// The stream will validate sortedness during iteration and panic
    /// if items are not in case-fold order.
    pub(crate) fn new(inner: I) -> Self {
        Self {
            inner: inner.peekable(),
        }
    }

    /// Consumes the stream and returns the underlying peekable iterator.
    pub fn into_inner(self) -> Peekable<I> {
        self.inner
    }
}

impl<I> Iterator for WordStream<I>
where
    I: Iterator<Item = io::Result<String>>,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;

        match item {
            Ok(s) => {
                // Validate sortedness by peeking at the next item
                if let Some(Ok(next)) = self.inner.peek()
                    && case_fold_cmp(&s, next) == Ordering::Greater
                {
                    panic!(
                        "WordStream is not sorted: {:?} came before {:?}",
                        s, next
                    );
                }
                Some(Ok(s))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(items: I) -> impl Iterator<Item = io::Result<String>> {
        items.into_iter().map(|s| Ok(s.to_string()))
    }

    #[test]
    fn test_sorted_stream_iterates() {
        let stream = WordStream::new(ok_iter(["apple", "banana", "cherry"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_case_fold_sorted_stream() {
        // "apple" < "Apple" < "banana" in case-fold order
        let stream = WordStream::new(ok_iter(["apple", "Apple", "banana"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "Apple", "banana"]);
    }

    #[test]
    #[should_panic(expected = "not sorted")]
    fn test_unsorted_stream_panics() {
        let stream = WordStream::new(ok_iter(["banana", "apple"]));
        let _: Vec<_> = stream.collect();
    }

    #[test]
    #[should_panic(expected = "not sorted")]
    fn test_case_unsorted_stream_panics() {
        // "Apple" should come after "apple", not before
        let stream = WordStream::new(ok_iter(["Apple", "apple"]));
        let _: Vec<_> = stream.collect();
    }

    #[test]
    fn test_empty_stream() {
        let stream: WordStream<_> = WordStream::new(ok_iter([]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_single_item_stream() {
        let stream = WordStream::new(ok_iter(["hello"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["hello"]);
    }

    #[test]
    fn test_io_error_propagates() {
        let items: Vec<io::Result<String>> = vec![
            Ok("apple".to_string()),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok("banana".to_string()),
        ];
        let stream = WordStream::new(items.into_iter());
        let results: Vec<_> = stream.collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        // After error, stream continues
        assert!(results[2].is_ok());
    }
}

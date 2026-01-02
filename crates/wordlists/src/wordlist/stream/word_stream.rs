//! Core WordStream type for sorted word processing.

use std::cmp::Ordering;
use std::io;
use std::iter::Peekable;

use crate::wordlist::Word;

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

impl<I> WordStream<I>
where
    I: Iterator<Item = io::Result<Word>> + 'static,
{
    /// Converts to a type-erased `BoxedWordStream` for dynamic composition.
    ///
    /// This allows merging an arbitrary number of streams in a loop,
    /// at the cost of dynamic dispatch overhead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_zst_file;
    ///
    /// let inputs = ["a.zst", "b.zst"];
    /// let mut stream = from_sorted_zst_file(inputs[0])?.boxed();
    /// for input in &inputs[1..] {
    ///     stream = stream.merge(from_sorted_zst_file(input)?.boxed());
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn boxed(self) -> super::boxed::BoxedWordStream {
        super::boxed::BoxedWordStream::new(self.inner)
    }
}

impl<I> Iterator for WordStream<I>
where
    I: Iterator<Item = io::Result<Word>>,
{
    type Item = io::Result<Word>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;

        match item {
            Ok(w) => {
                // Validate sortedness by peeking at the next item
                if let Some(Ok(next)) = self.inner.peek()
                    && w.cmp(next) == Ordering::Greater
                {
                    panic!(
                        "WordStream is not sorted: {:?} came before {:?}",
                        w, next
                    );
                }
                Some(Ok(w))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(
        items: I,
    ) -> impl Iterator<Item = io::Result<Word>> {
        items
            .into_iter()
            .map(|s| Ok(Word(s.to_string())))
    }

    #[test]
    fn test_sorted_stream_iterates() {
        let stream = WordStream::new(ok_iter(["apple", "banana", "cherry"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_case_fold_sorted_stream() {
        // "apple" < "Apple" < "banana" in case-fold order
        let stream = WordStream::new(ok_iter(["apple", "Apple", "banana"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
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
        let collected: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_single_item_stream() {
        let stream = WordStream::new(ok_iter(["hello"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["hello"]);
    }

    #[test]
    fn test_io_error_propagates() {
        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok(Word("banana".to_string())),
        ];
        let stream = WordStream::new(items.into_iter());
        let results: Vec<_> = stream.collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        // After error, stream continues
        assert!(results[2].is_ok());
    }
}

//! Type-erased word stream for dynamic composition.

use std::io;
use std::path::Path;

use crate::wordlist::Word;

use super::sinks;
use super::transforms::{DedupStream, FilterStream, LowercaseStream, MergeStream};

/// A type-erased word stream for dynamic composition.
///
/// Unlike `WordStream<I>`, `BoxedWordStream` uses dynamic dispatch to allow
/// merging an arbitrary number of streams in a loop. This comes with a small
/// runtime overhead but enables flexible stream composition.
///
/// # Example
///
/// ```no_run
/// use wordle::wordlist::stream::from_unsorted_zst_file;
///
/// let inputs = ["a.zst", "b.zst", "c.zst"];
/// let mut stream = from_unsorted_zst_file(inputs[0])?.boxed();
///
/// for input in &inputs[1..] {
///     stream = stream.merge(from_unsorted_zst_file(input)?.boxed());
/// }
///
/// stream
///     .filter(|w| w.len() == 5)
///     .to_lowercase()
///     .dedup()
///     .write_to_zst_file("output.zst")?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BoxedWordStream {
    inner: Box<dyn Iterator<Item = io::Result<Word>>>,
}

impl BoxedWordStream {
    /// Creates a new BoxedWordStream from any iterator.
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = io::Result<Word>> + 'static,
    {
        BoxedWordStream {
            inner: Box::new(iter),
        }
    }

    /// Merges this stream with another boxed stream.
    ///
    /// Both streams must be sorted in case-fold order.
    pub fn merge(self, other: BoxedWordStream) -> Self {
        BoxedWordStream::new(MergeStream::new(
            self.inner.peekable(),
            other.inner.peekable(),
        ))
    }

    /// Filters items using a predicate.
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: FnMut(&str) -> bool + 'static,
    {
        BoxedWordStream::new(FilterStream::new(self.inner.peekable(), predicate))
    }

    /// Converts all items to lowercase.
    pub fn to_lowercase(self) -> Self {
        BoxedWordStream::new(LowercaseStream::new(self.inner.peekable()))
    }

    /// Removes consecutive duplicates using case-fold equality.
    pub fn dedup(self) -> Self {
        BoxedWordStream::new(DedupStream::new(self.inner.peekable()))
    }

    /// Writes all items to a file, one per line.
    pub fn write_to_file(self, path: impl AsRef<Path>) -> io::Result<()> {
        sinks::write_to_file(self.inner, path)
    }

    /// Writes all items to a zstd-compressed file, one per line.
    pub fn write_to_zst_file(self, path: impl AsRef<Path>) -> io::Result<()> {
        sinks::write_to_zst_file(self.inner, path)
    }
}

impl Iterator for BoxedWordStream {
    type Item = io::Result<Word>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(
        items: I,
    ) -> impl Iterator<Item = io::Result<Word>> {
        items.into_iter().map(|s| Ok(Word(s.to_string())))
    }

    fn collect_strings(stream: BoxedWordStream) -> Vec<String> {
        stream.map(|r| r.unwrap().0).collect()
    }

    #[test]
    fn test_basic_iteration() {
        let stream = BoxedWordStream::new(ok_iter(["apple", "banana", "cherry"]));
        assert_eq!(
            collect_strings(stream),
            vec!["apple", "banana", "cherry"]
        );
    }

    #[test]
    fn test_filter() {
        let stream = BoxedWordStream::new(ok_iter(["a", "bb", "ccc", "dddd"]))
            .filter(|w| w.len() >= 2);
        assert_eq!(collect_strings(stream), vec!["bb", "ccc", "dddd"]);
    }

    #[test]
    fn test_to_lowercase() {
        let stream =
            BoxedWordStream::new(ok_iter(["Apple", "BANANA", "Cherry"])).to_lowercase();
        assert_eq!(
            collect_strings(stream),
            vec!["apple", "banana", "cherry"]
        );
    }

    #[test]
    fn test_dedup() {
        // Input must be sorted for dedup to work correctly
        let stream =
            BoxedWordStream::new(ok_iter(["apple", "Apple", "APPLE", "banana"])).dedup();
        assert_eq!(collect_strings(stream), vec!["apple", "banana"]);
    }

    #[test]
    fn test_merge_two_streams() {
        let stream1 = BoxedWordStream::new(ok_iter(["apple", "cherry"]));
        let stream2 = BoxedWordStream::new(ok_iter(["banana", "date"]));
        let merged = stream1.merge(stream2);
        assert_eq!(
            collect_strings(merged),
            vec!["apple", "banana", "cherry", "date"]
        );
    }

    #[test]
    fn test_merge_three_streams_in_loop() {
        let inputs = [
            vec!["apple", "date"],
            vec!["banana", "elderberry"],
            vec!["cherry", "fig"],
        ];

        let mut stream = BoxedWordStream::new(ok_iter(inputs[0].clone()));
        for input in &inputs[1..] {
            stream = stream.merge(BoxedWordStream::new(ok_iter(input.clone())));
        }

        assert_eq!(
            collect_strings(stream),
            vec!["apple", "banana", "cherry", "date", "elderberry", "fig"]
        );
    }

    #[test]
    fn test_full_pipeline() {
        // Simulate merging two unsorted-but-now-sorted streams
        let stream1 = BoxedWordStream::new(ok_iter(["Apple", "apple", "Cherry"]));
        let stream2 = BoxedWordStream::new(ok_iter(["banana", "Banana", "date"]));

        let result = stream1
            .merge(stream2)
            .filter(|w| w.len() >= 5)
            .to_lowercase()
            .dedup();

        assert_eq!(
            collect_strings(result),
            vec!["apple", "banana", "cherry"]
        );
    }

    #[test]
    fn test_empty_stream() {
        let stream = BoxedWordStream::new(ok_iter([]));
        assert_eq!(collect_strings(stream), Vec::<String>::new());
    }

    #[test]
    fn test_merge_with_empty() {
        let stream1 = BoxedWordStream::new(ok_iter(["apple", "banana"]));
        let stream2 = BoxedWordStream::new(ok_iter([]));
        let merged = stream1.merge(stream2);
        assert_eq!(collect_strings(merged), vec!["apple", "banana"]);
    }

    #[test]
    fn test_error_propagates() {
        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok(Word("banana".to_string())),
        ];
        let stream = BoxedWordStream::new(items.into_iter());
        let results: Vec<_> = stream.collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
}

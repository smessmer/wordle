//! Stream processing pipeline for sorted word lists.
//!
//! This module provides a composable, lazy stream processing pipeline
//! for word lists. All streams are guaranteed to be sorted in case-fold
//! order (lowercase primary, original case secondary).
//!
//! # Example
//!
//! ```no_run
//! use wordle::wordlist::stream::{from_sorted_file, from_unsorted_file};
//!
//! // Load a sorted file, filter to 5-letter words, collect
//! let words = from_sorted_file("words.txt")?
//!     .filter(|w| w.len() == 5)
//!     .collect_to_set()?;
//!
//! // Load unsorted, normalize, deduplicate, write
//! from_unsorted_file("raw.txt")?
//!     .to_lowercase()
//!     .dedup()
//!     .write_to_file("output.txt")?;
//!
//! // Load from zstd-compressed file, process, write to compressed file
//! use wordle::wordlist::stream::{from_sorted_zst_file, from_unsorted_zst_file};
//!
//! from_sorted_zst_file("words.zst")?
//!     .filter(|w| w.len() == 5)
//!     .write_to_zst_file("filtered.zst")?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Case-Fold Ordering
//!
//! Strings are ordered by:
//! 1. Primary key: lowercase form of characters
//! 2. Secondary key: original case (lowercase < uppercase)
//!
//! This means `"apple" < "Apple" < "APPLE" < "banana"`.

mod boxed;
mod sinks;
mod sources;
pub(crate) mod transforms;
mod word_stream;

pub use boxed::BoxedWordStream;
pub use super::ordering::case_fold_cmp;
pub use sources::{
    from_sorted_file, from_sorted_reader, from_sorted_zst_file, from_unsorted_file,
    from_unsorted_reader, from_unsorted_zst_file, SortedLines, UnsortedWords,
};
pub use word_stream::WordStream;

use std::fs::File;
use std::io::{self, BufReader};
use std::iter::Peekable;
use std::path::Path;

use zstd::Decoder;

use crate::wordlist::{Word, WordSet};
use transforms::{filter_non_alphabetic, DedupStream, FilterStream, LowercaseStream, MergeStream};

/// Type alias for the iterator produced by `WordStream::from_word_set`.
type WordSetIter = std::iter::Map<
    <WordSet as IntoIterator>::IntoIter,
    fn(Word) -> io::Result<Word>,
>;

impl WordStream<SortedLines<BufReader<File>>> {
    /// Creates a WordStream from a pre-sorted file.
    ///
    /// Reads lines lazily without loading the entire file into memory.
    /// Panics during iteration if the file is not sorted in case-fold order.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    ///
    /// # Panics
    ///
    /// Panics during iteration if the file is not sorted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::WordStream;
    ///
    /// let stream = WordStream::from_sorted_file("words.txt")?;
    /// for word in stream {
    ///     println!("{}", word?);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_sorted_file(path: impl AsRef<Path>) -> io::Result<Self> {
        sources::from_sorted_file(path)
    }
}

impl WordStream<SortedLines<BufReader<Decoder<'static, BufReader<File>>>>> {
    /// Creates a WordStream from a pre-sorted zstd-compressed file.
    ///
    /// Reads lines lazily, decompressing on the fly.
    /// Panics during iteration if the file is not sorted in case-fold order.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or is not valid zstd.
    ///
    /// # Panics
    ///
    /// Panics during iteration if the file is not sorted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::WordStream;
    ///
    /// let stream = WordStream::from_sorted_zst_file("words.zst")?;
    /// for word in stream {
    ///     println!("{}", word?);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_sorted_zst_file(path: impl AsRef<Path>) -> io::Result<Self> {
        sources::from_sorted_zst_file(path)
    }
}

impl WordStream<UnsortedWords> {
    /// Creates a WordStream from an unsorted file.
    ///
    /// Loads the entire file into memory, sorts it using case-fold ordering,
    /// and returns a stream over the sorted data.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::WordStream;
    ///
    /// let stream = WordStream::from_unsorted_file("raw_words.txt")?;
    /// for word in stream {
    ///     println!("{}", word?);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_unsorted_file(path: impl AsRef<Path>) -> io::Result<Self> {
        sources::from_unsorted_file(path)
    }

    /// Creates a WordStream from an unsorted zstd-compressed file.
    ///
    /// Loads and decompresses the entire file into memory, sorts it using case-fold ordering,
    /// and returns a stream over the sorted data.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, is not valid zstd, or cannot be read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::WordStream;
    ///
    /// let stream = WordStream::from_unsorted_zst_file("raw_words.zst")?;
    /// for word in stream {
    ///     println!("{}", word?);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_unsorted_zst_file(path: impl AsRef<Path>) -> io::Result<Self> {
        sources::from_unsorted_zst_file(path)
    }
}

impl WordStream<WordSetIter> {
    /// Creates a WordStream from a WordSet.
    ///
    /// Since WordSet is already sorted, this is an infallible operation
    /// that wraps each word in `Ok()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::{from_sorted_file, WordStream};
    ///
    /// // Load, filter, collect to set, then convert back to stream
    /// let set = from_sorted_file("words.txt")?
    ///     .filter(|w| w.len() == 5)
    ///     .collect_to_set()?;
    ///
    /// // Convert set back to stream for further processing
    /// let stream = WordStream::from_word_set(set);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_word_set(set: WordSet) -> Self {
        WordStream::new(set.into_iter().map(Ok as fn(Word) -> io::Result<Word>))
    }
}

impl<I> WordStream<I>
where
    I: Iterator<Item = io::Result<Word>>,
{
    /// Filters items using a predicate.
    ///
    /// Only items where `predicate(&str)` returns `true` are kept.
    /// Errors pass through unchanged.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// let five_letter_words = from_sorted_file("words.txt")?
    ///     .filter(|w| w.len() == 5)
    ///     .collect_to_set()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn filter<F>(self, predicate: F) -> WordStream<FilterStream<Peekable<I>, F>>
    where
        F: FnMut(&str) -> bool,
    {
        WordStream::new(FilterStream::new(self.into_inner(), predicate))
    }

    /// Converts all items to lowercase.
    ///
    /// This preserves the sort order because case-fold ordering uses
    /// lowercase as the primary sort key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// from_sorted_file("words.txt")?
    ///     .to_lowercase()
    ///     .write_to_file("lowercase_words.txt")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn to_lowercase(self) -> WordStream<LowercaseStream<Peekable<I>>> {
        WordStream::new(LowercaseStream::new(self.into_inner()))
    }

    /// Removes consecutive duplicates using case-fold equality.
    ///
    /// Since the stream is sorted in case-fold order, this removes all
    /// duplicates. For example, "apple", "Apple", and "APPLE" are all
    /// considered equal; only the first occurrence is kept.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_unsorted_file;
    ///
    /// from_unsorted_file("words.txt")?
    ///     .to_lowercase()
    ///     .dedup()
    ///     .write_to_file("unique_words.txt")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn dedup(self) -> WordStream<DedupStream<Peekable<I>>> {
        WordStream::new(DedupStream::new(self.into_inner()))
    }

    /// Filters out words with non-alphabetic characters, warning on stderr.
    ///
    /// Words containing any non-alphabetic character (e.g., digits, punctuation)
    /// are removed from the stream, and a warning is printed to stderr for each.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_unsorted_file;
    ///
    /// from_unsorted_file("words.txt")?
    ///     .filter_non_alphabetic()
    ///     .write_to_file("alphabetic_words.txt")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn filter_non_alphabetic(
        self,
    ) -> WordStream<FilterStream<Peekable<I>, impl FnMut(&str) -> bool>> {
        WordStream::new(filter_non_alphabetic(self.into_inner()))
    }

    /// Merges this stream with another sorted stream.
    ///
    /// Both streams must be sorted in case-fold order. The resulting stream
    /// maintains the sorted order by comparing heads of both streams and
    /// emitting the smaller one.
    ///
    /// Duplicates are preserved (not deduplicated).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// let merged = from_sorted_file("words1.txt")?
    ///     .merge(from_sorted_file("words2.txt")?)
    ///     .collect_to_set()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn merge<I2>(self, other: WordStream<I2>) -> WordStream<MergeStream<I, I2>>
    where
        I2: Iterator<Item = io::Result<Word>>,
    {
        WordStream::new(MergeStream::new(self.into_inner(), other.into_inner()))
    }

    /// Collects all items into a `WordSet`.
    ///
    /// # Errors
    ///
    /// Returns an error if any item in the stream is an I/O error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// let words = from_sorted_file("words.txt")?
    ///     .filter(|w| w.len() == 5)
    ///     .collect_to_set()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn collect_to_set(self) -> io::Result<WordSet> {
        sinks::collect_to_set(self.into_inner())
    }

    /// Writes all items to a file, one per line.
    ///
    /// Uses buffered writing for efficiency. This is a streaming operation
    /// that doesn't require loading all items into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, written to,
    /// or if any item in the stream is an I/O error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// from_sorted_file("words.txt")?
    ///     .filter(|w| w.chars().all(|c| c.is_alphabetic()))
    ///     .write_to_file("alphabetic_words.txt")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn write_to_file(self, path: impl AsRef<Path>) -> io::Result<()> {
        sinks::write_to_file(self.into_inner(), path)
    }

    /// Writes all items to a zstd-compressed file, one per line.
    ///
    /// Uses buffered writing and default compression level for efficiency.
    /// This is a streaming operation that doesn't require loading all items into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, written to,
    /// or if any item in the stream is an I/O error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wordle::wordlist::stream::from_sorted_file;
    ///
    /// from_sorted_file("words.txt")?
    ///     .filter(|w| w.chars().all(|c| c.is_alphabetic()))
    ///     .write_to_zst_file("alphabetic_words.zst")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn write_to_zst_file(self, path: impl AsRef<Path>) -> io::Result<()> {
        sinks::write_to_zst_file(self.into_inner(), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write};
    use zstd::Encoder;

    fn create_temp_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_stream_integration_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn create_temp_zst_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_stream_integration_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = std::fs::File::create(&path).unwrap();
        let mut encoder = Encoder::new(file, 0).unwrap();
        write!(encoder, "{}", content).unwrap();
        encoder.finish().unwrap();
        path
    }

    #[test]
    fn test_full_pipeline_sorted_file() {
        let path = create_temp_file("apple\nApple\nAPPLE\nbanana\nBanana\ncherry\n");
        let set = from_sorted_file(&path)
            .unwrap()
            .to_lowercase()
            .dedup()
            .collect_to_set()
            .unwrap();

        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_full_pipeline_unsorted_file() {
        let path = create_temp_file("cherry\nAPPLE\napple\nbanana\nApple\n");
        let set = from_unsorted_file(&path)
            .unwrap()
            .to_lowercase()
            .dedup()
            .collect_to_set()
            .unwrap();

        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_filter_chain() {
        let path = create_temp_file("a\nbb\nccc\ndddd\neeeee\n");
        let set = from_sorted_file(&path)
            .unwrap()
            .filter(|w| w.len() >= 2)
            .filter(|w| w.len() <= 4)
            .collect_to_set()
            .unwrap();

        assert_eq!(set.len(), 3);
        assert!(set.contains("bb"));
        assert!(set.contains("ccc"));
        assert!(set.contains("dddd"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_file() {
        let input_path = create_temp_file("apple\nbanana\ncherry\n");
        let output_path = std::env::temp_dir().join(format!(
            "test_write_output_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        from_sorted_file(&input_path)
            .unwrap()
            .filter(|w| w.starts_with('b') || w.starts_with('c'))
            .write_to_file(&output_path)
            .unwrap();

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "banana\ncherry\n");

        std::fs::remove_file(input_path).ok();
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_full_pipeline_sorted_zst_file() {
        let path = create_temp_zst_file("apple\nApple\nAPPLE\nbanana\nBanana\ncherry\n");
        let set = from_sorted_zst_file(&path)
            .unwrap()
            .to_lowercase()
            .dedup()
            .collect_to_set()
            .unwrap();

        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_full_pipeline_unsorted_zst_file() {
        let path = create_temp_zst_file("cherry\nAPPLE\napple\nbanana\nApple\n");
        let set = from_unsorted_zst_file(&path)
            .unwrap()
            .to_lowercase()
            .dedup()
            .collect_to_set()
            .unwrap();

        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_zst_file() {
        let input_path = create_temp_file("apple\nbanana\ncherry\n");
        let output_path = std::env::temp_dir().join(format!(
            "test_write_output_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        from_sorted_file(&input_path)
            .unwrap()
            .filter(|w| w.starts_with('b') || w.starts_with('c'))
            .write_to_zst_file(&output_path)
            .unwrap();

        // Read and decompress to verify
        let file = File::open(&output_path).unwrap();
        let mut decoder = Decoder::new(file).unwrap();
        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();
        assert_eq!(content, "banana\ncherry\n");

        std::fs::remove_file(input_path).ok();
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_zst_roundtrip() {
        // Write to zst, then read back and verify
        let input_path = create_temp_file("apple\nbanana\ncherry\n");
        let zst_path = std::env::temp_dir().join(format!(
            "test_roundtrip_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        from_sorted_file(&input_path)
            .unwrap()
            .write_to_zst_file(&zst_path)
            .unwrap();

        let words: Vec<String> = from_sorted_zst_file(&zst_path)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();

        assert_eq!(words, vec!["apple", "banana", "cherry"]);

        std::fs::remove_file(input_path).ok();
        std::fs::remove_file(zst_path).ok();
    }
}

//! Lazy reading for pre-sorted word sources.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;

use zstd::Decoder;

use crate::Word;
use crate::stream::word_stream::WordStream;

/// Iterator that reads lines from any `BufRead` source, trimming whitespace and skipping empty lines.
///
/// This is the underlying iterator type for sorted word streams.
pub struct SortedLines<R: BufRead> {
    lines: Lines<R>,
}

impl<R: BufRead> SortedLines<R> {
    /// Creates a new `SortedLines` iterator from a buffered reader.
    pub fn new(reader: R) -> Self {
        Self {
            lines: reader.lines(),
        }
    }
}

impl<R: BufRead> Iterator for SortedLines<R> {
    type Item = io::Result<Word>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lines.next()? {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    return Some(Ok(Word(trimmed.to_string())));
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

/// Creates a WordStream from any buffered reader containing pre-sorted words.
///
/// Reads lines lazily. Panics during iteration if the data is not sorted in case-fold order.
///
/// # Panics
///
/// Panics during iteration if the data is not sorted.
pub fn from_sorted_reader<R: BufRead>(reader: R) -> WordStream<SortedLines<R>> {
    WordStream::new(SortedLines::new(reader))
}

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
/// use wordle::wordlist::stream::from_sorted_file;
///
/// let stream = from_sorted_file("words.txt")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_sorted_file(
    path: impl AsRef<Path>,
) -> io::Result<WordStream<SortedLines<BufReader<File>>>> {
    let file = File::open(path)?;
    Ok(from_sorted_reader(BufReader::new(file)))
}

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
/// use wordle::wordlist::stream::from_sorted_zst_file;
///
/// let stream = from_sorted_zst_file("words.zst")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_sorted_zst_file(
    path: impl AsRef<Path>,
) -> io::Result<WordStream<SortedLines<BufReader<Decoder<'static, BufReader<File>>>>>> {
    let file = File::open(path)?;
    let decoder = Decoder::new(file)?;
    Ok(from_sorted_reader(BufReader::new(decoder)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_sorted_file_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn create_temp_zst_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_sorted_file_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = File::create(&path).unwrap();
        let mut encoder = zstd::Encoder::new(file, 0).unwrap();
        write!(encoder, "{}", content).unwrap();
        encoder.finish().unwrap();
        path
    }

    #[test]
    fn test_read_sorted_file() {
        let path = create_temp_file("apple\nbanana\ncherry\n");
        let stream = from_sorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_skips_empty_lines() {
        let path = create_temp_file("apple\n\nbanana\n  \ncherry\n");
        let stream = from_sorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_trims_whitespace() {
        let path = create_temp_file("  apple  \n  banana\ncherry  \n");
        let stream = from_sorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    #[should_panic(expected = "not sorted")]
    fn test_unsorted_file_panics() {
        let path = create_temp_file("banana\napple\n");
        let stream = from_sorted_file(&path).unwrap();
        let _: Vec<_> = stream.collect();
        // Cleanup won't run due to panic, but that's ok for tests
    }

    #[test]
    fn test_file_not_found() {
        let result = from_sorted_file("/nonexistent/path/to/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let path = create_temp_file("");
        let stream = from_sorted_file(&path).unwrap();
        let words: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(words.is_empty());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_read_sorted_zst_file() {
        let path = create_temp_zst_file("apple\nbanana\ncherry\n");
        let stream = from_sorted_zst_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_zst_skips_empty_lines() {
        let path = create_temp_zst_file("apple\n\nbanana\n  \ncherry\n");
        let stream = from_sorted_zst_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    #[should_panic(expected = "not sorted")]
    fn test_unsorted_zst_file_panics() {
        let path = create_temp_zst_file("banana\napple\n");
        let stream = from_sorted_zst_file(&path).unwrap();
        let _: Vec<_> = stream.collect();
    }

    #[test]
    fn test_zst_file_not_found() {
        let result = from_sorted_zst_file("/nonexistent/path/to/file.zst");
        assert!(result.is_err());
    }
}

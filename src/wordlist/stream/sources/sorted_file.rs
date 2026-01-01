//! Lazy file reading for pre-sorted word files.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::Path;

use crate::wordlist::stream::word_stream::WordStream;
use crate::wordlist::Word;

/// Iterator that reads lines from a file, trimming whitespace and skipping empty lines.
///
/// This is the underlying iterator type for `WordStream::from_sorted_file()`.
pub struct SortedFileLines {
    lines: Lines<BufReader<File>>,
}

impl SortedFileLines {
    fn new(file: File) -> Self {
        Self {
            lines: BufReader::new(file).lines(),
        }
    }
}

impl Iterator for SortedFileLines {
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
pub fn from_sorted_file(path: impl AsRef<Path>) -> io::Result<WordStream<SortedFileLines>> {
    let file = File::open(path)?;
    Ok(WordStream::new(SortedFileLines::new(file)))
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
}

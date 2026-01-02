//! Loading with in-memory sorting for unsorted word sources.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use zstd::Decoder;

use crate::wordlist::stream::word_stream::WordStream;
use crate::wordlist::Word;

/// Iterator over words loaded from an unsorted source and sorted in memory.
///
/// This is the underlying iterator type for unsorted word streams.
pub struct UnsortedWords {
    inner: std::vec::IntoIter<Word>,
}

impl UnsortedWords {
    pub fn new(words: Vec<Word>) -> Self {
        Self {
            inner: words.into_iter(),
        }
    }
}

impl Iterator for UnsortedWords {
    type Item = io::Result<Word>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Ok)
    }
}

/// Creates a WordStream from any buffered reader containing unsorted words.
///
/// Loads all lines into memory, sorts them using case-fold ordering,
/// and returns a stream over the sorted data.
///
/// # Errors
///
/// Returns an error if reading fails.
pub fn from_unsorted_reader<R: BufRead>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
    // Read all lines, trim, skip empty
    let mut words: Vec<Word> = reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(Word(trimmed.to_string()))
            }
        })
        .collect();

    // Sort using case-fold ordering (Word implements Ord with case-fold)
    words.sort();

    Ok(WordStream::new(UnsortedWords::new(words)))
}

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
/// use wordle::wordlist::stream::from_unsorted_file;
///
/// let stream = from_unsorted_file("raw_words.txt")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_unsorted_file(path: impl AsRef<Path>) -> io::Result<WordStream<UnsortedWords>> {
    let file = File::open(path)?;
    from_unsorted_reader(BufReader::new(file))
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
/// use wordle::wordlist::stream::from_unsorted_zst_file;
///
/// let stream = from_unsorted_zst_file("raw_words.zst")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_unsorted_zst_file(path: impl AsRef<Path>) -> io::Result<WordStream<UnsortedWords>> {
    let file = File::open(path)?;
    let decoder = Decoder::new(file)?;
    from_unsorted_reader(BufReader::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_unsorted_file_{}.txt",
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
            "test_unsorted_file_{}.zst",
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
    fn test_sorts_unsorted_file() {
        let path = create_temp_file("cherry\napple\nbanana\n");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_case_fold_sorting() {
        let path = create_temp_file("APPLE\napple\nApple\nbanana\n");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        // case-fold order: apple < Apple < APPLE < banana
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_skips_empty_lines() {
        let path = create_temp_file("cherry\n\napple\n  \nbanana\n");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_trims_whitespace() {
        let path = create_temp_file("  cherry  \n  apple\nbanana  \n");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_file_not_found() {
        let result = from_unsorted_file("/nonexistent/path/to/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let path = create_temp_file("");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(words.is_empty());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_german_umlauts_sorting() {
        let path = create_temp_file("Ärger\närger\nbär\nÄRGER\n");
        let stream = from_unsorted_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        // In Unicode, 'b' < 'ä', so: bär < ärger < Ärger < ÄRGER
        assert_eq!(words, vec!["bär", "ärger", "Ärger", "ÄRGER"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_sorts_unsorted_zst_file() {
        let path = create_temp_zst_file("cherry\napple\nbanana\n");
        let stream = from_unsorted_zst_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_zst_case_fold_sorting() {
        let path = create_temp_zst_file("APPLE\napple\nApple\nbanana\n");
        let stream = from_unsorted_zst_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_zst_file_not_found() {
        let result = from_unsorted_zst_file("/nonexistent/path/to/file.zst");
        assert!(result.is_err());
    }
}

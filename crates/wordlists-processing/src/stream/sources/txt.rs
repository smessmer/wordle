//! Loading words from plain text streams with in-memory sorting.

use std::io::{self, BufRead, BufReader, Read};

use zstd::Decoder;

use crate::stream::word_stream::WordStream;
use crate::Word;

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

/// Creates a WordStream from a buffered reader containing plain text words.
///
/// Loads all lines into memory, sorts them using case-fold ordering,
/// and returns a stream over the sorted data.
///
/// # Errors
///
/// Returns an error if reading fails.
///
/// # Example
///
/// ```no_run
/// use std::io::Cursor;
/// use wordle::wordlist::stream::from_txt;
///
/// let data = b"cherry\napple\nbanana\n";
/// let stream = from_txt(Cursor::new(data))?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_txt<R: BufRead>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
    // Read all lines, trim, skip empty
    let mut words: Vec<Word> = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            words.push(Word(trimmed.to_string()));
        }
    }

    // Sort using case-fold ordering (Word implements Ord with case-fold)
    words.sort();

    Ok(WordStream::new(UnsortedWords::new(words)))
}

/// Creates a WordStream from a zstd-compressed plain text stream.
///
/// Wraps the reader in a zstd decoder, then parses as plain text.
/// Loads all lines into memory, sorts them using case-fold ordering,
/// and returns a stream over the sorted data.
///
/// # Errors
///
/// Returns an error if reading fails or the stream is not valid zstd.
///
/// # Example
///
/// ```no_run
/// use std::io::Cursor;
/// use wordle::wordlist::stream::from_txt_zstd;
///
/// let compressed_data: &[u8] = include_bytes!("some_file.txt.zst");
/// let stream = from_txt_zstd(Cursor::new(compressed_data))?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_txt_zstd<R: Read>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
    let decoder = Decoder::new(reader)?;
    from_txt(BufReader::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn compress(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(Cursor::new(data), 0).unwrap()
    }

    #[test]
    fn test_sorts_unsorted() {
        let data = b"cherry\napple\nbanana\n";
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_case_fold_sorting() {
        let data = b"APPLE\napple\nApple\nbanana\n";
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        // case-fold order: apple < Apple < APPLE < banana
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
    }

    #[test]
    fn test_skips_empty_lines() {
        let data = b"cherry\n\napple\n  \nbanana\n";
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_trims_whitespace() {
        let data = b"  cherry  \n  apple\nbanana  \n";
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_empty() {
        let data = b"";
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(words.is_empty());
    }

    #[test]
    fn test_german_umlauts_sorting() {
        let data = "Ärger\närger\nbär\nÄRGER\n".as_bytes();
        let stream = from_txt(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        // In Unicode, 'b' < 'ä', so: bär < ärger < Ärger < ÄRGER
        assert_eq!(words, vec!["bär", "ärger", "Ärger", "ÄRGER"]);
    }

    #[test]
    fn test_txt_zstd() {
        let data = compress(b"cherry\napple\nbanana\n");
        let stream = from_txt_zstd(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_txt_zstd_case_fold_sorting() {
        let data = compress(b"APPLE\napple\nApple\nbanana\n");
        let stream = from_txt_zstd(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
    }

    #[test]
    fn test_txt_zstd_invalid() {
        let data = b"not valid zstd data";
        let result = from_txt_zstd(Cursor::new(data));
        assert!(result.is_err());
    }
}

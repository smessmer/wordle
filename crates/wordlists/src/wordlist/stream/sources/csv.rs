//! Loading words from CSV streams with in-memory sorting.

use std::io::{self, BufReader, Read};

use zstd::Decoder;

use super::txt::UnsortedWords;
use crate::wordlist::stream::word_stream::WordStream;
use crate::wordlist::Word;

/// Creates a WordStream from a CSV reader, using the first column as words.
///
/// Uses the `csv` crate for proper parsing including quoted fields.
/// Loads all rows, extracts the first field, sorts using case-fold ordering.
///
/// # Errors
///
/// Returns an error if reading fails or CSV parsing encounters invalid data.
///
/// # Example
///
/// ```no_run
/// use std::io::Cursor;
/// use wordle::wordlist::stream::from_csv;
///
/// let data = b"apple,1\nbanana,2\ncherry,3\n";
/// let stream = from_csv(Cursor::new(data))?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_csv<R: Read>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);

    let mut words: Vec<Word> = Vec::new();

    for result in csv_reader.records() {
        let record = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if let Some(first_field) = record.get(0) {
            let trimmed = first_field.trim();
            if !trimmed.is_empty() {
                words.push(Word(trimmed.to_string()));
            }
        }
    }

    words.sort();
    Ok(WordStream::new(UnsortedWords::new(words)))
}

/// Creates a WordStream from a zstd-compressed CSV stream.
///
/// Wraps the reader in a zstd decoder, then parses as CSV.
/// Uses the `csv` crate for proper parsing including quoted fields.
/// Loads all rows, extracts the first field, sorts using case-fold ordering.
///
/// # Errors
///
/// Returns an error if reading fails, the stream is not valid zstd,
/// or CSV parsing encounters invalid data.
///
/// # Example
///
/// ```no_run
/// use std::io::Cursor;
/// use wordle::wordlist::stream::from_csv_zstd;
///
/// let compressed_data: &[u8] = include_bytes!("some_file.csv.zst");
/// let stream = from_csv_zstd(Cursor::new(compressed_data))?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_csv_zstd<R: Read>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
    let decoder = Decoder::new(reader)?;
    from_csv(BufReader::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn compress(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(Cursor::new(data), 0).unwrap()
    }

    #[test]
    fn test_basic_csv() {
        let data = b"apple,1,ignored\nbanana,2,data\ncherry,3,here\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_csv_with_quotes() {
        let data = b"\"hello,world\",ignored\ntest,data\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["hello,world", "test"]);
    }

    #[test]
    fn test_csv_with_spaces() {
        let data = b"  apple  ,data\n  banana,more\ncherry  ,stuff\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_csv_empty_first_field() {
        let data = b"apple,1\n,empty\nbanana,2\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana"]);
    }

    #[test]
    fn test_csv_sorts_words() {
        let data = b"cherry,1\napple,2\nbanana,3\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_csv_case_fold_sorting() {
        let data = b"APPLE,1\napple,2\nApple,3\nbanana,4\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
    }

    #[test]
    fn test_csv_empty() {
        let data = b"";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(words.is_empty());
    }

    #[test]
    fn test_csv_single_column() {
        let data = b"apple\nbanana\ncherry\n";
        let stream = from_csv(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_csv_zstd() {
        let data = compress(b"cherry,1\napple,2\nbanana,3\n");
        let stream = from_csv_zstd(Cursor::new(data)).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_csv_zstd_invalid() {
        let data = b"not valid zstd data";
        let result = from_csv_zstd(Cursor::new(data));
        assert!(result.is_err());
    }
}

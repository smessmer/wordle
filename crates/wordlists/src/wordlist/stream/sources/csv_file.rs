//! Loading words from CSV files with in-memory sorting.

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use zstd::Decoder;

use super::unsorted_file::UnsortedWords;
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
/// use wordle::wordlist::stream::from_csv_file;
///
/// let stream = from_csv_file("words.csv")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_csv_reader<R: Read>(reader: R) -> io::Result<WordStream<UnsortedWords>> {
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

/// Creates a WordStream from a CSV file.
///
/// Uses the `csv` crate for proper parsing including quoted fields.
/// Loads all rows, extracts the first field, sorts using case-fold ordering.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or CSV parsing fails.
///
/// # Example
///
/// ```no_run
/// use wordle::wordlist::stream::from_csv_file;
///
/// let stream = from_csv_file("words.csv")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_csv_file(path: impl AsRef<Path>) -> io::Result<WordStream<UnsortedWords>> {
    let file = File::open(path)?;
    from_csv_reader(BufReader::new(file))
}

/// Creates a WordStream from a zstd-compressed CSV file.
///
/// Uses the `csv` crate for proper parsing including quoted fields.
/// Decompresses and loads all rows, extracts the first field,
/// sorts using case-fold ordering.
///
/// # Errors
///
/// Returns an error if the file cannot be opened, is not valid zstd,
/// or CSV parsing fails.
///
/// # Example
///
/// ```no_run
/// use wordle::wordlist::stream::from_csv_zst_file;
///
/// let stream = from_csv_zst_file("words.csv.zst")?;
/// for word in stream {
///     println!("{}", word?);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn from_csv_zst_file(path: impl AsRef<Path>) -> io::Result<WordStream<UnsortedWords>> {
    let file = File::open(path)?;
    let decoder = Decoder::new(file)?;
    from_csv_reader(BufReader::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_csv_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_csv_file_{}.csv",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn create_temp_csv_zst_file(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "test_csv_file_{}.csv.zst",
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
    fn test_basic_csv() {
        let path = create_temp_csv_file("apple,1,ignored\nbanana,2,data\ncherry,3,here\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_with_quotes() {
        let path = create_temp_csv_file("\"hello,world\",ignored\ntest,data\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["hello,world", "test"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_with_spaces() {
        let path = create_temp_csv_file("  apple  ,data\n  banana,more\ncherry  ,stuff\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_empty_first_field() {
        let path = create_temp_csv_file("apple,1\n,empty\nbanana,2\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_sorts_words() {
        let path = create_temp_csv_file("cherry,1\napple,2\nbanana,3\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_case_fold_sorting() {
        let path = create_temp_csv_file("APPLE,1\napple,2\nApple,3\nbanana,4\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "Apple", "APPLE", "banana"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_file_not_found() {
        let result = from_csv_file("/nonexistent/path/to/file.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_empty_file() {
        let path = create_temp_csv_file("");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(words.is_empty());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_single_column() {
        let path = create_temp_csv_file("apple\nbanana\ncherry\n");
        let stream = from_csv_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_zst_file() {
        let path = create_temp_csv_zst_file("cherry,1\napple,2\nbanana,3\n");
        let stream = from_csv_zst_file(&path).unwrap();
        let words: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(words, vec!["apple", "banana", "cherry"]);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_zst_file_not_found() {
        let result = from_csv_zst_file("/nonexistent/path/to/file.csv.zst");
        assert!(result.is_err());
    }
}

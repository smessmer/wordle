//! Terminal operations for WordStream.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use zstd::Encoder;

use crate::wordlist::{Word, WordSet};

/// Collects an iterator of `io::Result<Word>` into a `WordSet`.
///
/// # Errors
///
/// Returns an error if any item in the iterator is an error.
pub fn collect_to_set<I>(iter: I) -> io::Result<WordSet>
where
    I: Iterator<Item = io::Result<Word>>,
{
    let words: Result<Vec<Word>, io::Error> = iter.collect();
    Ok(words?.into_iter().map(|w| w.0).collect())
}

/// Writes items from an iterator to any writer, one per line.
///
/// # Errors
///
/// Returns an error if writing fails or if any item in the iterator is an error.
pub fn write_to_writer<I, W>(iter: I, mut writer: W) -> io::Result<()>
where
    I: Iterator<Item = io::Result<Word>>,
    W: Write,
{
    for item in iter {
        let w = item?;
        writeln!(writer, "{}", w.0)?;
    }
    writer.flush()?;
    Ok(())
}

/// Writes items from an iterator to a file, one per line.
///
/// Uses buffered writing for efficiency.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written to,
/// or if any item in the iterator is an error.
pub fn write_to_file<I>(iter: I, path: impl AsRef<Path>) -> io::Result<()>
where
    I: Iterator<Item = io::Result<Word>>,
{
    let file = File::create(path)?;
    write_to_writer(iter, BufWriter::new(file))
}

/// Writes items from an iterator to a zstd-compressed file, one per line.
///
/// Uses buffered writing and default compression level for efficiency.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written to,
/// or if any item in the iterator is an error.
pub fn write_to_zst_file<I>(iter: I, path: impl AsRef<Path>) -> io::Result<()>
where
    I: Iterator<Item = io::Result<Word>>,
{
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let encoder = Encoder::new(writer, 19)?.auto_finish();
    write_to_writer(iter, encoder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(
        items: I,
    ) -> impl Iterator<Item = io::Result<Word>> {
        items.into_iter().map(|s| Ok(Word(s.to_string())))
    }

    #[test]
    fn test_collect_to_set() {
        let set = collect_to_set(ok_iter(["cherry", "apple", "banana"])).unwrap();
        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));
    }

    #[test]
    fn test_collect_to_set_deduplicates() {
        let set = collect_to_set(ok_iter(["apple", "apple", "banana"])).unwrap();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_collect_to_set_empty() {
        let set = collect_to_set(ok_iter([])).unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn test_collect_to_set_error() {
        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
        ];
        let result = collect_to_set(items.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_to_file() {
        let path = std::env::temp_dir().join(format!(
            "test_write_stream_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        write_to_file(ok_iter(["apple", "banana", "cherry"]), &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "apple\nbanana\ncherry\n");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_file_empty() {
        let path = std::env::temp_dir().join(format!(
            "test_write_empty_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        write_to_file(ok_iter([]), &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.is_empty());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_file_error_in_stream() {
        let path = std::env::temp_dir().join(format!(
            "test_write_error_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
        ];

        let result = write_to_file(items.into_iter(), &path);
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_zst_file() {
        let path = std::env::temp_dir().join(format!(
            "test_write_stream_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        write_to_zst_file(ok_iter(["apple", "banana", "cherry"]), &path).unwrap();

        // Read and decompress to verify
        let file = File::open(&path).unwrap();
        let mut decoder = zstd::Decoder::new(file).unwrap();
        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();
        assert_eq!(content, "apple\nbanana\ncherry\n");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_write_to_zst_file_empty() {
        let path = std::env::temp_dir().join(format!(
            "test_write_empty_{}.zst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        write_to_zst_file(ok_iter([]), &path).unwrap();

        // Read and decompress to verify
        let file = File::open(&path).unwrap();
        let mut decoder = zstd::Decoder::new(file).unwrap();
        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();
        assert!(content.is_empty());

        std::fs::remove_file(path).ok();
    }
}

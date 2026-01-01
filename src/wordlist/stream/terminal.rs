//! Terminal operations for WordStream.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::wordlist::UniqueStringSet;

/// Collects an iterator of `io::Result<String>` into a `UniqueStringSet`.
///
/// # Errors
///
/// Returns an error if any item in the iterator is an error.
pub fn collect_to_set<I>(iter: I) -> io::Result<UniqueStringSet>
where
    I: Iterator<Item = io::Result<String>>,
{
    let words: Result<Vec<String>, io::Error> = iter.collect();
    Ok(UniqueStringSet::from_iter(words?))
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
    I: Iterator<Item = io::Result<String>>,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for item in iter {
        let s = item?;
        writeln!(writer, "{}", s)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_iter<I: IntoIterator<Item = &'static str>>(
        items: I,
    ) -> impl Iterator<Item = io::Result<String>> {
        items.into_iter().map(|s| Ok(s.to_string()))
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
        let items: Vec<io::Result<String>> = vec![
            Ok("apple".to_string()),
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

        let items: Vec<io::Result<String>> = vec![
            Ok("apple".to_string()),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
        ];

        let result = write_to_file(items.into_iter(), &path);
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }
}

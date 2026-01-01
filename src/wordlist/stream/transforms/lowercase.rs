//! Lowercase transform for WordStream.

use std::io;

/// An iterator that converts all strings to lowercase.
///
/// Preserves sort order in case-fold ordering because the primary sort key
/// (lowercase form) remains unchanged.
pub struct LowercaseStream<I> {
    inner: I,
}

impl<I> LowercaseStream<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for LowercaseStream<I>
where
    I: Iterator<Item = io::Result<String>>,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            Ok(s) => Some(Ok(s.to_lowercase())),
            Err(e) => Some(Err(e)),
        }
    }
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
    fn test_lowercase_uppercase() {
        let stream = LowercaseStream::new(ok_iter(["HELLO", "WORLD"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["hello", "world"]);
    }

    #[test]
    fn test_lowercase_mixed_case() {
        let stream = LowercaseStream::new(ok_iter(["HeLLo", "WoRLd"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["hello", "world"]);
    }

    #[test]
    fn test_lowercase_already_lowercase() {
        let stream = LowercaseStream::new(ok_iter(["hello", "world"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["hello", "world"]);
    }

    #[test]
    fn test_lowercase_german_umlauts() {
        let stream = LowercaseStream::new(ok_iter(["ÄRGER", "Ärger", "ärger"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert_eq!(collected, vec!["ärger", "ärger", "ärger"]);
    }

    #[test]
    fn test_lowercase_preserves_errors() {
        let items: Vec<io::Result<String>> = vec![
            Ok("HELLO".to_string()),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok("WORLD".to_string()),
        ];
        let stream = LowercaseStream::new(items.into_iter());
        let results: Vec<_> = stream.collect();

        assert_eq!(results[0].as_ref().unwrap(), "hello");
        assert!(results[1].is_err());
        assert_eq!(results[2].as_ref().unwrap(), "world");
    }

    #[test]
    fn test_lowercase_empty() {
        let stream = LowercaseStream::new(ok_iter([]));
        let collected: Vec<String> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }
}

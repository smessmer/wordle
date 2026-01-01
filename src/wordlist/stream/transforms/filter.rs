//! Filter transform for WordStream.

use std::io;

/// An iterator that filters items based on a predicate.
///
/// Only applies the predicate to `Ok` values; errors pass through unchanged.
pub struct FilterStream<I, F> {
    inner: I,
    predicate: F,
}

impl<I, F> FilterStream<I, F> {
    pub fn new(inner: I, predicate: F) -> Self {
        Self { inner, predicate }
    }
}

impl<I, F> Iterator for FilterStream<I, F>
where
    I: Iterator<Item = io::Result<String>>,
    F: FnMut(&str) -> bool,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(s) => {
                    if (self.predicate)(&s) {
                        return Some(Ok(s));
                    }
                    // Filtered out, continue to next
                }
                Err(e) => return Some(Err(e)),
            }
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
    fn test_filter_by_length() {
        let stream = FilterStream::new(ok_iter(["a", "bb", "ccc", "dddd"]), |s: &str| s.len() == 3);
        let collected: Vec<String> = stream.map(|r: io::Result<String>| r.unwrap()).collect();
        assert_eq!(collected, vec!["ccc"]);
    }

    #[test]
    fn test_filter_by_prefix() {
        let stream = FilterStream::new(
            ok_iter(["apple", "apricot", "banana", "avocado"]),
            |s: &str| s.starts_with('a'),
        );
        let collected: Vec<String> = stream.map(|r: io::Result<String>| r.unwrap()).collect();
        assert_eq!(collected, vec!["apple", "apricot", "avocado"]);
    }

    #[test]
    fn test_filter_all() {
        let stream = FilterStream::new(ok_iter(["hello", "world"]), |_: &str| false);
        let collected: Vec<String> = stream.map(|r: io::Result<String>| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_filter_none() {
        let stream = FilterStream::new(ok_iter(["hello", "world"]), |_: &str| true);
        let collected: Vec<String> = stream.map(|r: io::Result<String>| r.unwrap()).collect();
        assert_eq!(collected, vec!["hello", "world"]);
    }

    #[test]
    fn test_filter_preserves_errors() {
        let items: Vec<io::Result<String>> = vec![
            Ok("apple".to_string()),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok("banana".to_string()),
        ];
        let stream = FilterStream::new(items.into_iter(), |_: &str| true);
        let results: Vec<_> = stream.collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }

    #[test]
    fn test_filter_empty() {
        let stream = FilterStream::new(ok_iter([]), |_: &str| true);
        let collected: Vec<String> = stream.map(|r: io::Result<String>| r.unwrap()).collect();
        assert!(collected.is_empty());
    }
}

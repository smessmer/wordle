//! Deduplication transform for WordStream.

use std::io;

use crate::Word;

/// An iterator that removes consecutive duplicates using case-insensitive equality.
///
/// Two strings are considered equal if their lowercase forms are identical.
/// Since the stream is sorted in case-fold order, this effectively removes
/// all case variations (e.g., "apple", "Apple", and "APPLE" are all considered equal).
pub struct DedupStream<I> {
    inner: I,
    previous_lower: Option<String>,
}

impl<I> DedupStream<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            previous_lower: None,
        }
    }
}

impl<I> Iterator for DedupStream<I>
where
    I: Iterator<Item = io::Result<Word>>,
{
    type Item = io::Result<Word>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(w) => {
                    let s_lower = w.0.to_lowercase();
                    let is_dup = self
                        .previous_lower
                        .as_ref()
                        .is_some_and(|prev| *prev == s_lower);

                    if is_dup {
                        // Skip duplicate, continue to next
                        continue;
                    }

                    self.previous_lower = Some(s_lower);
                    return Some(Ok(w));
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
    ) -> impl Iterator<Item = io::Result<Word>> {
        items.into_iter().map(|s| Ok(Word(s.to_string())))
    }

    #[test]
    fn test_dedup_exact_duplicates() {
        let stream = DedupStream::new(ok_iter(["apple", "apple", "banana", "banana", "cherry"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_dedup_case_fold_duplicates() {
        // In case-fold order: apple < Apple < APPLE, but they're equal for dedup
        let stream = DedupStream::new(ok_iter(["apple", "Apple", "APPLE", "banana"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        // Keeps the first occurrence
        assert_eq!(collected, vec!["apple", "banana"]);
    }

    #[test]
    fn test_dedup_no_duplicates() {
        let stream = DedupStream::new(ok_iter(["apple", "banana", "cherry"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_dedup_all_same() {
        let stream = DedupStream::new(ok_iter(["apple", "apple", "apple"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple"]);
    }

    #[test]
    fn test_dedup_german_umlauts() {
        let stream = DedupStream::new(ok_iter(["ärger", "Ärger", "ÄRGER", "bär"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["ärger", "bär"]);
    }

    #[test]
    fn test_dedup_preserves_errors() {
        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok(Word("apple".to_string())), // This is still considered a dup of the first apple
            Ok(Word("banana".to_string())), // Different word, not a dup
        ];
        let stream = DedupStream::new(items.into_iter());
        let results: Vec<_> = stream.collect();

        // Error passes through, but dedup state is preserved across errors
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok()); // apple
        assert!(results[1].is_err()); // error
        assert!(results[2].is_ok()); // banana (second apple was skipped as dup)
    }

    #[test]
    fn test_dedup_empty() {
        let stream = DedupStream::new(ok_iter([]));
        let collected: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_dedup_single() {
        let stream = DedupStream::new(ok_iter(["hello"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["hello"]);
    }
}

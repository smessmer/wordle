//! Filter transform that warns about non-alphabetic words.

use std::io;

use crate::wordlist::Word;

use super::FilterStream;

/// Creates a filter that removes words with non-alphabetic characters.
/// Outputs a warning to stderr for each filtered word.
pub fn filter_non_alphabetic<I>(iter: I) -> FilterStream<I, impl FnMut(&str) -> bool>
where
    I: Iterator<Item = io::Result<Word>>,
{
    FilterStream::new(iter, |w: &str| {
        if w.chars().all(|c| c.is_alphabetic()) {
            true
        } else {
            eprintln!("Warning: filtering non-alphabetic word: {}", w);
            false
        }
    })
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
    fn test_keeps_alphabetic_words() {
        let stream = filter_non_alphabetic(ok_iter(["apple", "banana", "cherry"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_filters_words_with_digits() {
        let stream = filter_non_alphabetic(ok_iter(["apple", "test123", "banana"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana"]);
    }

    #[test]
    fn test_filters_words_with_punctuation() {
        let stream = filter_non_alphabetic(ok_iter(["hello", "world!", "test"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["hello", "test"]);
    }

    #[test]
    fn test_filters_words_with_spaces() {
        let stream = filter_non_alphabetic(ok_iter(["hello", "hello world", "test"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["hello", "test"]);
    }

    #[test]
    fn test_filters_words_with_hyphens() {
        let stream = filter_non_alphabetic(ok_iter(["apple", "self-aware", "banana"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["apple", "banana"]);
    }

    #[test]
    fn test_keeps_unicode_alphabetic() {
        let stream = filter_non_alphabetic(ok_iter(["café", "naïve", "über"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["café", "naïve", "über"]);
    }

    #[test]
    fn test_keeps_german_umlauts() {
        let stream = filter_non_alphabetic(ok_iter(["Äpfel", "Größe", "schön"]));
        let collected: Vec<String> = stream.map(|r| r.unwrap().0).collect();
        assert_eq!(collected, vec!["Äpfel", "Größe", "schön"]);
    }

    #[test]
    fn test_empty_stream() {
        let stream = filter_non_alphabetic(ok_iter([]));
        let collected: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_all_filtered() {
        let stream = filter_non_alphabetic(ok_iter(["123", "test!", "hello-world"]));
        let collected: Vec<Word> = stream.map(|r| r.unwrap()).collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_preserves_errors() {
        let items: Vec<io::Result<Word>> = vec![
            Ok(Word("apple".to_string())),
            Err(io::Error::new(io::ErrorKind::Other, "test error")),
            Ok(Word("banana".to_string())),
        ];
        let stream = filter_non_alphabetic(items.into_iter());
        let results: Vec<_> = stream.collect();

        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }
}

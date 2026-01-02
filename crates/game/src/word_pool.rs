use crate::letter::Word;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::io;

/// A pool of valid words for the game
#[derive(Debug, Clone)]
pub struct WordPool {
    words: Vec<Word>,
    word_set: HashSet<Word>,
}

impl WordPool {
    /// Create from iterator of words
    pub fn from_words(words: impl IntoIterator<Item = Word>) -> Self {
        let words: Vec<Word> = words.into_iter().collect();
        let word_set: HashSet<Word> = words.iter().cloned().collect();
        Self { words, word_set }
    }

    /// Create from string iterator (convenience)
    pub fn from_strings(strings: impl IntoIterator<Item = String>) -> Self {
        let words: Vec<Word> = strings
            .into_iter()
            .filter_map(|s| Word::parse(&s))
            .collect();
        Self::from_words(words)
    }

    /// Check if a word is valid
    pub fn contains(&self, word: &Word) -> bool {
        self.word_set.contains(word)
    }

    /// Get a random word
    pub fn random(&self) -> &Word {
        self.words
            .choose(&mut rand::thread_rng())
            .expect("WordPool should not be empty")
    }

    /// Number of words in the pool
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Is the pool empty
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// Load the embedded German wordlist
pub fn load_german_wordlist() -> io::Result<WordPool> {
    use wordle_wordlists_processing::stream::from_txt_zstd;

    let stream = from_txt_zstd(crate::wordlists::DE)?;
    let mut words = Vec::new();

    for word_result in stream {
        let word_str = word_result?.0;
        if let Some(word) = Word::parse(&word_str) {
            words.push(word);
        }
    }

    Ok(WordPool::from_words(words))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_pool_from_strings() {
        let pool = WordPool::from_strings(vec![
            "hello".to_string(),
            "world".to_string(),
            "short".to_string(),
        ]);

        assert_eq!(pool.len(), 3);
        assert!(pool.contains(&Word::parse("hello").unwrap()));
        assert!(pool.contains(&Word::parse("world").unwrap()));
        assert!(!pool.contains(&Word::parse("other").unwrap()));
    }

    #[test]
    fn test_word_pool_filters_invalid() {
        let pool = WordPool::from_strings(vec![
            "hello".to_string(),
            "hi".to_string(),       // too short
            "toolong".to_string(),  // too long
            "12345".to_string(),    // not alphabetic
        ]);

        assert_eq!(pool.len(), 1);
        assert!(pool.contains(&Word::parse("hello").unwrap()));
    }

    #[test]
    fn test_random_word() {
        let pool = WordPool::from_strings(vec![
            "hello".to_string(),
            "world".to_string(),
        ]);

        let random = pool.random();
        assert!(pool.contains(random));
    }
}

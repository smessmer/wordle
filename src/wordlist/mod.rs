mod error;

pub use error::{Result, UniqueStringSetError};

use sorted_vec::SortedSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub mod stream;

/// A sorted, unique collection of strings.
///
/// Backed by `SortedSet<String>` for O(log n) lookups and O(n+m) merges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueStringSet {
    inner: SortedSet<String>,
}

impl UniqueStringSet {
    /// Creates an empty `UniqueStringSet`.
    pub fn new() -> Self {
        Self {
            inner: SortedSet::new(),
        }
    }

    /// Creates a `UniqueStringSet` from an iterator of items convertible to strings.
    pub fn from_iter<I, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            inner: iter.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Returns `true` if the set contains the given string.
    pub fn contains(&self, s: &str) -> bool {
        self.inner.binary_search_by(|probe| probe.as_str().cmp(s)).is_ok()
    }

    /// Returns the number of strings in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Inserts a string into the set.
    ///
    /// Returns `true` if the string was newly inserted, `false` if it already existed.
    pub fn insert(&mut self, s: impl Into<String>) -> bool {
        let s = s.into();
        match self.inner.find_or_insert(s) {
            sorted_vec::FindOrInsert::Found(_) => false,
            sorted_vec::FindOrInsert::Inserted(_) => true,
        }
    }

    /// Returns an iterator over the strings as `&str`.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(|s| s.as_str())
    }

    /// Loads strings from a file, one per line.
    ///
    /// Empty lines are skipped. Lines are trimmed of whitespace.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let words: std::result::Result<Vec<String>, std::io::Error> = reader
            .lines()
            .filter_map(|line| {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => return Some(Err(e)),
                };
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Ok(trimmed.to_string()))
                }
            })
            .collect();

        Ok(Self {
            inner: words?.into_iter().collect(),
        })
    }

    /// Saves strings to a file, one per line.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = File::create(path)?;
        for s in self.inner.iter() {
            writeln!(file, "{}", s)?;
        }
        Ok(())
    }

    /// Merges this set with another, consuming both.
    ///
    /// Returns a new set containing all strings from both sets.
    pub fn merge_with(self, other: Self) -> Self {
        // TODO A proper merge sort with two iterators could be faster
        Self {
            inner: self.inner.into_iter().chain(other.inner).collect(),
        }
    }

    /// Filters the set using a predicate, returning a new set.
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool,
    {
        Self {
            inner: self
                .inner
                .iter()
                .filter(|s| predicate(s.as_str()))
                .cloned()
                .collect(),
        }
    }

    /// Filters to keep only strings that are purely alphabetic.
    pub fn filter_alphabetic(&self) -> Self {
        self.filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphabetic()))
    }

    /// Converts all strings to lowercase in-place.
    pub fn to_lowercase(&mut self) {
        self.inner = self.inner.iter().map(|s| s.to_lowercase()).collect();
    }
}

impl Default for UniqueStringSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for UniqueStringSet {
    type Item = String;
    type IntoIter = <SortedSet<String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a UniqueStringSet {
    type Item = &'a str;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, String>, fn(&'a String) -> &'a str>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().map(|s| s.as_str())
    }
}

impl std::iter::FromIterator<String> for UniqueStringSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl Extend<String> for UniqueStringSet {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        for s in iter {
            self.insert(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn test_new_creates_empty_set() {
            let set = UniqueStringSet::new();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_default_creates_empty_set() {
            let set: UniqueStringSet = Default::default();
            assert!(set.is_empty());
        }

        #[test]
        fn test_from_iter_with_vec() {
            let set = UniqueStringSet::from_iter(vec!["hello", "world"]);
            assert_eq!(set.len(), 2);
            assert!(set.contains("hello"));
            assert!(set.contains("world"));
        }

        #[test]
        fn test_from_iter_with_strings() {
            let set = UniqueStringSet::from_iter(vec!["a".to_string(), "b".to_string()]);
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_from_iter_deduplicates() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "a", "c", "b"]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_from_iter_maintains_sorted_order() {
            let set = UniqueStringSet::from_iter(vec!["cherry", "apple", "banana"]);
            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["apple", "banana", "cherry"]);
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn test_len() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_is_empty() {
            let empty = UniqueStringSet::new();
            let non_empty = UniqueStringSet::from_iter(vec!["a"]);

            assert!(empty.is_empty());
            assert!(!non_empty.is_empty());
        }

        #[test]
        fn test_contains() {
            let set = UniqueStringSet::from_iter(vec!["hello", "world"]);

            assert!(set.contains("hello"));
            assert!(set.contains("world"));
            assert!(!set.contains("foo"));
            assert!(!set.contains(""));
        }
    }

    mod iterator{
        use super::*;

        #[test]
        fn test_iter_returns_str() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["a", "b", "c"]);
        }

        #[test]
        fn test_into_iterator_owned() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            let collected: Vec<String> = set.into_iter().collect();
            assert_eq!(collected, vec!["a", "b", "c"]);
        }

        #[test]
        fn test_into_iterator_ref() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            let mut count = 0;
            for _s in &set {
                count += 1;
            }
            assert_eq!(count, 3);
            // set still usable
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_for_loop_yields_str() {
            let set = UniqueStringSet::from_iter(vec!["hello"]);
            for s in &set {
                // s should be &str
                let _: &str = s;
            }
        }
    }

    mod file_io {
        use super::*;

        #[test]
        fn test_load_from_file() {
            let dir = std::env::temp_dir();
            let path = dir.join("test_unique_string_set_load.txt");

            {
                let mut file = std::fs::File::create(&path).unwrap();
                writeln!(file, "apple").unwrap();
                writeln!(file, "banana").unwrap();
                writeln!(file, "").unwrap();
                writeln!(file, "  cherry  ").unwrap();
            }

            let set = UniqueStringSet::load_from_file(&path).unwrap();

            assert_eq!(set.len(), 3);
            assert!(set.contains("apple"));
            assert!(set.contains("banana"));
            assert!(set.contains("cherry"));

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn test_load_from_file_not_found() {
            let result = UniqueStringSet::load_from_file("/nonexistent/path.txt");
            assert!(result.is_err());
        }

        #[test]
        fn test_save_to_file() {
            let set = UniqueStringSet::from_iter(vec!["one", "two", "three"]);
            let path = std::env::temp_dir().join("test_unique_string_set_save.txt");

            set.save_to_file(&path).unwrap();

            let content = std::fs::read_to_string(&path).unwrap();
            assert!(content.contains("one"));
            assert!(content.contains("two"));
            assert!(content.contains("three"));

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn test_save_and_load_roundtrip() {
            let original = UniqueStringSet::from_iter(vec!["alpha", "beta", "gamma"]);
            let path = std::env::temp_dir().join("test_roundtrip.txt");

            original.save_to_file(&path).unwrap();
            let loaded = UniqueStringSet::load_from_file(&path).unwrap();

            assert_eq!(original, loaded);

            std::fs::remove_file(path).ok();
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn test_merge_with() {
            let set1 = UniqueStringSet::from_iter(vec!["a", "b"]);
            let set2 = UniqueStringSet::from_iter(vec!["c", "d"]);

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 4);
            assert!(merged.contains("a"));
            assert!(merged.contains("b"));
            assert!(merged.contains("c"));
            assert!(merged.contains("d"));
        }

        #[test]
        fn test_merge_with_overlapping() {
            let set1 = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            let set2 = UniqueStringSet::from_iter(vec!["b", "c", "d"]);

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 4);
            let collected: Vec<&str> = merged.iter().collect();
            assert_eq!(collected, vec!["a", "b", "c", "d"]);
        }

        #[test]
        fn test_merge_with_empty() {
            let set1 = UniqueStringSet::from_iter(vec!["a", "b"]);
            let set2 = UniqueStringSet::new();

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 2);
        }

        #[test]
        fn test_merge_maintains_sorted_order() {
            let set1 = UniqueStringSet::from_iter(vec!["zebra", "apple"]);
            let set2 = UniqueStringSet::from_iter(vec!["mango", "banana"]);

            let merged = set1.merge_with(set2);
            let collected: Vec<&str> = merged.iter().collect();

            assert_eq!(collected, vec!["apple", "banana", "mango", "zebra"]);
        }
    }

    mod filter {
        use super::*;

        #[test]
        fn test_filter_by_length() {
            let set = UniqueStringSet::from_iter(vec!["a", "bb", "ccc", "dddd"]);
            let filtered = set.filter(|s| s.len() == 3);

            assert_eq!(filtered.len(), 1);
            assert!(filtered.contains("ccc"));
        }

        #[test]
        fn test_filter_custom_predicate() {
            let set = UniqueStringSet::from_iter(vec!["apple", "apricot", "banana", "avocado"]);
            let filtered = set.filter(|s| s.starts_with('a'));

            assert_eq!(filtered.len(), 3);
            assert!(filtered.contains("apple"));
            assert!(filtered.contains("apricot"));
            assert!(filtered.contains("avocado"));
            assert!(!filtered.contains("banana"));
        }

        #[test]
        fn test_filter_returns_empty() {
            let set = UniqueStringSet::from_iter(vec!["hello", "world"]);
            let filtered = set.filter(|s| s.len() > 100);

            assert!(filtered.is_empty());
        }

        #[test]
        fn test_filter_preserves_order() {
            let set = UniqueStringSet::from_iter(vec!["aaa", "bbb", "ccc", "ddd"]);
            let filtered = set.filter(|s| s != "bbb");

            let collected: Vec<&str> = filtered.iter().collect();
            assert_eq!(collected, vec!["aaa", "ccc", "ddd"]);
        }

        #[test]
        fn test_filter_alphabetic() {
            let set = UniqueStringSet::from_iter(vec![
                "hello",
                "world123",
                "test",
                "foo-bar",
                "valid",
                "",
                "123",
            ]);

            let filtered = set.filter_alphabetic();

            assert_eq!(filtered.len(), 3);
            assert!(filtered.contains("hello"));
            assert!(filtered.contains("test"));
            assert!(filtered.contains("valid"));
            assert!(!filtered.contains("world123"));
            assert!(!filtered.contains("foo-bar"));
        }

        #[test]
        fn test_filter_alphabetic_with_unicode() {
            let set = UniqueStringSet::from_iter(vec!["cafe", "hello", "test1"]);
            let filtered = set.filter_alphabetic();

            assert_eq!(filtered.len(), 2);
            assert!(filtered.contains("cafe"));
            assert!(filtered.contains("hello"));
        }
    }

    mod transformations {
        use super::*;

        #[test]
        fn test_to_lowercase() {
            let mut set = UniqueStringSet::from_iter(vec!["HELLO", "World", "rust"]);
            set.to_lowercase();

            assert!(set.contains("hello"));
            assert!(set.contains("world"));
            assert!(set.contains("rust"));
            assert!(!set.contains("HELLO"));
            assert!(!set.contains("World"));
        }

        #[test]
        fn test_to_lowercase_deduplicates() {
            let mut set = UniqueStringSet::from_iter(vec!["Hello", "HELLO", "hello"]);
            set.to_lowercase();

            assert_eq!(set.len(), 1);
            assert!(set.contains("hello"));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_set_operations() {
            let set = UniqueStringSet::new();

            assert!(!set.contains("anything"));
            assert_eq!(set.iter().count(), 0);

            let filtered = set.filter(|_| true);
            assert!(filtered.is_empty());
        }

        #[test]
        fn test_single_element() {
            let set = UniqueStringSet::from_iter(vec!["only"]);

            assert_eq!(set.len(), 1);
            assert!(set.contains("only"));

            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["only"]);
        }

        #[test]
        fn test_clone() {
            let set = UniqueStringSet::from_iter(vec!["a", "b", "c"]);
            let cloned = set.clone();

            assert_eq!(set, cloned);
        }

        #[test]
        fn test_equality() {
            let set1 = UniqueStringSet::from_iter(vec!["a", "b"]);
            let set2 = UniqueStringSet::from_iter(vec!["b", "a"]); // different order, same content
            let set3 = UniqueStringSet::from_iter(vec!["a", "c"]);

            assert_eq!(set1, set2);
            assert_ne!(set1, set3);
        }

    }
}
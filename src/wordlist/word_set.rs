//! A sorted, unique collection of words.

use sorted_vec::SortedSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use super::error::Result;

/// A sorted, unique collection of strings.
///
/// Backed by `SortedSet<String>` for O(log n) lookups and O(n+m) merges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSet {
    inner: SortedSet<String>,
}

impl WordSet {
    /// Creates an empty `WordSet`.
    pub fn new() -> Self {
        Self {
            inner: SortedSet::new(),
        }
    }

    /// Creates a `WordSet` from an iterator of items convertible to strings.
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

impl Default for WordSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for WordSet {
    type Item = String;
    type IntoIter = <SortedSet<String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a WordSet {
    type Item = &'a str;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, String>, fn(&'a String) -> &'a str>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().map(|s| s.as_str())
    }
}

impl std::iter::FromIterator<String> for WordSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl Extend<String> for WordSet {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        for s in iter {
            self.insert(s);
        }
    }
}

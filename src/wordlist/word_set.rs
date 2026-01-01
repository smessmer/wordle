//! A sorted, unique collection of words.

use sorted_vec::SortedSet;

/// A sorted, unique collection of strings.
///
/// Backed by `SortedSet<String>` for O(log n) lookups.
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

    /// Returns `true` if the set contains the given string.
    pub fn contains(&self, s: &str) -> bool {
        self.inner
            .binary_search_by(|probe| probe.as_str().cmp(s))
            .is_ok()
    }

    /// Returns the number of strings in the set.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
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

impl std::iter::FromIterator<String> for WordSet {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

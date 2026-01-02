//! Case-fold ordering for string comparison.
//!
//! Provides a deterministic ordering where:
//! - Primary sort key: lowercase form of characters
//! - Secondary sort key: original case (lowercase < uppercase)
//!
//! This ensures `"apple" < "Apple" < "APPLE" < "banana"`.

use std::cmp::Ordering;

/// Compare two chars using case-fold ordering.
///
/// Compares the full lowercase sequence first (handles multi-char expansions
/// like 'İ' → "i\u{307}"), then uses case as tiebreaker.
///
/// Result: `'a' < 'A' < 'b' < 'B'`
fn char_cmp(a: char, b: char) -> Ordering {
    match a.to_lowercase().cmp(b.to_lowercase()) {
        Ordering::Equal => {
            // Tiebreaker: lowercase < uppercase (false < true)
            a.is_uppercase().cmp(&b.is_uppercase())
        }
        other => other,
    }
}

/// Compare two strings using case-fold ordering (char by char).
///
/// # Examples
///
/// ```
/// use std::cmp::Ordering;
/// # use wordle::wordlist::ordering::case_fold_cmp;
///
/// assert_eq!(case_fold_cmp("apple", "Apple"), Ordering::Less);
/// assert_eq!(case_fold_cmp("Apple", "APPLE"), Ordering::Less);
/// assert_eq!(case_fold_cmp("apple", "banana"), Ordering::Less);
/// assert_eq!(case_fold_cmp("ärger", "Ärger"), Ordering::Less);
/// ```
pub fn case_fold_cmp(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars();
    let mut b_chars = b.chars();

    loop {
        match (a_chars.next(), b_chars.next()) {
            (Some(ac), Some(bc)) => {
                let cmp = char_cmp(ac, bc);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_case_ordering() {
        assert_eq!(case_fold_cmp("apple", "banana"), Ordering::Less);
        assert_eq!(case_fold_cmp("banana", "apple"), Ordering::Greater);
        assert_eq!(case_fold_cmp("apple", "apple"), Ordering::Equal);
    }

    #[test]
    fn test_case_insensitive_primary() {
        // Same word, different cases should be grouped together
        assert_eq!(case_fold_cmp("apple", "banana"), Ordering::Less);
        assert_eq!(case_fold_cmp("APPLE", "banana"), Ordering::Less);
        assert_eq!(case_fold_cmp("Apple", "banana"), Ordering::Less);
    }

    #[test]
    fn test_lowercase_before_uppercase() {
        assert_eq!(case_fold_cmp("apple", "Apple"), Ordering::Less);
        assert_eq!(case_fold_cmp("Apple", "APPLE"), Ordering::Less);
        assert_eq!(case_fold_cmp("apple", "APPLE"), Ordering::Less);
    }

    #[test]
    fn test_mixed_case_ordering() {
        // 'a' < 'A' for the first differing char
        assert_eq!(case_fold_cmp("aA", "Aa"), Ordering::Less);
        assert_eq!(case_fold_cmp("aB", "Ab"), Ordering::Less);
    }

    #[test]
    fn test_german_umlauts() {
        // lowercase before uppercase for same word
        assert_eq!(case_fold_cmp("ärger", "Ärger"), Ordering::Less);
        assert_eq!(case_fold_cmp("Ärger", "ÄRGER"), Ordering::Less);
        // In Unicode order, 'ä' (U+00E4) > 'b' (U+0062), so ärger > bär
        assert_eq!(case_fold_cmp("ärger", "bär"), Ordering::Greater);
        assert_eq!(case_fold_cmp("bär", "ärger"), Ordering::Less);
    }

    #[test]
    fn test_prefix_ordering() {
        // Shorter string comes before longer when one is prefix of other
        assert_eq!(case_fold_cmp("app", "apple"), Ordering::Less);
        assert_eq!(case_fold_cmp("apple", "app"), Ordering::Greater);
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(case_fold_cmp("", ""), Ordering::Equal);
        assert_eq!(case_fold_cmp("", "a"), Ordering::Less);
        assert_eq!(case_fold_cmp("a", ""), Ordering::Greater);
    }

    #[test]
    fn test_char_cmp_basic() {
        assert_eq!(char_cmp('a', 'b'), Ordering::Less);
        assert_eq!(char_cmp('a', 'A'), Ordering::Less);
        assert_eq!(char_cmp('A', 'a'), Ordering::Greater);
        assert_eq!(char_cmp('a', 'a'), Ordering::Equal);
    }

    #[test]
    fn test_multi_char_lowercase() {
        // 'İ' (U+0130, Turkish capital I with dot) lowercases to "i\u{0307}" (2 chars)
        // 'İ'.to_lowercase() yields ['i', '\u{0307}']
        assert_eq!(char_cmp('İ', 'i'), Ordering::Greater); // 'i' + combining mark > 'i'
        assert_eq!(char_cmp('İ', 'j'), Ordering::Less); // 'i...' < 'j'

        // 'ẞ' (U+1E9E, German capital sharp s) lowercases to 'ß' (1 char in Rust)
        assert_eq!(char_cmp('ẞ', 'ß'), Ordering::Greater); // uppercase > lowercase
        assert_eq!(char_cmp('ß', 'ẞ'), Ordering::Less);

        // String comparison with İ
        // "İstanbul" lowercases to "i\u{0307}stanbul"
        assert_eq!(case_fold_cmp("İstanbul", "istanbul"), Ordering::Greater);
        assert_eq!(case_fold_cmp("istanbul", "İstanbul"), Ordering::Less);

        // İ and I are different (İ has combining dot after lowercasing)
        assert_eq!(case_fold_cmp("İ", "I"), Ordering::Greater);
        assert_eq!(case_fold_cmp("I", "İ"), Ordering::Less);
    }
}

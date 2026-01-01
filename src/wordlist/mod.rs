mod error;
mod word_set;

pub use error::{Result, WordSetError};
pub use word_set::WordSet;

pub mod stream;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    mod constructor {
        use super::*;

        #[test]
        fn test_new_creates_empty_set() {
            let set = WordSet::new();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_default_creates_empty_set() {
            let set: WordSet = Default::default();
            assert!(set.is_empty());
        }

        #[test]
        fn test_from_iter_with_vec() {
            let set = WordSet::from_iter(vec!["hello", "world"]);
            assert_eq!(set.len(), 2);
            assert!(set.contains("hello"));
            assert!(set.contains("world"));
        }

        #[test]
        fn test_from_iter_with_strings() {
            let set = WordSet::from_iter(vec!["a".to_string(), "b".to_string()]);
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_from_iter_deduplicates() {
            let set = WordSet::from_iter(vec!["a", "b", "a", "c", "b"]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_from_iter_maintains_sorted_order() {
            let set = WordSet::from_iter(vec!["cherry", "apple", "banana"]);
            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["apple", "banana", "cherry"]);
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn test_len() {
            let set = WordSet::from_iter(vec!["a", "b", "c"]);
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_is_empty() {
            let empty = WordSet::new();
            let non_empty = WordSet::from_iter(vec!["a"]);

            assert!(empty.is_empty());
            assert!(!non_empty.is_empty());
        }

        #[test]
        fn test_contains() {
            let set = WordSet::from_iter(vec!["hello", "world"]);

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
            let set = WordSet::from_iter(vec!["a", "b", "c"]);
            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["a", "b", "c"]);
        }

        #[test]
        fn test_into_iterator_owned() {
            let set = WordSet::from_iter(vec!["a", "b", "c"]);
            let collected: Vec<String> = set.into_iter().collect();
            assert_eq!(collected, vec!["a", "b", "c"]);
        }

        #[test]
        fn test_into_iterator_ref() {
            let set = WordSet::from_iter(vec!["a", "b", "c"]);
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
            let set = WordSet::from_iter(vec!["hello"]);
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

            let set = WordSet::load_from_file(&path).unwrap();

            assert_eq!(set.len(), 3);
            assert!(set.contains("apple"));
            assert!(set.contains("banana"));
            assert!(set.contains("cherry"));

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn test_load_from_file_not_found() {
            let result = WordSet::load_from_file("/nonexistent/path.txt");
            assert!(result.is_err());
        }

        #[test]
        fn test_save_to_file() {
            let set = WordSet::from_iter(vec!["one", "two", "three"]);
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
            let original = WordSet::from_iter(vec!["alpha", "beta", "gamma"]);
            let path = std::env::temp_dir().join("test_roundtrip.txt");

            original.save_to_file(&path).unwrap();
            let loaded = WordSet::load_from_file(&path).unwrap();

            assert_eq!(original, loaded);

            std::fs::remove_file(path).ok();
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn test_merge_with() {
            let set1 = WordSet::from_iter(vec!["a", "b"]);
            let set2 = WordSet::from_iter(vec!["c", "d"]);

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 4);
            assert!(merged.contains("a"));
            assert!(merged.contains("b"));
            assert!(merged.contains("c"));
            assert!(merged.contains("d"));
        }

        #[test]
        fn test_merge_with_overlapping() {
            let set1 = WordSet::from_iter(vec!["a", "b", "c"]);
            let set2 = WordSet::from_iter(vec!["b", "c", "d"]);

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 4);
            let collected: Vec<&str> = merged.iter().collect();
            assert_eq!(collected, vec!["a", "b", "c", "d"]);
        }

        #[test]
        fn test_merge_with_empty() {
            let set1 = WordSet::from_iter(vec!["a", "b"]);
            let set2 = WordSet::new();

            let merged = set1.merge_with(set2);

            assert_eq!(merged.len(), 2);
        }

        #[test]
        fn test_merge_maintains_sorted_order() {
            let set1 = WordSet::from_iter(vec!["zebra", "apple"]);
            let set2 = WordSet::from_iter(vec!["mango", "banana"]);

            let merged = set1.merge_with(set2);
            let collected: Vec<&str> = merged.iter().collect();

            assert_eq!(collected, vec!["apple", "banana", "mango", "zebra"]);
        }
    }

    mod filter {
        use super::*;

        #[test]
        fn test_filter_by_length() {
            let set = WordSet::from_iter(vec!["a", "bb", "ccc", "dddd"]);
            let filtered = set.filter(|s| s.len() == 3);

            assert_eq!(filtered.len(), 1);
            assert!(filtered.contains("ccc"));
        }

        #[test]
        fn test_filter_custom_predicate() {
            let set = WordSet::from_iter(vec!["apple", "apricot", "banana", "avocado"]);
            let filtered = set.filter(|s| s.starts_with('a'));

            assert_eq!(filtered.len(), 3);
            assert!(filtered.contains("apple"));
            assert!(filtered.contains("apricot"));
            assert!(filtered.contains("avocado"));
            assert!(!filtered.contains("banana"));
        }

        #[test]
        fn test_filter_returns_empty() {
            let set = WordSet::from_iter(vec!["hello", "world"]);
            let filtered = set.filter(|s| s.len() > 100);

            assert!(filtered.is_empty());
        }

        #[test]
        fn test_filter_preserves_order() {
            let set = WordSet::from_iter(vec!["aaa", "bbb", "ccc", "ddd"]);
            let filtered = set.filter(|s| s != "bbb");

            let collected: Vec<&str> = filtered.iter().collect();
            assert_eq!(collected, vec!["aaa", "ccc", "ddd"]);
        }

        #[test]
        fn test_filter_alphabetic() {
            let set = WordSet::from_iter(vec![
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
            let set = WordSet::from_iter(vec!["cafe", "hello", "test1"]);
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
            let mut set = WordSet::from_iter(vec!["HELLO", "World", "rust"]);
            set.to_lowercase();

            assert!(set.contains("hello"));
            assert!(set.contains("world"));
            assert!(set.contains("rust"));
            assert!(!set.contains("HELLO"));
            assert!(!set.contains("World"));
        }

        #[test]
        fn test_to_lowercase_deduplicates() {
            let mut set = WordSet::from_iter(vec!["Hello", "HELLO", "hello"]);
            set.to_lowercase();

            assert_eq!(set.len(), 1);
            assert!(set.contains("hello"));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_set_operations() {
            let set = WordSet::new();

            assert!(!set.contains("anything"));
            assert_eq!(set.iter().count(), 0);

            let filtered = set.filter(|_| true);
            assert!(filtered.is_empty());
        }

        #[test]
        fn test_single_element() {
            let set = WordSet::from_iter(vec!["only"]);

            assert_eq!(set.len(), 1);
            assert!(set.contains("only"));

            let collected: Vec<&str> = set.iter().collect();
            assert_eq!(collected, vec!["only"]);
        }

        #[test]
        fn test_clone() {
            let set = WordSet::from_iter(vec!["a", "b", "c"]);
            let cloned = set.clone();

            assert_eq!(set, cloned);
        }

        #[test]
        fn test_equality() {
            let set1 = WordSet::from_iter(vec!["a", "b"]);
            let set2 = WordSet::from_iter(vec!["b", "a"]); // different order, same content
            let set3 = WordSet::from_iter(vec!["a", "c"]);

            assert_eq!(set1, set2);
            assert_ne!(set1, set3);
        }

    }
}
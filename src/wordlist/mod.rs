mod word_set;

pub use word_set::WordSet;

pub mod stream;

#[cfg(test)]
mod tests {
    use super::*;

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
        fn test_collect_from_iter() {
            let set: WordSet = vec!["hello".to_string(), "world".to_string()]
                .into_iter()
                .collect();
            assert_eq!(set.len(), 2);
            assert!(set.contains("hello"));
            assert!(set.contains("world"));
        }

        #[test]
        fn test_collect_deduplicates() {
            let set: WordSet = vec!["a", "b", "a", "c", "b"]
                .into_iter()
                .map(String::from)
                .collect();
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_collect_maintains_sorted_order() {
            let set: WordSet = vec!["cherry", "apple", "banana"]
                .into_iter()
                .map(String::from)
                .collect();
            let collected: Vec<String> = set.into_iter().collect();
            assert_eq!(collected, vec!["apple", "banana", "cherry"]);
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn test_len() {
            let set: WordSet = vec!["a", "b", "c"]
                .into_iter()
                .map(String::from)
                .collect();
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_is_empty() {
            let empty = WordSet::new();
            let non_empty: WordSet = vec!["a".to_string()].into_iter().collect();

            assert!(empty.is_empty());
            assert!(!non_empty.is_empty());
        }

        #[test]
        fn test_contains() {
            let set: WordSet = vec!["hello", "world"]
                .into_iter()
                .map(String::from)
                .collect();

            assert!(set.contains("hello"));
            assert!(set.contains("world"));
            assert!(!set.contains("foo"));
            assert!(!set.contains(""));
        }
    }

    mod iterator {
        use super::*;

        #[test]
        fn test_into_iterator_owned() {
            let set: WordSet = vec!["a", "b", "c"]
                .into_iter()
                .map(String::from)
                .collect();
            let collected: Vec<String> = set.into_iter().collect();
            assert_eq!(collected, vec!["a", "b", "c"]);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_set_operations() {
            let set = WordSet::new();
            assert!(!set.contains("anything"));
        }

        #[test]
        fn test_single_element() {
            let set: WordSet = vec!["only".to_string()].into_iter().collect();

            assert_eq!(set.len(), 1);
            assert!(set.contains("only"));

            let collected: Vec<String> = set.into_iter().collect();
            assert_eq!(collected, vec!["only"]);
        }

        #[test]
        fn test_clone() {
            let set: WordSet = vec!["a", "b", "c"]
                .into_iter()
                .map(String::from)
                .collect();
            let cloned = set.clone();

            assert_eq!(set, cloned);
        }

        #[test]
        fn test_equality() {
            let set1: WordSet = vec!["a", "b"]
                .into_iter()
                .map(String::from)
                .collect();
            let set2: WordSet = vec!["b", "a"] // different order, same content
                .into_iter()
                .map(String::from)
                .collect();
            let set3: WordSet = vec!["a", "c"]
                .into_iter()
                .map(String::from)
                .collect();

            assert_eq!(set1, set2);
            assert_ne!(set1, set3);
        }
    }
}

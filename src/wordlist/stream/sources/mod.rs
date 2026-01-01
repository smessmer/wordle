//! Source iterators for WordStream.

mod sorted_file;
mod unsorted_file;

pub use sorted_file::{from_sorted_file, from_sorted_reader, from_sorted_zst_file, SortedLines};
pub use unsorted_file::{
    from_unsorted_file, from_unsorted_reader, from_unsorted_zst_file, UnsortedWords,
};

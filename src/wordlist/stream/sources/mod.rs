//! Source iterators for WordStream.

mod sorted_file;
mod unsorted_file;

pub use sorted_file::{from_sorted_file, SortedFileLines};
pub use unsorted_file::{from_unsorted_file, UnsortedFileWords};

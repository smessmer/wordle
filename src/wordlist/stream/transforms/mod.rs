//! Transform iterators for WordStream.

mod dedup;
mod filter;
mod lowercase;
mod merge;
mod filter_non_alphabetic;

pub use dedup::DedupStream;
pub use filter::FilterStream;
pub use filter_non_alphabetic::filter_non_alphabetic;
pub use lowercase::LowercaseStream;
pub use merge::MergeStream;

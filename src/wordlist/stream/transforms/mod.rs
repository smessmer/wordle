//! Transform iterators for WordStream.

mod dedup;
mod filter;
mod lowercase;
mod merge;

pub use dedup::DedupStream;
pub use filter::FilterStream;
pub use lowercase::LowercaseStream;
pub use merge::MergeStream;

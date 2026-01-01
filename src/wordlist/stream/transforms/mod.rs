//! Transform iterators for WordStream.

mod dedup;
mod filter;
mod lowercase;

pub use dedup::DedupStream;
pub use filter::FilterStream;
pub use lowercase::LowercaseStream;

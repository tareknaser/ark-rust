mod index;
mod serde;
mod utils;
mod watch;

pub use index::ResourceIndex;
pub use utils::load_or_build_index;
pub use watch::watch_index;

#[cfg(test)]
mod tests;

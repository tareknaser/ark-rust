mod index;
mod serde;
mod utils;

pub use utils::load_or_build_index;

pub use index::ResourceIndex;

#[cfg(test)]
mod test_blake3;
#[cfg(test)]
mod test_crc32;
#[cfg(test)]
mod test_utils;

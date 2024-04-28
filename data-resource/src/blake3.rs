use crate::ResourceIdTrait;
use data_error::{ArklibError, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use blake3::Hasher;

/// Represents a resource identifier using the BLAKE3 algorithm.
///
/// Uses [`blake3`] crate to compute the hash value.
#[derive(
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Hash,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Serialize,
)]
pub struct ResourceId {}

#[derive(
    Hash, Ord, PartialOrd, Eq, PartialEq, Clone, Debug, Serialize, Deserialize,
)]
pub struct Hash(Vec<u8>);

impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use hex formatting for string representation
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for Hash {
    type Err = ArklibError;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() % 2 != 0 {
            return Err(ArklibError::Parse);
        }

        let mut result = Vec::new();
        for i in 0..s.len() / 2 {
            let byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_| ArklibError::Parse)?;
            result.push(byte);
        }
        Ok(Hash(result))
    }
}

impl ResourceIdTrait for ResourceId {
    type HashType = Hash;

    fn from_path<P: AsRef<Path>>(file_path: P) -> Result<Self::HashType> {
        log::debug!("Computing BLAKE3 hash for file: {:?}", file_path.as_ref());

        let file = fs::File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        let mut buffer = Vec::new();
        loop {
            let bytes_read = reader.read_until(b'\n', &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer);
            buffer.clear();
        }
        let hash = hasher.finalize();
        Ok(Hash(hash.as_bytes().to_vec()))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::HashType> {
        log::debug!("Computing BLAKE3 hash for bytes");

        let mut hasher = Hasher::new();
        hasher.update(bytes);
        let hash = hasher.finalize();
        Ok(Hash(hash.as_bytes().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_to_string() {
        let hash = Hash(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let hash_str = hash.to_string();
        assert_eq!(hash_str, "00010203040506070809");

        let hash = Hash::from_str(&hash_str).expect("Failed to parse hash");
        assert_eq!(hash, Hash(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    }

    #[test]
    fn sanity_check() {
        let file_path = Path::new("../test-assets/lena.jpg");
        let id = ResourceId::from_path(file_path)
            .expect("Failed to compute resource identifier");
        assert_eq!(
            id,
            Hash(vec![
                23, 43, 75, 241, 72, 232, 88, 177, 61, 222, 15, 198, 97, 52,
                19, 188, 183, 85, 46, 92, 78, 92, 69, 25, 90, 198, 200, 15, 32,
                235, 95, 245
            ])
        );

        let raw_bytes = fs::read(file_path).expect("Failed to read file");
        let id = ResourceId::from_bytes(&raw_bytes)
            .expect("Failed to compute resource identifier");
        assert_eq!(
            id,
            Hash(vec![
                23, 43, 75, 241, 72, 232, 88, 177, 61, 222, 15, 198, 97, 52,
                19, 188, 183, 85, 46, 92, 78, 92, 69, 25, 90, 198, 200, 15, 32,
                235, 95, 245
            ])
        );
    }
}

use fastmurmur3::murmur3_x64_128;
use uuid::Uuid;

/// A utility struct for hashing strings using the Murmur3 algorithm.
///
/// This is primarily used for generating a hash of the MCTS tree for debugging and
/// verification purposes.
pub struct MurMurHasher;

impl MurMurHasher {
    /// Hashes a given string slice and returns it as a hex string.
    pub fn hash(str: &str) -> String {
        let hash = murmur3_x64_128(str.as_bytes(), 0);
        let guid = Uuid::from_bytes(hash.to_le_bytes());
        guid.to_string().to_lowercase().replace('-', "")
    }
}

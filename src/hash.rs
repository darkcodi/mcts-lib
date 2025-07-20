use fastmurmur3::murmur3_x64_128;
use uuid::Uuid;

pub struct MurMurHasher;

impl MurMurHasher {
    pub fn hash(str: &str) -> String {
        let hash = murmur3_x64_128(str.as_bytes(), 0);
        let guid = Uuid::from_bytes(hash.to_le_bytes());
        let result = guid.to_string().to_lowercase().replace("-", "");
        result
    }
}

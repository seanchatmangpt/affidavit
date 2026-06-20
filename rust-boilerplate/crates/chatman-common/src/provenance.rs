//! Content-addressing and rolling hash utilities using BLAKE3.

/// Compute the BLAKE3 content address of `bytes` as a lowercase hex string.
pub fn content_address(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// A rolling BLAKE3 hasher that can be updated incrementally.
pub struct RollingHash {
    hasher: blake3::Hasher,
}

impl RollingHash {
    /// Create a new rolling hasher.
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Feed more bytes into the hasher.
    pub fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    /// Finalize and return the hash as a lowercase hex string.
    pub fn finalize(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

impl Default for RollingHash {
    fn default() -> Self {
        Self::new()
    }
}

/// Return `true` if `s` is exactly 64 lowercase hexadecimal characters.
pub fn is_valid_digest(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

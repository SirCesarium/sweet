//! Logic for identifying code clones and calculating duplication percentage.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Breaks a list of hashes into windows and maps each chunk (via its hash) to its positions.
///
/// Returns a map where the key is the chunk hash and the value is a list of
/// starting indices within the input slice.
#[must_use]
pub fn get_chunks(hashes: &[u64], window_size: usize) -> HashMap<u64, Vec<usize>> {
    if hashes.len() < window_size {
        return HashMap::new();
    }

    let mut chunks: HashMap<u64, Vec<usize>> = HashMap::new();

    for i in 0..=hashes.len() - window_size {
        let mut chunk_hasher = DefaultHasher::new();
        hashes[i..i + window_size].hash(&mut chunk_hasher);
        chunks.entry(chunk_hasher.finish()).or_default().push(i);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chunks() {
        let hashes = vec![1, 2, 3, 1, 2, 3, 4];
        let chunks = get_chunks(&hashes, 3);
        assert_eq!(chunks.len(), 4);
        let h = calculate_hash(&[1, 2, 3]);

        let pos = chunks.get(&h).map_or_else(Vec::new, Clone::clone);
        assert_eq!(pos.len(), 2);
        assert!(pos.contains(&0));
        assert!(pos.contains(&3));
    }

    fn calculate_hash(data: &[u64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

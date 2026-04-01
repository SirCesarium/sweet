//! Logic for identifying code clones and calculating duplication percentage.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Holds the result of a repetition analysis.
pub struct RepetitionResult {
    /// Percentage of duplicated code (0.0 to 100.0).
    pub percentage: f64,
    /// List of computed chunk hashes and their line positions.
    pub hashes: Vec<u64>,
}

/// Analyze content for repeated lines and returns a summary.
#[must_use]
pub fn analyze_repetition(content: &[u8], window_size: usize) -> RepetitionResult {
    if content.is_empty() || window_size == 0 {
        return RepetitionResult {
            percentage: 0.0,
            hashes: Vec::new(),
        };
    }

    let mut hashes = Vec::new();
    let mut line_count = 0;

    for line in content.split(|&b| b == b'\n') {
        line_count += 1;
        let mut line_hasher = DefaultHasher::new();
        trim_bytes(line).hash(&mut line_hasher);
        hashes.push(line_hasher.finish());
    }

    if line_count < window_size {
        return RepetitionResult {
            percentage: 0.0,
            hashes,
        };
    }

    let chunks = get_chunks(&hashes, window_size);
    let mut duplicated_lines = HashSet::new();

    for positions in chunks.values() {
        if positions.len() > 1 {
            for &pos in positions {
                for i in 0..window_size {
                    duplicated_lines.insert(pos + i);
                }
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let percentage = (duplicated_lines.len() as f64 / line_count as f64) * 100.0;

    RepetitionResult { percentage, hashes }
}

fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|&b| !b.is_ascii_whitespace())
        .map_or(start, |p| p + 1);
    &bytes[start..end]
}

/// Breaks a list of hashes into windows and maps each chunk (via its hash) to its positions.
#[must_use]
pub fn get_chunks(hashes: &[u64], window_size: usize) -> HashMap<u64, Vec<usize>> {
    if hashes.len() < window_size {
        return HashMap::new();
    }

    let mut chunks: HashMap<u64, Vec<usize>> = HashMap::new();

    for i in 0..=hashes.len() - window_size {
        let mut chunk_hasher = DefaultHasher::new();
        hashes[i..i + window_size].hash(&mut chunk_hasher);
        chunks.entry(chunk_hasher.finish()).or_default().push(i + 1);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition() {
        let content = b"a\nb\nc\na\nb\nc\nd";
        let res = analyze_repetition(content, 3);
        assert!(res.percentage > 0.0);
        assert_eq!(res.hashes.len(), 7);
    }
}

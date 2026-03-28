//! Duplicate code detection using sliding window hashing.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Measured repetition results for a specific file.
pub struct RepetitionResult {
    /// Percentage of duplicated code.
    pub percentage: f64,
    /// Detailed information about the duplicated chunks.
    pub hashes: Vec<u64>,
    /// Line mapping to retrieve original content.
    pub line_map: Vec<String>,
}

/// Measures the percentage of repetitive code blocks in the content.
///
/// Uses normalized line hashing and a sliding window to identify duplicate code chunks.
#[must_use]
pub fn analyze_repetition(content: &str) -> RepetitionResult {
    let lines_content: Vec<String> = content
        .lines()
        .filter(|l| !l.contains("@sweetignore"))
        .map(std::string::ToString::to_string)
        .collect();

    if lines_content.is_empty() {
        return RepetitionResult {
            percentage: 0.0,
            hashes: vec![],
            line_map: vec![],
        };
    }

    let hashes: Vec<u64> = lines_content
        .iter()
        .map(|l| hash_normalized_line(l))
        .collect();

    let window_size = 4;
    if hashes.len() < window_size {
        return RepetitionResult {
            percentage: 0.0,
            hashes,
            line_map: lines_content,
        };
    }

    let mut repetitive_lines = vec![false; hashes.len()];
    let mut chunks: HashMap<&[u64], Vec<usize>> = HashMap::with_capacity(hashes.len());

    for i in 0..=hashes.len() - window_size {
        let chunk = &hashes[i..i + window_size];

        if lines_content[i..i + window_size]
            .iter()
            .all(|l| l.trim().len() < 3)
        {
            continue;
        }

        chunks.entry(chunk).or_default().push(i);
    }

    for occurrences in chunks.values().filter(|v| v.len() > 1) {
        for &start_idx in occurrences {
            for r in &mut repetitive_lines[start_idx..start_idx + window_size] {
                *r = true;
            }
        }
    }

    let repeated_count = repetitive_lines.iter().filter(|&&r| r).count();

    #[allow(clippy::cast_precision_loss)]
    let percentage = (repeated_count as f64 / hashes.len() as f64) * 100.0;

    RepetitionResult {
        percentage,
        hashes,
        line_map: lines_content,
    }
}

/// Computes a hash for a line after removing whitespace and converting to lowercase.
#[must_use]
pub fn hash_normalized_line(line: &str) -> u64 {
    let mut s = DefaultHasher::new();
    for c in line.chars().filter(|c| !c.is_whitespace()) {
        for lc in c.to_lowercase() {
            lc.hash(&mut s);
        }
    }
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition() {
        let no_rep =
            "fn main() {\n    let x = 1;\n    let y = 2;\n    let z = 3;\n    let w = 4;\n}";
        assert!(analyze_repetition(no_rep).percentage.abs() < f64::EPSILON);

        let rep = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;\nlet a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;";
        assert!(analyze_repetition(rep).percentage > 0.0);
    }
}

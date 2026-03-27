//! Logic for analyzing code repetition.

use std::collections::HashMap;

/// Analyzes the percentage of repetitive code in the content.
///
/// It uses a sliding window of lines to find duplicate blocks.
/// Lines are normalized (whitespace removed, lowercased) for fuzzy matching.
#[must_use]
pub fn analyze_repetition(content: &str) -> f64 {
    let lines: Vec<String> = content
        .lines()
        .map(|l| {
            l.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_lowercase()
        })
        .collect();

    if lines.is_empty() {
        return 0.0;
    }

    let window_size = 4;
    if lines.len() < window_size {
        return 0.0;
    }

    let mut repetitive_lines = vec![false; lines.len()];
    let mut chunks: HashMap<Vec<String>, Vec<usize>> = HashMap::new();

    for i in 0..=lines.len() - window_size {
        let chunk: Vec<String> = lines[i..i + window_size].to_vec();

        // Skip chunks that only contain very short lines (e.g., just brackets)
        if chunk.iter().all(|l| l.len() < 3) {
            continue;
        }

        chunks.entry(chunk).or_default().push(i);
    }

    for occurrences in chunks.values().filter(|v| v.len() > 1) {
        for &start_idx in occurrences {
            repetitive_lines[start_idx..start_idx + window_size]
                .iter_mut()
                .for_each(|r| *r = true);
        }
    }

    let repeated_count = repetitive_lines.iter().filter(|&&r| r).count();
    #[allow(clippy::cast_precision_loss)]
    {
        (repeated_count as f64 / lines.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition() {
        let no_rep =
            "fn main() {\n    let x = 1;\n    let y = 2;\n    let z = 3;\n    let w = 4;\n}";
        assert!(analyze_repetition(no_rep).abs() < f64::EPSILON);

        let rep = "let a = 1; let b = 2; let c = 3; let d = 4; let a = 1; let b = 2; let c = 3; let d = 4;";
        assert!(analyze_repetition(rep) > 0.0);

        let fuzzy =
            "let a = 1; let b = 2; let c = 3; let d = 4; let a=1; let b=2; let c=3; let d=4;";
        assert!(analyze_repetition(fuzzy) > 0.0);
    }
}

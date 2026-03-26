//! Logic for analyzing code coupling through import counting.

use regex::Regex;
use std::sync::OnceLock;

/// Pre-compiled regex patterns for supported languages.
static RS_RE: OnceLock<Regex> = OnceLock::new();
static PY_RE: OnceLock<Vec<Regex>> = OnceLock::new();
static CS_RE: OnceLock<Regex> = OnceLock::new();
static DEFAULT_RE: OnceLock<Regex> = OnceLock::new();

/// Counts the number of imports in a file based on its language extension.
///
/// Uses pre-compiled regex patterns for maximum performance.
///
/// # Panics
///
/// Panics if a regex pattern is invalid (should never happen with hardcoded patterns).
#[must_use]
pub fn count_imports(content: &str, extension: &str) -> usize {
    match extension {
        "rs" => {
            let re = RS_RE.get_or_init(|| Regex::new(r"^use\s+").expect("Valid regex"));
            count_matches(content, &[re])
        }
        "py" => {
            let python_regexes = PY_RE.get_or_init(|| {
                vec![
                    Regex::new(r"^import\s+").expect("Valid regex"),
                    Regex::new(r"^from\s+\w+\s+import").expect("Valid regex"),
                ]
            });
            let regex_refs: Vec<&Regex> = python_regexes.iter().collect();
            count_matches(content, &regex_refs)
        }
        "cs" => {
            let re = CS_RE.get_or_init(|| Regex::new(r"^using\s+").expect("Valid regex"));
            count_matches(content, &[re])
        }
        _ => {
            let re = DEFAULT_RE.get_or_init(|| Regex::new(r"^import\s+").expect("Valid regex"));
            count_matches(content, &[re])
        }
    }
}

/// Helper to count matches for a set of regexes.
fn count_matches(content: &str, regexes: &[&Regex]) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            regexes.iter().any(|re| re.is_match(trimmed))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rs_imports() {
        let code = "use std::io;\nuse crate::analyzer;\nfn main() {}";
        assert_eq!(count_imports(code, "rs"), 2);
    }

    #[test]
    fn test_py_imports() {
        let code = "import os\nfrom sys import path\nprint('hello')";
        assert_eq!(count_imports(code, "py"), 2);
    }

    #[test]
    fn test_java_ts_imports() {
        let code = "import React from 'react';\nimport { useState } from 'react';";
        assert_eq!(count_imports(code, "ts"), 2);
    }
}

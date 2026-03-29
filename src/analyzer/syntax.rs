//! Dependency and import counting logic.

use crate::languages::LanguageRegistry;

/// Counts the number of module imports/includes based on language-specific keywords.
#[must_use]
pub fn count_imports(content: &str, extension: &str) -> usize {
    let registry = LanguageRegistry::get();
    let Some(lang) = registry.get_by_extension(extension) else {
        return 0;
    };

    let keywords = lang.import_keywords();

    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                return false;
            }
            keywords.iter().any(|&kw| {
                if kw.ends_with('(') || kw.ends_with('"') {
                    // Function-like or quote-starting keywords can be anywhere (e.g., preload in GDScript)
                    trimmed.contains(kw)
                } else {
                    // Statement-like keywords must be at the start
                    trimmed.starts_with(kw)
                }
            })
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

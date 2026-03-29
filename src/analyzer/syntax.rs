//! Dependency and import counting logic.

use crate::languages::LanguageRegistry;

/// Counts the number of module imports/includes based on language-specific keywords.
#[must_use]
pub fn count_imports(content: &str, extension: &str) -> usize {
    let registry = LanguageRegistry::get();
    let Some(lang) = registry.get_by_extension(extension) else {
        return 0;
    };

    lang.count_imports(content)
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

    #[test]
    fn test_go_imports() {
        let code = "import (\n\t\"fmt\"\n\t\"os\"\n)\nimport \"math\"";
        assert_eq!(count_imports(code, "go"), 3);
    }
}

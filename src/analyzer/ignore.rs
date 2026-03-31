//! Analysis exclusion logic based on @sweetignore and @swt-disable comments.

use std::collections::HashSet;

/// Check if a file should be ignored globally via a top-level @sweetignore.
#[must_use]
pub fn is_file_ignored(content: &str) -> bool {
    content
        .lines()
        .take(10)
        .any(|line| line.contains("@sweetignore"))
}

/// Check if a specific line or block is marked for exclusion.
#[must_use]
pub fn is_line_ignored(line: &str) -> bool {
    line.contains("@sweetignore")
}

/// Extract a set of rules to be disabled for a specific file.
///
/// Looks for comments like `@swt-disable <rule1> <rule2>` in the first 20 lines.
#[must_use]
pub fn get_disabled_rules(content: &str) -> HashSet<String> {
    let mut disabled = HashSet::new();
    let marker = "@swt-disable";
    for line in content.lines().take(20) {
        let lower = line.to_lowercase();
        if let Some(pos) = lower.find(marker) {
            let rules = &line[pos + marker.len()..];
            for rule in rules.split_whitespace() {
                disabled.insert(rule.to_lowercase());
            }
        }
    }
    disabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_ignored() {
        assert!(is_file_ignored("// @sweetignore\nfn main() {}"));
        assert!(!is_file_ignored("fn main() {}"));
    }

    #[test]
    fn test_get_disabled_rules() {
        let content = "// @swt-disable max-lines max-depth\nfn main() {}";
        let disabled = get_disabled_rules(content);
        assert!(disabled.contains("max-lines"));
        assert!(disabled.contains("max-depth"));
        assert!(!disabled.contains("max-imports"));
    }

    #[test]
    fn test_get_disabled_rules_case_insensitive() {
        let content = "# @SWT-DISABLE Max-Lines";
        let disabled = get_disabled_rules(content);
        assert!(disabled.contains("max-lines"));
    }
}

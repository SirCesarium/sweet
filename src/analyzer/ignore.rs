//! Analysis exclusion logic based on @sweetignore and @swt-disable comments.

use std::collections::HashSet;
use std::str;

/// Check if a file should be ignored globally via a top-level @sweetignore.
#[must_use]
pub fn is_file_ignored(content: &[u8]) -> bool {
    content
        .split(|&b| b == b'\n')
        .take(10)
        .any(|line| line.windows(12).any(|w| w == b"@sweetignore"))
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
pub fn get_disabled_rules(content: &[u8]) -> HashSet<String> {
    let mut disabled = HashSet::new();
    let marker = b"@swt-disable";
    for line in content.split(|&b| b == b'\n').take(20) {
        let line_lower = line.to_ascii_lowercase();
        if let Some(pos) = find_subsequence(&line_lower, marker) {
            let rules_part = &line[pos + marker.len()..];
            if let Ok(rules_str) = str::from_utf8(rules_part) {
                for rule in rules_str.split_whitespace() {
                    disabled.insert(rule.to_lowercase());
                }
            }
        }
    }
    disabled
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_ignored() {
        assert!(is_file_ignored(b"// @sweetignore\nfn main() {}"));
        assert!(!is_file_ignored(b"fn main() {}"));
    }

    #[test]
    fn test_get_disabled_rules() {
        let content = b"// @swt-disable max-lines max-depth\nfn main() {}";
        let disabled = get_disabled_rules(content);
        assert!(disabled.contains("max-lines"));
        assert!(disabled.contains("max-depth"));
        assert!(!disabled.contains("max-imports"));
    }
}

//! Logic for measuring code complexity and control flow depth.

/// Estimates the maximum nesting depth of a file's control flow.
///
/// Analysis is based on leading whitespace indentation.
/// Each tab or `indent_size` spaces equals one nesting level.
#[must_use]
pub fn analyze_depth(content: &str, indent_size: usize) -> usize {
    get_line_depths(content, indent_size)
        .map(|(_, depth)| depth)
        .max()
        .unwrap_or(0)
}

/// Find lines where the nesting depth exceeds a given threshold.
#[must_use]
pub fn find_deep_lines(content: &str, indent_size: usize, threshold: usize) -> Vec<(usize, usize)> {
    get_line_depths(content, indent_size)
        .filter(|&(_, depth)| depth > threshold)
        .collect()
}

/// Internal helper to calculate depth for each non-empty line.
fn get_line_depths(content: &str, indent_size: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
    content.lines().enumerate().filter_map(move |(i, line)| {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return None;
        }

        let leading_whitespace = line.len() - trimmed.len();
        let depth = if line.starts_with('\t') {
            leading_whitespace
        } else {
            leading_whitespace / indent_size
        };

        Some((i + 1, depth))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_spaces() {
        let code = "fn main() {\n    if true {\n        println!();\n    }\n}";
        assert_eq!(analyze_depth(code, 4), 2);
    }

    #[test]
    fn test_depth_tabs() {
        let code = "fn main() {\n\tif true {\n\t\tprintln!();\n\t}\n}";
        assert_eq!(analyze_depth(code, 4), 2);
    }

    #[test]
    fn test_find_deep_lines() {
        let code = "fn main() {\n    if true {\n        if true {\n            println!();\n        }\n    }\n}";
        let deep = find_deep_lines(code, 4, 1);
        assert_eq!(deep.len(), 3);
        let deep_lines: Vec<usize> = deep.iter().map(|(l, _)| *l).collect();

        assert!(deep_lines.contains(&3));
        assert!(deep_lines.contains(&4));
        assert!(deep_lines.contains(&5));
    }
}

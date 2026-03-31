//! Line counting and file volume metrics.

/// Count the number of source lines in the content.
#[must_use]
pub fn count_lines(content: &str) -> usize {
    content.lines().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines("line1\nline2\nline3"), 3);
    }

    #[test]
    fn test_count_empty() {
        assert_eq!(count_lines(""), 0);
    }
}

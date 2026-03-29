//! Logic for identifying and counting function/method declarations.

use crate::languages::LanguageRegistry;

/// Counts the number of function/method declarations in the content.
#[must_use]
pub fn count_functions(content: &str, extension: &str) -> usize {
    let registry = LanguageRegistry::get();
    let Some(lang) = registry.get_by_extension(extension) else {
        return 0;
    };

    let keywords = lang.function_keywords();

    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            keywords.iter().any(|&kw| trimmed.starts_with(kw))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rs_functions() {
        let code = "fn main() {}\nasync fn test() {}";
        assert_eq!(count_functions(code, "rs"), 2);
    }
}

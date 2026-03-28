//! High-performance comment stripping with documentation preservation.

use crate::languages::LanguageRegistry;

/// Removes comments from source code while optionally preserving documentation.
///
/// Uses language-specific delimiters from the registry to identify and strip comments.
#[must_use]
pub fn remove_comments(content: &str, extension: &str, aggressive: bool) -> String {
    let registry = LanguageRegistry::get();
    let Some(lang) = registry.get_by_extension(extension) else {
        return content.to_string();
    };

    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    let line_prefix = lang.line_comment();
    let block_delimiters = lang.block_comment();

    let mut in_string = false;
    let mut string_char = '\"';
    let mut in_block_comment = false;
    let mut in_line_comment = false;

    while let Some(current) = chars.next() {
        if in_block_comment {
            if let Some((_, end)) = block_delimiters {
                let end_chars: Vec<char> = end.chars().collect();
                if current == end_chars[0]
                    && end_chars.len() > 1
                    && chars.peek() == Some(&end_chars[1])
                {
                    chars.next();
                    in_block_comment = false;
                    continue;
                }
            }
            if current == '\n' {
                result.push('\n');
            }
            continue;
        }

        if in_line_comment {
            if current == '\n' {
                in_line_comment = false;
                result.push('\n');
            }
            continue;
        }

        if in_string {
            result.push(current);
            if current == '\\' {
                if let Some(n) = chars.next() {
                    result.push(n);
                }
            } else if current == string_char {
                in_string = false;
            }
            continue;
        }

        if current == '\"' || current == '\'' || current == '`' {
            in_string = true;
            string_char = current;
            result.push(current);
            continue;
        }

        if let Some((start, _)) = block_delimiters {
            let start_chars: Vec<char> = start.chars().collect();
            if current == start_chars[0]
                && start_chars.len() > 1
                && chars.peek() == Some(&start_chars[1])
            {
                let mut is_doc = false;
                if start == "/*" {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        is_doc = true;
                    }

                    if !aggressive && is_doc {
                        result.push_str("/**");
                        chars.next();
                        continue;
                    }

                    in_block_comment = true;
                    continue;
                }
            }
        }

        if let Some(prefix) = line_prefix {
            let prefix_chars: Vec<char> = prefix.chars().collect();
            if current == prefix_chars[0]
                && (prefix_chars.len() == 1
                    || (prefix_chars.len() > 1 && chars.peek() == Some(&prefix_chars[1])))
            {
                if prefix_chars.len() > 1 {
                    chars.next();
                }

                let is_doc = if prefix == "//" {
                    chars.peek() == Some(&'/') || chars.peek() == Some(&'!')
                } else {
                    false
                };

                if !aggressive && is_doc {
                    result.push_str(prefix);
                    continue;
                }

                in_line_comment = true;
                continue;
            }
        }

        result.push(current);
    }

    normalize_whitespace(&result)
}

/// Trims trailing spaces and collapses consecutive empty lines.
fn normalize_whitespace(content: &str) -> String {
    let mut res = String::with_capacity(content.len());
    let mut last_was_empty = false;

    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !last_was_empty {
                res.push('\n');
                last_was_empty = true;
            }
        } else {
            if !res.is_empty() {
                res.push('\n');
            }
            res.push_str(trimmed);
            last_was_empty = false;
        }
    }

    res.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(code: &str, expected: &str, aggressive: bool) {
        assert_eq!(remove_comments(code, "rs", aggressive), expected);
    }

    #[test]
    fn test_uncommenting() {
        check(
            "fn main() {\n    // comment\n    /* block */\n    let x = 5;\n}",
            "fn main() {\n\n    let x = 5;\n}",
            true,
        );
        check("/// doc\nfn main() {}", "/// doc\nfn main() {}", false);
        check("/// doc\nfn main() {}", "fn main() {}", true);
        check(
            "let s = \"http://example.com\";",
            "let s = \"http://example.com\";",
            true,
        );
    }
}

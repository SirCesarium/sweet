//! Logic for removing comments while preserving documentation or string literals.

/// Strips comments from the provided source code content.
///
/// It supports C-style comments (//, /* */) and Python-style comments (#).
///
/// Arguments:
/// * `aggressive`: If true, it also removes documentation comments (///, /**).
/// * `extension`: The file extension used to determine the comment style.
#[must_use]
pub fn remove_comments(content: &str, extension: &str, aggressive: bool) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    let is_python = extension == "py";
    let is_c_style = matches!(extension, "rs" | "java" | "ts" | "js" | "cs");

    let mut in_string = false;
    let mut string_char = '\"';
    let mut in_block_comment = false;
    let mut in_line_comment = false;

    while i < chars.len() {
        let current = chars[i];
        let next = chars.get(i + 1);
        let next_next = chars.get(i + 2);

        // State: Inside /* block comment */
        if in_block_comment {
            if is_c_style && current == '*' && next == Some(&'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        // State: Inside // line comment
        if in_line_comment {
            if current == '\n' {
                in_line_comment = false;
                result.push('\n');
            }
            i += 1;
            continue;
        }

        if in_string {
            result.push(current);
            #[allow(clippy::collapsible_if)]
            if current == '\\' {
                if let Some(n) = next {
                    result.push(*n);
                    i += 2;
                    continue;
                }
            }
            if current == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        // Detect String start
        if current == '\"' || current == '\'' || (current == '`' && !is_python) {
            in_string = true;
            string_char = current;
            result.push(current);
            i += 1;
            continue;
        }

        // Detect Comment starts (C-style and Python)
        if is_c_style && current == '/' {
            if let Some(&c) = next {
                let is_block = c == '*';
                let is_line = c == '/';
                if is_block || is_line {
                    let is_doc = next_next.map_or(false, |&n| n == '*' || n == '/' || n == '!');
                    if !aggressive && is_doc {
                        result.push('/');
                        result.push(c);
                        i += 2;
                        continue;
                    }
                    if is_block { in_block_comment = true; } else { in_line_comment = true; }
                    i += 2;
                    continue;
                }
            }
        }

        if is_python && current == '#' {
            in_line_comment = true;
            i += 1;
            continue;
        }

        result.push(current);
        i += 1;
    }

    // Clean up trailing whitespace and normalize consecutive empty lines.
    normalize_whitespace(&result)
}

/// Normalizes whitespace by removing trailing spaces and collapsing multiple empty lines.
fn normalize_whitespace(content: &str) -> String {
    let mut final_lines = Vec::new();
    let mut last_was_empty = false;

    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !last_was_empty {
                final_lines.push("");
                last_was_empty = true;
            }
        } else {
            final_lines.push(trimmed);
            last_was_empty = false;
        }
    }

    // Trim leading/trailing empty lines for cleaner comparison in tests
    let mut res = final_lines.join("\n");
    res = res.trim().to_string();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(code: &str, expected: &str, aggressive: bool) {
        assert_eq!(remove_comments(code, "rs", aggressive), expected);
    }

    #[test]
    fn test_uncommenting() {
        check("fn main() {\n    // comment\n    /* block */\n    let x = 5;\n}", "fn main() {\n\n    let x = 5;\n}", true);
        check("/// doc\nfn main() {}", "/// doc\nfn main() {}", false);
        check("/// doc\nfn main() {}", "fn main() {}", true);
        check("let s = \"http://example.com\";", "let s = \"http://example.com\";", true);
    }
}

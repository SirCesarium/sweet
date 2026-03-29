use crate::languages::Language;

pub struct Go;

impl Language for Go {
    fn name(&self) -> &'static str {
        "Go"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["go"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some("//")
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(("/*", "*/"))
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["import"]
    }
    fn function_keywords(&self) -> &'static [&'static str] {
        &["func "]
    }

    fn count_imports(&self, content: &str) -> usize {
        let mut count = 0;
        let mut in_block = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import (") {
                in_block = true;
                continue;
            }
            if in_block {
                if trimmed == ")" {
                    in_block = false;
                    continue;
                }
                if !trimmed.is_empty() && !trimmed.starts_with("//") {
                    count += 1;
                }
                continue;
            }
            if trimmed.starts_with("import \"") || trimmed.starts_with("import ") {
                count += 1;
            }
        }
        count
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 400,
            max_imports: 20,
            ..Default::default()
        }
    }
}

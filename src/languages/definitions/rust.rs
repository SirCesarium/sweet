use crate::languages::{Language, c_base::CBaseRules};

pub struct Rust;

impl Language for Rust {
    fn name(&self) -> &'static str {
        "Rust"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some(CBaseRules::LINE_COMMENT)
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(CBaseRules::BLOCK_COMMENT)
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["use "]
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 300,
            max_imports: 30,
            ..Default::default()
        }
    }
}

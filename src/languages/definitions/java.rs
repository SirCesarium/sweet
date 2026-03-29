use crate::languages::{Language, c_base::CBaseRules};

pub struct Java;

impl Language for Java {
    fn name(&self) -> &'static str {
        "Java"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["java"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some(CBaseRules::LINE_COMMENT)
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(CBaseRules::BLOCK_COMMENT)
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["import "]
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 500,
            max_imports: 30,
            ..Default::default()
        }
    }
}

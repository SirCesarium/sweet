use crate::languages::{Language, c_base::CBaseRules};

pub struct CSharp;

impl Language for CSharp {
    fn name(&self) -> &'static str {
        "C#"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some(CBaseRules::LINE_COMMENT)
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(CBaseRules::BLOCK_COMMENT)
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["using "]
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 500,
            max_imports: 30,
            ..Default::default()
        }
    }

    fn function_keywords(&self) -> &'static [&'static str] {
        &[
            "public ",
            "private ",
            "protected ",
            "static ",
            "void ",
            "internal ",
            "async ",
            "override ",
        ]
    }
}

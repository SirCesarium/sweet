use crate::languages::{Language, c_base::CBaseRules};

pub struct JavaScript;

impl Language for JavaScript {
    fn name(&self) -> &'static str {
        "JavaScript"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["js", "mjs", "cjs"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some(CBaseRules::LINE_COMMENT)
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(CBaseRules::BLOCK_COMMENT)
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["import ", "require("]
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 400,
            max_imports: 30,
            ..Default::default()
        }
    }

    fn function_keywords(&self) -> &'static [&'static str] {
        &["function ", "async function ", "const ", "let ", "var "]
        // Note: const/let/var can be arrow functions. Simple starts_with might catch some variables too.
        // For now, let's keep it simple or look for " => ".
        // Given the current architecture, starts_with is the standard.
    }
}

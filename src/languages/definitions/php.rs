use crate::languages::Language;

pub struct PHP;

impl Language for PHP {
    fn name(&self) -> &'static str {
        "PHP"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["php"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some("//")
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        Some(("/*", "*/"))
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &[
            "use ",
            "require ",
            "require_once ",
            "include ",
            "include_once ",
        ]
    }
    fn function_keywords(&self) -> &'static [&'static str] {
        &[
            "function ",
            "public function ",
            "protected function ",
            "private function ",
            "static function ",
            "final public function ",
            "final protected function ",
            "final private function ",
        ]
    }
    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 500,
            max_imports: 30,
            ..Default::default()
        }
    }
}

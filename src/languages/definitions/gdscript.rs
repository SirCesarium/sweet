use crate::languages::Language;

pub struct GDScript;

impl Language for GDScript {
    fn name(&self) -> &'static str {
        "GDScript"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["gd"]
    }
    fn line_comment(&self) -> Option<&'static str> {
        Some("#")
    }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        None
    }
    fn import_keywords(&self) -> &'static [&'static str] {
        &["extends", "preload(", "load(", "class_name"]
    }

    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 400,
            max_depth: 7,
            ..Default::default()
        }
    }

    fn function_keywords(&self) -> &'static [&'static str] {
        &["func ", "static func "]
    }
}

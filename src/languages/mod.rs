//! Strategy pattern for language-specific analysis rules.

pub mod c_base;
pub mod definitions;

use std::collections::HashMap;
use std::sync::OnceLock;

/// Interface for language-specific analysis strategies.
pub trait Language: Send + Sync {
    /// Friendly name of the language (e.g., "Rust").
    fn name(&self) -> &'static str;

    /// File extensions associated with this language.
    fn extensions(&self) -> &'static [&'static str];

    /// Delimiter for single-line comments.
    fn line_comment(&self) -> Option<&'static str>;

    /// Start and end delimiters for multi-line block comments.
    fn block_comment(&self) -> Option<(&'static str, &'static str)>;

    /// Keywords used to declare imports or dependencies.
    fn import_keywords(&self) -> &'static [&'static str];

    /// Number of spaces representing one level of indentation.
    fn indent_size(&self) -> usize {
        4
    }

    /// Default health thresholds specifically tuned for this language.
    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds::default()
    }

    /// Keywords or patterns that identify a function/method declaration.
    fn function_keywords(&self) -> &'static [&'static str];
}

/// Thread-safe registry for managing supported languages.
pub struct LanguageRegistry {
    languages: Vec<Box<dyn Language>>,
    extension_map: HashMap<&'static str, usize>,
}

static REGISTRY: OnceLock<LanguageRegistry> = OnceLock::new();

impl LanguageRegistry {
    /// Returns the global registry instance.
    #[must_use]
    pub fn get() -> &'static Self {
        REGISTRY.get_or_init(Self::new)
    }

    fn new() -> Self {
        let languages: Vec<Box<dyn Language>> = vec![
            Box::new(definitions::rust::Rust),
            Box::new(definitions::python::Python),
            Box::new(definitions::javascript::JavaScript),
            Box::new(definitions::typescript::TypeScript),
            Box::new(definitions::java::Java),
            Box::new(definitions::csharp::CSharp),
            Box::new(definitions::gdscript::GDScript),
        ];

        let mut extension_map = HashMap::new();
        for (i, lang) in languages.iter().enumerate() {
            for ext in lang.extensions() {
                extension_map.insert(*ext, i);
            }
        }

        Self {
            languages,
            extension_map,
        }
    }

    /// Resolves a language strategy by file extension.
    #[must_use]
    pub fn get_by_extension(&self, ext: &str) -> Option<&dyn Language> {
        self.extension_map
            .get(ext)
            .map(|&i| self.languages[i].as_ref())
    }

    /// Returns a list of all supported file extensions.
    #[must_use]
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }
}

# 🍬 Contributing to Sweet

Thank you for your interest in improving Sweet! This guide provides technical specifications for contributing to the core engine and adding support for new programming languages.

---

## 🏗️ Project Structure

- **`src/`**: The Rust core.
  - **`analyzer/`**: Logic for metrics (complexity, repetition, nesting depth, etc.).
  - **`languages/`**: Language definitions and the Strategy Pattern registry.
  - **`config/`**: Configuration resolution and hierarchical threshold merging.
  - **`bin/lsp.rs`**: The Language Server Protocol implementation.
  - **`main.rs`**: CLI entry point.
- **`editors/vscode/`**: VS Code extension (TypeScript).
- **`schema.json`**: Auto-generated JSON schema for `.swtrc` validation.

---

## 🚀 Adding a New Language

Sweet uses the **Strategy Pattern** to handle language-specific rules. To add support for a new language (e.g., **Go**), follow these steps:

### 1. Define the Language Strategy
Create a new file in `src/languages/definitions/go.rs`:

```rust
use crate::languages::Language;

pub struct Go;

impl Language for Go {
    fn name(&self) -> &'static str { "Go" }
    fn extensions(&self) -> &'static [&'static str] { &["go"] }
    fn line_comment(&self) -> Option<&'static str> { Some("//") }
    fn block_comment(&self) -> Option<(&'static str, &'static str)> { Some(("/*", "*/")) }
    fn import_keywords(&self) -> &'static [&'static str] { &["import"] }
    
    // Default thresholds for Go (optional override)
    fn default_thresholds(&self) -> crate::Thresholds {
        crate::Thresholds {
            max_lines: 400,
            max_imports: 20,
            ..Default::default()
        }
    }
}
```

### 2. Handling Complex Syntax (e.g., Block Imports)
If a language has non-standard syntax that the default keyword-based counter cannot handle, you can override the trait methods directly.

For example, Go's `import (...)` blocks:

```rust
impl Language for Go {
    // ...
    fn count_imports(&self, content: &str) -> usize {
        // Implement custom logic to count package declarations 
        // inside both single-line and block imports.
        let mut count = 0;
        let mut in_block = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import (") { in_block = true; continue; }
            if in_block && trimmed == ")" { in_block = false; continue; }
            if in_block && !trimmed.is_empty() { count += 1; }
            if !in_block && trimmed.starts_with("import \"") { count += 1; }
        }
        count
    }
}
```

### 3. Registration
1.  **Definitions**: Add `pub mod go;` to `src/languages/definitions/mod.rs`.
2.  **Registry**: Add `Box::new(definitions::go::Go)` to `LanguageRegistry::new()` in `src/languages/mod.rs`.
3.  **Config**: Add the extension (`go`) to `ThresholdsOverrides` in `src/config/thresholds.rs` and update its `get()` and `extend()` methods.

### 4. VS Code Integration
1.  **`package.json`**: Add the language to `activationEvents` and the `languages` contribution point.
2.  **`extension.ts`**: Add the language ID to the `supportedLanguages` array.

---

## 🧪 Quality Standards

We enforce strict engineering standards to keep Sweet "Sweet":

- **No Panics**: Use `Result` and `Option` handling. Avoid `unwrap()` or `expect()` in production code (enforced by Clippy).
- **Performance**: Analysis must remain O(n). Avoid complex regex in hot paths; prefer string slices and iterators.
- **Validation**: Run `./hooks/pre-push` before submitting. It executes:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`

---

## 📜 License
By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).

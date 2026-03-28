# 🍬 Contributing to Sweet

Thank you for your interest in improving Sweet! This guide will help you understand the project structure and how to contribute effectively, specifically when adding support for new programming languages.

---

## 🏗️ Project Structure

- **`src/`**: The Rust core.
  - **`analyzer/`**: Logic for metrics (complexity, repetition, etc.).
  - **`languages/`**: Language definitions and registry.
  - **`config/`**: Configuration resolution and thresholds.
  - **`bin/lsp.rs`**: The Language Server Protocol implementation.
  - **`main.rs`**: The `swt` CLI entry point.
- **`editors/vscode/`**: The Visual Studio Code extension (TypeScript).
- **`schema.json`**: The JSON schema for `.swtrc` (auto-generated).

---

## 🚀 Adding a New Language

To add support for a new language (e.g., **JavaScript**), follow these steps:

### 1. Define the Language Rules
Create a new file in `src/languages/definitions/`. For example, `src/languages/definitions/javascript.rs`:

```rust
use crate::languages::Language;

pub struct JavaScript;

impl Language for JavaScript {
    fn name(&self) -> &'static str { "JavaScript" }
    fn extensions(&self) -> &'static [&'static str] { &["js", "mjs", "cjs"] }
    fn import_keywords(&self) -> &'static [&'static str] { &["import", "require"] }
    fn line_comment_prefix(&self) -> &'static str { "//" }
    fn block_comment_delimiters(&self) -> (&'static str, &'static str) { ("/*", "*/") }
    fn indent_size(&self) -> usize { 2 }
}
```

### 2. Register the Language
1.  Add the module to `src/languages/definitions/mod.rs`:
    ```rust
    pub mod javascript;
    ```
2.  Register it in the `LanguageRegistry` in `src/languages/mod.rs`:
    ```rust
    // Inside LanguageRegistry::new()
    let mut languages: Vec<Box<dyn Language>> = vec![
        // ...
        Box::new(definitions::javascript::JavaScript),
    ];
    ```

### 3. Update Configuration Autocomplete
To enable autocompletion for the new language in `.swtrc` files:

1.  Open `src/config/thresholds.rs`.
2.  Add the language to the `ThresholdsOverrides` struct:
    ```rust
    pub struct ThresholdsOverrides {
        // ...
        pub js: Option<PartialThresholds>, // Matches the field name in JSON
        #[serde(flatten)]
        pub custom: HashMap<String, PartialThresholds>,
    }
    ```
3.  Update the `get()` and `extend()` methods in the same file to handle the new field.

### 4. Update the VS Code Extension
1.  **`package.json`**: Add the language to `activationEvents`:
    ```json
    "activationEvents": [
      "onLanguage:javascript",
      // ...
    ]
    ```
2.  **`src/extension.ts`**: Add the language to the `documentSelector`:
    ```typescript
    documentSelector: [
      { scheme: 'file', language: 'javascript' },
      // ...
    ]
    ```

### 5. Finalize and Verify
Run the build-all script to regenerate the schema and verify the changes:

```bash
./build-all.sh
```

- Update the **Supported Languages** table in `README.md`.
- Ensure all tests pass.

---

## 🧪 Testing

We value high-quality code. Ensure that:
- All Rust tests pass: `cargo test --all-features`.
- Clippy is happy: `cargo clippy --all-targets --all-features -- -D warnings`.
- Code is formatted: `cargo fmt -- --check`.

The `./hooks/pre-push` script runs these checks automatically. Please run it before submitting a Pull Request.

---

## 📜 License
By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).

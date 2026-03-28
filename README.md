# 🍬 Sweet (swt)

[![Crates.io](https://img.shields.io/crates/v/swt.svg)](https://crates.io/crates/swt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/SirCesarium/sweet/workflows/CI/badge.svg)](https://github.com/SirCesarium/sweet/actions)

**Turn code maintainability into a measurable metric.**

`Sweet` is a blazing-fast code health analyzer designed to keep project architectures lean and sustainable. It identifies technical debt, tangled dependencies, and complex logic patterns.

---

## 🍭 The Sweet Index

`Sweet` evaluates source files against configurable health thresholds.

| Status | Meaning | Action |
| :--- | :--- | :--- |
| **Sweet** 🍭 | Balanced, cohesive, and easy to maintain. | Keep it up! |
| **Bitter** 🍋 | Overly complex, high coupling, or high repetition. | Refactor recommended. |

---

## ✨ Key Features

- **🚀 Blazing Fast:** Process thousands of files in milliseconds (e.g., self-analysis in <10ms).
- **📁 Hierarchical Config:** Support for multiple `.swtrc` files to define specific rules for different subdirectories.
- **🔍 Global Inspection:** Detect code duplication across the entire project with detailed reporting.
- **🛡️ Quality Guard:** Built-in support for git hooks to prevent "Bitter" code from being pushed.
- **🧹 Source Cleanup:** Professional comment stripping and whitespace normalization.

---

## 📊 Supported Languages

| Language | Extension | Import Style | Comment Style |
| :--- | :--- | :--- | :--- |
| **Rust** | `.rs` | `use` | `//`, `/* */` |
| **Python** | `.py` | `import`, `from` | `#` |
| **JavaScript** | `.js`, `.mjs`, `.cjs` | `import`, `require` | `//`, `/* */` |
| **TypeScript** | `.ts`, `.tsx` | `import` | `//`, `/* */` |
| **Java** | `.java` | `import` | `//`, `/* */` |
| **C#** | `.cs` | `using` | `//`, `/* */` |

---

## 🛠️ Installation

```bash
cargo install swt
```

---

## 📖 Usage

### Analyze Project
```bash
swt .
```

### Global Inspection
Show exact code fragments repeated across different files:
```bash
swt . --inspect
```

### Strip Comments
```bash
swt --uncomment src/lib.rs --aggressive
```

---

## ⚙️ Configuration

`Sweet` resolves `.swtrc` files hierarchically.

```json
{
  "$schema": "https://raw.githubusercontent.com/SirCesarium/sweet/main/schema.json",
  "thresholds": {
    "global": { 
      "max_lines": 250, 
      "max_depth": 5, 
      "max_repetition": 10.0,
      "min_duplicate_lines": 4
    },
    "overrides": {
      "rust": { "max_imports": 15 }
    }
  }
}
```

---

## 📜 License

Licensed under the [MIT License](./LICENSE).

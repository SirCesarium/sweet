# 🍬 swt (Sweet)

**Turn code maintainability into a measurable metric.**

`swt` is a high-performance analyzer designed to keep your project's architecture clean. It scans your codebase to identify sustainability risks, tangled dependencies, and complex logic patterns that hinder long-term development.

## 🍭 Sweet Index

`swt` evaluates your files based on a health threshold. It's not just about finding bugs; it's about identifying code that is becoming a burden to your team.

| Status | Meaning | Action |
| :--- | :--- | :--- |
| **Sweet** 🍭 | Balanced, cohesive, and easy to test. | Keep it up! |
| **Bitter** 🍋 | Overly complex, high coupling, or "God File" patterns. | Needs refactoring. |

---

## Key Features

*   **Sustainability Audits:** Automatically find files that have grown beyond manageable limits.
*   **Decoupling Tracking:** Detect excessive dependencies and imports that make code hard to isolate.
*   **Logic Simplification:** Spot deeply nested functions and "God-logic" blocks before they become technical debt.
*   **Source Cleanup:** A precision tool to strip comments and normalize whitespace for cleaner source distribution.
*   **Multilingual Support:** Native understanding of Rust, TypeScript, JavaScript, Java, C#, and Python.
*   **Automation Ready:** Designed for CI/CD with dedicated JSON reporting and minimalist output modes.

## Performance

`swt` is engineered for instant feedback. It processes thousands of files in milliseconds, making it ideal for large monorepos and pre-commit hooks.

### Benchmarks (current project)
```bash
Benchmark 1: swt .
  Time (mean ± σ):       4.2 ms ±   0.6 ms
  Range (min … max):     3.2 ms …   6.9 ms
```

### Scalability at 13,000+ files
*   **Execution Time:** ~744ms
*   **Processing Rate:** ~18,000 files/sec

## Installation

Install the binary via Cargo:

```bash
cargo install swt
```

## Usage

### Analyze Project
Scan the current directory and list files by maintenance priority:
```bash
swt .
```

### Strip Comments
Clean a source file by removing comments and normalizing whitespace:
```bash
swt --uncomment src/main.rs
```
*Use `--aggressive` to also remove documentation headers* (like Rust Doc comments `///` or JSDoc `/** */`)

### Automated Reporting
Export metrics to JSON for integration with other tools:
```bash
swt --json

# You can also generate a JSON file:
swt --json reports.json
```

## Configuration

Control the health thresholds using a `.swtrc` file in your project root:

```json
{
  "$schema": "https://raw.githubusercontent.com/SirCesarium/sweet/main/schema.json",
  "thresholds": {
    "global": { "max_lines": 200, "max_depth": 5 },
    "overrides": {
      "rust": { "max_imports": 15 },
      "java": { "max_imports": 50 }
    }
  }
}
```

## License

This project is licensed under the [MIT License](./LICENSE).

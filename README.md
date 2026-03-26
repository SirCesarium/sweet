# 🍬 swt (Sweet)

**Architectural health and maintainability analyzer.**

`swt` is a high-performance tool designed to help developers monitor code quality and project structure. It identifies complex files, tangled dependencies, and deeply nested logic, providing actionable insights into your project's maintainability.

## Features

- **Project Health Analysis:** Quickly scan directories to find files that have become too large or difficult to maintain.
- **Dependency Tracking:** Identify files with excessive imports or high coupling across multiple languages.
- **Complexity Detection:** Spot "God functions" and overly complex logic through deep nesting analysis.
- **Comment Stripping:** Remove comments and trailing whitespace from source files while optionally preserving documentation.
- **Broad Language Support:** Native support for Rust, TypeScript, JavaScript, Java, C#, and Python.
- **Flexible Configuration:** Define custom quality thresholds globally or per language using a simple configuration file.
- **CI/CD Integration:** Automated reporting via JSON and a clean, minimalist output mode for build pipelines.

## Benchmarks

`swt` provides near-instant analysis even for very large projects.

### Local Performance
Results for the current project:

```bash
Benchmark 1: swt .
  Time (mean ± σ):       4.2 ms ±   0.6 ms
  Range (min … max):     3.2 ms …   6.9 ms
```

### Scalability
Analysis of a system with over 13,000 source files:
- **Execution Time:** ~744ms
- **Processing Rate:** ~18,000 files/sec

## Installation

Install the binary via Cargo:

```bash
cargo install swt
```

## Usage

### Scan Project
Analyze the current directory and sort files by maintenance priority:
```bash
swt .
```

### Remove Comments
Clean a source file by stripping comments:
```bash
swt --uncomment src/main.rs
```
*Use `--aggressive` to also remove documentation comments.*

### Export Data
Generate a JSON report for external processing:
```bash
swt --json reports.json
```

## Configuration

Customize health thresholds via a `.swtrc` file:

```json
{
  "$schema": "https://raw.githubusercontent.com/SirCesarium/sweet/main/schema.json",
  "thresholds": {
    "global": { "max_lines": 200, "max_depth": 5 },
    "overrides": {
      "java": { "max_imports": 50 },
      "rust": { "max_imports": 15 }
    }
  }
}
```

## License

This project is licensed under the MIT License.

<p align="center">
  <img src="editors/vscode/icon.svg" width="128" alt="Sweet Icon">
</p>

<p align="center">
  <a href="https://crates.io/crates/swt"><img src="https://img.shields.io/crates/v/swt.svg" alt="Crates.io"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
  <a href="https://github.com/SirCesarium/sweet/actions"><img src="https://github.com/SirCesarium/sweet/workflows/CI/badge.svg" alt="Build Status"></a>
</p>

<h1 align="center">🍬 Sweet (swt)</h1>

<p align="center">
  The quality gate for sustainable software architecture.
</p>

`Sweet` is a high-performance code health and architectural integrity analyzer. It is **plug-and-play**: it works immediately with zero configuration using intelligent defaults, while offering the flexibility to enforce stricter standards via `.swtrc` files.

By quantifying technical debt and identifying complex logic patterns, `Sweet` helps teams adhere to core engineering principles like **SRP** (Single Responsibility Principle) and **DRY** (Don't Repeat Yourself).

## 🍬 Why Sweet?

Most linters focus on syntax; `Sweet` focuses on **maintainability**. It acts as a surgical tool to prevent the "Big Ball of Mud" anti-pattern by monitoring the physical and logical weight of your components.

*   **Maintain Maintainability:** Identify bloated files and excessive nesting that make code hard to reason about.
*   **Encourage Decoupling:** Track dependency density to prevent tangled, hard-to-test modules.
*   **Prevent Logic Bloat:** Detect deep nesting and cognitive complexity before they become technical debt.
*   **Eliminate Redundancy:** Project-wide inspection to find duplicated logic that should be abstract or shared.

## 🍬 Metrics

`Sweet` evaluates code health through four primary lenses of maintainability:

| Metric | Goal | Engineering Impact |
| :--- | :--- | :--- |
| **Physical Weight** | `max_lines` | Prevents bloated files and encourages decomposition. |
| **Control Flow** | `max_depth` | Flags excessive nesting to keep logic readable. |
| **Coupling** | `max_imports` | Monitors dependency growth to prevent tangled architectures. |
| **Repetition** | `max_repetition` | Identifies violations of the **DRY** principle. |

## 🍬 Features

- **Blazing Fast:** Process thousands of files in seconds (scans the Linux Kernel in ~8.2s).
- **Zero-Copy Architecture:** Single-pass byte-level scanner for maximum efficiency.
- **Hierarchical Config:** Cascading `.swtrc` files for directory-specific rule sets.
- **Global Inspection:** Project-wide duplicate detection with detailed occurrence mapping.
- **Quality Guard:** Native support for pre-push hooks to block "Bitter" code.
- **Auto-Update:** Built-in update system to keep your tool always sharp.

### 🍭 Supported Languages

| Language | Status | Extension |
| :--- | :---: | :--- |
| **Rust** | ✅ | `.rs` |
| **Python** | ✅ | `.py` |
| **JavaScript** | ✅ | `.js`, `.mjs`, `.cjs`, `.jsx` |
| **TypeScript** | ✅ | `.ts`, `.tsx` |
| **Java** | ✅ | `.java` |
| **C\#** | ✅ | `.cs` |
| **GDScript** | ✅ | `.gd` |
| **Lua** | ✅ | `.lua` |
| **Go** | ✅ | `.go` |
| **PHP** | ✅ | `.php` |
| **C/C++** | ✅ | `.c`, `.cpp`, `.h`, `.hpp`, `.cc`, `.cxx` |

## 🍬 Installation

Visit the [Releases Page](https://github.com/SirCesarium/sweet/releases) for native installers:
- **Windows**: `.msi`
- **Linux**: `.deb`, `.rpm`

### Crates.io
```bash
cargo install swt
```

## 📖 Usage

Run a standard health check:
```bash
swt [path]
```

### Deep Inspection
Find exact code fragments repeated across different files:
```bash
swt inspect [path]
```

### Strip Comments
AI Agents often generate verbose comments. Reclaim your logic:
```bash
swt uncomment <file> [--aggressive]
```

### Update
Check or install new versions:
```bash
swt check-updates
swt update
```

### 🔌 Power User Integration

`Sweet` follows the Unix philosophy. It plays perfectly with the standard Rust toolbelt ([`fd`](https://github.com/sharkdp/fd), [`rg`](https://github.com/burntsushi/ripgrep)) to handle massive refactors in seconds.

**With [`fd` (Fast Find)](https://github.com/sharkdp/fd):**
Strip comments from every Rust file in your project at once.

```bash
fd -e rs -x swt uncomment
```

**With [`ripgrep` (rg)](https://github.com/burntsushi/ripgrep):**
Target only the files that contain a specific "Bitter" pattern or AI-generated signature.

```bash
rg "TODO:" -l | xargs swt uncomment
```

## 🏗️ CI/CD Integration

If you are using **GitHub Actions**, you don't need to manually install `Sweet`. We've built **[Refinery-RS](https://github.com/SirCesarium/refinery-rs)**, a surgical quality gate and build pipeline for Rust projects.

### [Refinery-RS CI](https://github.com/SirCesarium/refinery-rs)

Integrate `Sweet` with `clippy` and `rustfmt` in one single step:

```yaml
jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: sircesarium/refinery-rs/ci@main
        with:
          enable-sweet: true   # Runs 'swt' maintainability analysis
          enable-clippy: true  # Runs standard Rust lints
          enable-fmt: true     # Ensures consistent formatting
```

## ⚙️ Configuration

`Sweet` resolves `.swtrc` files hierarchically.

```json
{
  "$schema": "https://raw.githubusercontent.com/SirCesarium/sweet/main/schema.json",
  "thresholds": {
    "global": { 
      "max_lines": 400, 
      "max_depth": 5
    },
    "severities": {
      "max-repetition": "warning"
    }
  }
}
```

### 🍭 Severity Levels

By default, all rule violations are treated as **errors** (causing a non-zero exit code). You can downgrade specific rules to **warnings** (informational only, exit code 0) in your `.swtrc`.

### 🍭 In-file Control

Disable specific checks via comments in the first 20 lines:
`// @swt-disable max-lines max-repetition`

**Rules:** `max-lines`, `max-depth`, `max-imports`, `max-repetition`.

To ignore a file entirely, use `@sweetignore`.

### ⚡ Benches

To demonstrate Sweet's performance, we benchmarked it against established tools on the entire Linux Kernel source tree (~64k files, 8.0 GB):

| Tool       | Language | Primary Focus                    | Time (Linux Kernel) | Analysis Depth                                                 | Speed vs Sweet |
| :--------- | :------- | :------------------------------- | :------------------ | :------------------------------------------------------------- | :------------- |
| **Sweet (swt)** | **Rust** | **Arch. Health & Duplication** | **~8.2s**           | **Lines, Imports, Nesting, Duplication, Thresholds**           | **N/A** |
| **Tokei**  | Rust     | Raw Line Counting                | **~2.1s**           | Lines, Comments, Blanks                                        | **3.9x faster** |
| **cloc**   | Perl     | Line Count (Industry Standard)   | **~150s**           | Lines, Comments, Blanks, Language Breakdown                    | **18.3x slower** |
| **Lizard** | Python   | Cyclomatic Complexity            | **~456s**           | Nesting Depth, Function Count                                  | **55.6x slower** |

**Key Takeaways:**
- Sweet provides **high-fidelity architectural insights** (nesting depth, import density, project-wide duplication) at speeds approaching raw data counters.
- The engine is designed to handle massive codebases in seconds, making it suitable for real-time quality gates in large-scale industrial environments.

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for technical specifications and how to add new languages.

## 📜 License

Licensed under the [MIT License](./LICENSE).

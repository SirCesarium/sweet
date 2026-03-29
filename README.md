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
  <strong>The quality gate for sustainable software architecture.</strong>
</p>

`Sweet` is a high-performance code health analyzer designed to enforce architectural integrity. It is **plug-and-play**: it works immediately with zero configuration using intelligent defaults, while offering the flexibility to enforce stricter standards via `.swtrc` files. By quantifying technical debt and identifying complex logic patterns, it helps teams adhere to core engineering principles like **SRP** (Single Responsibility Principle) and **DRY** (Don't Repeat Yourself).

## 🍬 Why Sweet?

Most linters focus on syntax; `Sweet` focuses on **maintainability**. It acts as a surgical tool to prevent the "Big Ball of Mud" anti-pattern by monitoring the physical and logical weight of your components.

*   **Enforce SRP:** Identify "God Functions" and bloated files that take on too many responsibilities.
*   **Encourage Decoupling:** Track dependency density to prevent tangled, hard-to-test modules.
*   **Prevent Logic Bloat:** Detect deep nesting and cognitive complexity before they become technical debt.
*   **Eliminate Redundancy:** Project-wide inspection to find duplicated logic that should be abstract or shared.

## 🍬 Metrics

`Sweet` evaluates code health through four primary lenses of maintainability:

| Metric | Goal | Engineering Impact |
| :--- | :--- | :--- |
| **Physical Weight** | `max_lines` | Prevents bloated files and encourages decomposition. |
| **Logic Density** | `max_lines_per_function` | Enforces **SRP** by identifying "God Functions" that do too much. |
| **Control Flow** | `max_depth` | Flags excessive nesting to keep logic readable and testable. |
| **Coupling** | `max_imports` | Monitors dependency growth to prevent tangled architectures. |
| **Repetition** | `max_repetition` | Identifies violations of the **DRY** principle. |

## 🍬 Features

- **Blazing Fast:** Process thousands of files in milliseconds (self-analysis in <10ms).
- **Hierarchical Config:** Cascading `.swtrc` files for directory-specific rule sets.
- **Global Inspection:** Project-wide duplicate detection with detailed occurrence mapping.
- **Intelligent Defaults:** Language-specific thresholds tuned for different ecosystems (e.g., higher line limits for Java/C# vs. Rust).
- **Quality Guard:** Native support for pre-push hooks to block "Bitter" code from reaching production.

### 🍭 Supported Languages

| Language | Status | Extension |
| :--- | :---: | :--- |
| **Rust** | ✅ | `.rs` |
| **Python** | ✅ | `.py` |
| **JavaScript** | ✅ | `.js`, `.mjs`, `.cjs` |
| **TypeScript** | ✅ | `.ts`, `.tsx` |
| **Java** | ✅ | `.java` |
| **C\#** | ✅ | `.cs` |
| **GDScript** | ✅ | `.gd` |
| **Lua** | ✅ | `.lua` |
| **Go** | ✅ | `.go` |
| **PHP** | ✅ | `.php` |

Don't see your favorite language? `Sweet` is designed to be extensible. If you want to add support for a new language (like Go, C++, or Swift), we’d love your help\!

Check out our [Contributing Guide](./CONTRIBUTING.md) to see how easy it is to implement a new language provider.

## 🍬 Installation

[Click here](https://github.com/SirCesarium/sweet/releases) to get into the releases page!

### You can also install `sweet` from [crates.io](https://crates.io)

```bash
cargo install swt
```

### Compiling by yourself

- Clone the project `git clone https://github.com/SirCesarium/sweet.git`.
- Build using `cargo`: `cargo build --release`.
- Check the `target/release/` folder.

## 📖 Usage

```bash
swt # or specify the project path using `swt path/to/project`
```

### Copy-Paste Inspector

Show exact code fragments repeated across different files:

```bash
swt . --inspect
```
### Strip Comments

AI Agents and LLMs often generate verbose, redundant comments that clutter your codebase. Use the `--uncomment` flag to strip the noise and reclaim your screen real estate for what matters: **the logic.**

> [!TIP]
> By default, `Sweet` preserves your **documentation comments** (e.g., `//!`, `///`, or `/** */`). Use the `--aggressive` flag if you want a truly blank slate.

```bash
# Clean a single file while keeping documentation
swt --uncomment src/lib.rs

# Full cleanup (removes everything including docs)
swt --uncomment --aggressive src/lib.rs
```

### 🔌 Power User Integration

`Sweet` follows the Unix philosophy. It plays perfectly with the standard Rust toolbelt ([`fd`](https://github.com/sharkdp/fd), [`rg`](https://github.com/burntsushi/ripgrep)) to handle massive refactors in seconds.

**With [`fd` (Fast Find)](https://github.com/sharkdp/fd):**
Strip comments from every Rust file in your project at once.

```bash
fd -e rs -x swt --uncomment
```

**With [`ripgrep` (rg)](https://github.com/burntsushi/ripgrep):**
Target only the files that contain a specific "Bitter" pattern or AI-generated signature.

```bash
rg "TODO:" -l | xargs swt --uncomment
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

### Why use Refinery?

  * **Zero Setup:** No need to `cargo install swt` in every CI run; Refinery handles the caching and environment for you.
  * **Fail-Fast:** Automatically blocks "Bitter" code from being merged.
  * **One-Stop Shop:** If you need to ship, use the same suite to build multi-target binaries and push Docker images to GHCR.

## ⚙️ Configuration

`Sweet` resolves `.swtrc` files hierarchically, merging configurations from the file's directory up to the root.

```json
{
  "$schema": "https://raw.githubusercontent.com/SirCesarium/sweet/main/schema.json",
  "thresholds": {
    "global": { 
      "max_lines": 400, 
      "max_depth": 6, 
      "max_repetition": 15.0,
      "max_lines_per_function": 200
    },
    "overrides": {
      "rust": { "max_imports": 30 },
      "gdscript": { "max_depth": 7 }
    }
  }
}
```

### 🍭 In-file Control

You can granularly disable specific checks for a single file using comments in the first 20 lines. This is ideal for legacy codebases or generated assets.

Add `@swt-disable <rule1> <rule2>`:

```rust
// @swt-disable max-lines max-repetition
```

**Supported rules:** `max-lines`, `max-depth`, `max-imports`, `max-repetition`, `max-lines-per-function`.

To ignore a file entirely, use the `@sweetignore` directive.

## 🤝 Contributing

Contributions are welcome! Whether it's adding support for a new language, fixing a bug, or improving the documentation, please check our [Contributing Guide](./CONTRIBUTING.md) to get started.

## 📜 License

Licensed under the [MIT License](./LICENSE).

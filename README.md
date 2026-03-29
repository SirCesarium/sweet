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
  <strong>Turn code maintainability into a measurable metric.</strong>
</p>

`Sweet` is a blazing-fast code health analyzer designed to keep project architectures lean and sustainable. It identifies technical debt, tangled dependencies, and complex logic patterns.

## 🍬 Metrics

| Status | Meaning | Action |
| :--- | :--- | :--- |
| **Sweet** 🍭 | Balanced, cohesive, and easy to maintain. | Keep it up! |
| **Bitter** 🍋 | Overly complex, high coupling, or high repetition. | Refactor recommended. |

## 🍬 Features

- **Blazing Fast:** Process thousands of files in milliseconds (e.g., self-analysis in <10ms).
- **Hierarchical Config:** Support for multiple `.swtrc` files to define specific rules for different subdirectories.
- **Global Inspection:** Detect code duplication across the entire project with detailed reporting.
- **Quality Guard:** Built-in support for git hooks to prevent "Bitter" code from being pushed.

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
| **Go** | ⏳ *Planned* | `.go` |

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

## 🤝 Contributing

Contributions are welcome! Whether it's adding support for a new language, fixing a bug, or improving the documentation, please check our [Contributing Guide](./CONTRIBUTING.md) to get started.

## 📜 License

Licensed under the [MIT License](./LICENSE).

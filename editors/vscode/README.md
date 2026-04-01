<p align="center">
<img src="https://raw.githubusercontent.com/SirCesarium/sweet/main/editors/vscode/icon.png" width="128" alt="Sweet Icon">
</p>

<h1 align="center">🍬 Sweet for Visual Studio Code</h1>

<p align="center">
<strong>Real-time code health alerts for professional developers.</strong>
</p>

`Sweet` is a blazing-fast code health analyzer designed to keep project architectures lean and sustainable. It provides real-time feedback on technical debt, logic density, and structural integrity directly in your editor.

---

## 🚀 Getting Started

1. **Install the extension.**
2. **Activate it**: Create a `.swtrc` file (even an empty `{}`) in your project root.
3. **Enjoy**: Sweet will immediately start analyzing your code.

_Note: You can change this behavior to "Always On" in the VS Code settings (`sweet.enabledMode`)._

---

## 🍬 Features

- **Structural Guard:** Real-time analysis of file weight and logic density.
- **Logical Highlighting:** Immediate visual feedback for excessive nesting and cognitive complexity.
- **Repetition Alerts:** Identifies duplicated code blocks with links to other occurrences.
- **Quick Fix Integration:** Use `Ctrl+.` to granularly disable rules for specific files via `@swt-disable`.
- **Deep Integration:** Full support for hierarchical `.swtrc` configuration files.
- **Zero Latency:** Rust-powered core ensures a smooth typing experience without editor lag.

---

## 🍭 How it works

The extension provides live structural diagnostics through **VS Code Warnings** (yellow squiggles) for:

1. **File Bloat:** Total line count exceeds thresholds.
3. **Logical Depth:** Deeply nested control flow (cognitive complexity).
4. **Tangled Coupling:** Excessive import/dependency statements.
5. **Logic Duplication:** Repeated code blocks (local or project-wide).

---

## ⚙️ Configuration

`Sweet` automatically detects `.swtrc` files in your workspace. If absent, it applies intelligent defaults tuned for each supported language (Rust, Python, GDScript, Lua, etc.).

To enable **Project-Wide Duplication Analysis**, add this to your `.swtrc`:

```json
{
  "cross_file_repetition": true,
  "thresholds": {
    "global": {
      "max_repetition": 25
    }
  }
}
```

You can customize the importance of each rule in your `.swtrc`:

```json
{
  "thresholds": {
    "global": {
      "max_lines": 400,
      "max_depth": 5
    },
    "severities": {
      "max-repetition": "warning",
      "max-depth": "error"
    }
  }
}
```

### 🍭 In-file Disabling

Use `Ctrl+.` (Quick Fix) or manually add a comment to disable specific rules:
`@swt-disable max-lines max-repetition max-depth max-imports`

---

## 🤝 Contributing

This extension is part of the [Sweet Ecosystem](https://github.com/SirCesarium/sweet). If you want to improve the VSCode integration or add support for more languages, feel free to open an issue or PR in the main repository.

**Happy coding\! Stay Sweet. 🍭**

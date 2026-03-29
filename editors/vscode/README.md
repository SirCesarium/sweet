<p align="center">
<img src="https://raw.githubusercontent.com/SirCesarium/sweet/main/editors/vscode/icon.png" width="128" alt="Sweet Icon">
</p>

<h1 align="center">🍬 Sweet for Visual Studio Code</h1>

<p align="center">
<strong>Real-time code health alerts for professional developers.</strong>
</p>

`Sweet` is a blazing-fast code health analyzer designed to keep project architectures lean and sustainable. It is **plug-and-play**: it works immediately with zero configuration using intelligent defaults, while offering the flexibility to enforce stricter standards via `.swtrc` files.

-----

## 🍬 Features

  - **Structural Guard:** Real-time analysis of file weight and logic density.
  - **Logical Highlighting:** Immediate visual feedback for excessive nesting and cognitive complexity.
  - **Repetition Alerts:** Identifies duplicated code blocks with links to other occurrences.
  - **Quick Fix Integration:** Use `Ctrl+.` to granularly disable rules for specific files via `@swt-disable`.
  - **Deep Integration:** Full support for hierarchical `.swtrc` configuration files.
  - **Zero Latency:** Rust-powered core ensures a smooth typing experience without editor lag.

-----

## 🍭 How it works

The extension provides live structural diagnostics through **VS Code Warnings** (yellow squiggles) for:

1.  **File Bloat:** Total line count exceeds thresholds.
2.  **God Functions:** High average lines-per-function (Single Responsibility Principle).
3.  **Logical Depth:** Deeply nested control flow (cognitive complexity).
4.  **Tangled Coupling:** Excessive import/dependency statements.
5.  **Local Repetition:** Duplicated logic within the same file.

-----

## ⚙️ Configuration

`Sweet` automatically detects `.swtrc` files in your workspace. If absent, it applies intelligent defaults tuned for each supported language (Rust, Python, GDScript, Lua, etc.).

You can customize the importance of each rule in your `.swtrc`:

```json
{
  "thresholds": {
    "global": { 
      "max_lines": 400,
      "max_lines_per_function": 200
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
`@swt-disable max-lines max-repetition max-depth max-imports max-lines-per-function`

-----

## 🤝 Contributing

This extension is part of the [Sweet Ecosystem](https://github.com/SirCesarium/sweet). If you want to improve the VSCode integration or add support for more languages, feel free to open an issue or PR in the main repository.

**Happy coding\! Stay Sweet. 🍭**

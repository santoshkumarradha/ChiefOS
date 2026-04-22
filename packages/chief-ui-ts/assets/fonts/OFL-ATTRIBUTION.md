# Bundled Font Attribution

Chief UI bundles font files at package build time so Tauri surfaces do not load
web fonts at runtime.

| Font | Role | License |
|---|---|---|
| Inter Tight | Display | SIL Open Font License 1.1 |
| Inter | Body/UI | SIL Open Font License 1.1 |
| JetBrains Mono | Mono/data | SIL Open Font License 1.1 |

The source CSS references:

- `InterTight-Regular.ttf`
- `Inter-Regular.ttf`
- `JetBrainsMono-Regular.ttf`

These files are shipped from `assets/fonts/` and loaded by `src/tokens.css`.

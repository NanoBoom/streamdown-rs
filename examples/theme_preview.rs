//! 主题调试工具：从文件加载 style.toml + syntax.bin，渲染示例 markdown
//!
//! 用法:
//!   cargo run --example theme_preview -- <theme_dir> [markdown_file]
//!
//! 示例:
//!   cargo run --example theme_preview -- /Users/you/.config/aichat/themes/dark
//!   cargo run --example theme_preview -- ../aichat/themes/light README.md

use std::path::Path;
use streamdown_parser::Parser;
use streamdown_render::{load_theme_from_file, RenderStyle, Renderer};

const SAMPLE: &str = r#"# Heading 1
## Heading 2
### Heading 3

Normal text with **bold**, *italic*, and `inline code`.

> Blockquote text here.

- Bullet item one
- Bullet item two

```rust
fn main() {
    println!("Hello, theme!");
}
```

| Column A | Column B |
|----------|----------|
| cell 1   | cell 2   |

[Link text](https://example.com)

---
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let theme_dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let theme_dir = Path::new(theme_dir);

    // 加载 markdown 内容（可选，默认使用内置示例）
    let sample_owned;
    let content = if let Some(md_path) = args.get(2) {
        sample_owned = std::fs::read_to_string(md_path)
            .unwrap_or_else(|e| panic!("Failed to read markdown file '{}': {e}", md_path));
        eprintln!("Loaded markdown: {}", md_path);
        sample_owned.as_str()
    } else {
        SAMPLE
    };

    // 加载 style.toml（可选）
    let style_path = theme_dir.join("style.toml");
    let style = if style_path.exists() {
        let content = std::fs::read_to_string(&style_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", style_path.display()));
        let style: RenderStyle = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("Invalid style.toml: {e}"));
        eprintln!("Loaded style: {}", style_path.display());
        Some(style)
    } else {
        eprintln!("No style.toml found, using defaults");
        None
    };

    // 加载 syntax.bin（可选）
    let syntax_path = theme_dir.join("syntax.bin");
    let syntax_theme = if syntax_path.exists() {
        let theme = load_theme_from_file(&syntax_path)
            .unwrap_or_else(|e| panic!("Failed to load {}: {e}", syntax_path.display()));
        eprintln!("Loaded syntax theme: {}", syntax_path.display());
        Some(theme)
    } else {
        eprintln!("No syntax.bin found, using default syntax theme");
        None
    };

    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80);

    let mut output = Vec::new();
    let mut parser = Parser::new();

    {
        let mut renderer = match style {
            Some(s) => Renderer::with_style(&mut output, width, s),
            None => Renderer::new(&mut output, width),
        };
        if let Some(theme) = syntax_theme {
            renderer.set_custom_theme(theme);
        }
        for line in content.lines() {
            let events = parser.parse_line(line);
            for event in events {
                renderer.render_event(&event).unwrap();
            }
        }
    }

    print!("{}", String::from_utf8(output).unwrap());
}

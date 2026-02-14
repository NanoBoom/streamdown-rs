# Streamdown-rs 🦀

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/vd9KdeVT)

A streaming markdown renderer for modern terminals, written in Rust.

This is a Rust port of [Streamdown](https://github.com/kristopolous/Streamdown) by kristopolous, bringing the same beautiful terminal markdown rendering with the performance and safety of Rust.

## ✨ Features

- **Streaming rendering** - Renders markdown as it arrives, perfect for LLM output
- **Stateful syntax highlighting** - Accurate multi-line token highlighting via [syntect](https://github.com/trishume/syntect) with persistent parse state
- **Beautiful output** - Pretty tables, lists, code blocks with box drawing
- **Think blocks** - Renders LLM `<think>` blocks with distinct styling, including full code block support inside think blocks
- **Persistent render state** - Multi-line constructs (blockquotes, lists, tables, code blocks) render correctly across streaming chunks
- **Custom themes** - Pass syntect `Theme` objects directly or use built-in themes; control highlight background independently
- **PTY exec mode** - Run commands and render their markdown output
- **Clipboard integration** - Copy code blocks via OSC 52
- **Savebrace** - Save code blocks to temp files for shell access
- **LaTeX support** - Convert LaTeX math to Unicode symbols with `ProcessResult::Rewrite` preserving inline markdown formatting
- **Configurable** - TOML configuration for colors and behavior
- **Cross-platform** - Full Unix support, partial Windows support
- **Cloneable parser** - `Parser`, `InlineParser`, and `Tokenizer` implement `Clone` for state snapshotting

## 📦 Installation

### From source

```bash
git clone https://github.com/fed-stew/streamdown-rs.git
cd streamdown-rs
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install streamdown
```

## 🚀 Quick Start

### Pipe markdown content

```bash
# Pipe from any command
echo "# Hello World" | sd

# Render a file
cat README.md | sd

# From an LLM
curl -s https://api.openai.com/... | sd
```

### Render a file directly

```bash
sd document.md
```

### Execute a command and render output

```bash
# Run a command in a PTY and render its markdown output
sd --exec "python chat.py"
sd -e "llm query 'explain rust lifetimes'"
```

## 📖 Usage

```
Streamdown - A streaming markdown renderer for modern terminals.

Usage: sd [OPTIONS] [FILE]...

Arguments:
  [FILE]...              Input files to process (reads from stdin if not provided)

Options:
  -l, --loglevel <LOG_LEVEL>  Set the logging level (trace, debug, info, warn, error) [default: warn]
  -b, --base <BASE>           Set the HSV base color: h,s,v (e.g., "0.6,0.5,0.5")
  -c, --config <CONFIG>       Use a custom config file or inline TOML
  -w, --width <WIDTH>         Set the output width (0 = auto-detect) [default: 0]
  -e, --exec <CMD>            Wrap a program for proper streaming I/O handling
  -p, --prompt <PROMPT>       PCRE regex for prompt detection (with --exec) [default: ^.*>\s+$]
  -s, --scrape <DIR>          Scrape code snippets to a directory
      --no-highlight          Disable syntax highlighting
      --no-pretty-pad         Disable pretty code block borders (use spaces instead)
      --pretty-broken         Enable code line wrapping (breaks copy-paste)
      --clipboard             Enable clipboard integration (OSC 52)
      --savebrace             Enable savebrace (save code to /tmp/savebrace)
      --paths                 Show configuration paths and exit
      --theme <THEME>         Syntax highlighting theme [default: base16-ocean.dark]
  -h, --help                  Print help
  -V, --version               Print version
```

## ⚙️ Configuration

Streamdown looks for configuration in:
1. `$XDG_CONFIG_HOME/streamdown/config.toml`
2. `~/.config/streamdown/config.toml`
3. `~/.streamdown.toml`

### Default Configuration

```toml
[style]
# Base hue (0.0-1.0) for color theme
hue = 0.6

# Terminal margin (spaces on left)
margin = 2

[style.multipliers]
# HSV multipliers for derived colors
dark = [1.0, 0.8, 0.15]
mid = [1.0, 0.5, 0.4]
symbol = [1.0, 0.6, 0.7]
head = [1.0, 0.4, 0.9]
grey = [0.0, 0.0, 0.5]
bright = [1.0, 0.8, 1.0]

[features]
# Enable OSC 52 clipboard for code blocks
clipboard = true

# Enable savebrace (save code to temp file)
savebrace = true

# Prompt pattern for PTY mode (regex)
prompt_pattern = "[$#>] $"
```

### Color Customization

The color theme is generated from a single base hue using HSV color space. Adjust the `hue` value (0.0-1.0) to change the overall color scheme:

- `0.0` - Red
- `0.3` - Green  
- `0.6` - Blue (default)
- `0.8` - Purple

## 🎨 Output Examples

### Headings

```markdown
# Level 1 Heading
## Level 2 Heading
### Level 3 Heading
```

Rendered with bold text and proper centering.

### Code Blocks

````markdown
```python
def hello():
    print("Hello, World!")
```
````

Rendered with:
- Syntax highlighting
- Box drawing borders
- Language label
- Clipboard/savebrace integration

### Tables

```markdown
| Name  | Age | City    |
|-------|-----|-------  |
| Alice | 30  | NYC     |
| Bob   | 25  | LA      |
```

Rendered with Unicode box drawing characters.

### Lists

```markdown
- Item 1
  - Nested item
    - Deeply nested
- Item 2

1. First
2. Second
3. Third
```

Rendered with proper indentation and bullets.

### Think Blocks

```markdown
<think>
Internal reasoning that should be visually distinct...

```python
# Code blocks work inside think blocks too
def reason():
    return "supported"
```

</think>
```

Special rendering for LLM "thinking" output with text wrapping, inline formatting, and full code block support (syntax highlighting, background, box drawing).

## 🔌 Programmatic Usage

Use streamdown as a library in your Rust project:

```rust
use streamdown_parser::Parser;
use streamdown_render::Renderer;

fn main() {
    let markdown = "# Hello\n\nThis is **bold** text.";

    let mut output = Vec::new();
    let mut parser = Parser::new();

    {
        let mut renderer = Renderer::new(&mut output, 80);

        for line in markdown.lines() {
            for event in parser.parse_line(line) {
                renderer.render_event(&event).unwrap();
            }
        }
    }

    print!("{}", String::from_utf8(output).unwrap());
}
```

### Streaming with Persistent State

For streaming scenarios where the renderer is recreated per chunk, use `RenderState` to carry state across instances:

```rust
use streamdown_parser::Parser;
use streamdown_render::{Renderer, RenderState};

let mut parser = Parser::new();
let mut render_state = RenderState::default();

for chunk in stream {
    let mut output = Vec::new();
    let mut renderer = Renderer::new(&mut output, 80);
    renderer.restore_state(render_state.clone());

    for event in parser.parse_line(&chunk) {
        renderer.render_event(&event).unwrap();
    }

    render_state = renderer.save_state();
    print!("{}", String::from_utf8(output).unwrap());
}
```

### Custom Themes and Highlight Background

```rust
use streamdown_render::Renderer;
use streamdown_syntax::Theme;

let mut renderer = Renderer::new(&mut output, 80);

// Set a custom syntect Theme object directly
renderer.set_custom_theme(my_theme);

// Strip token background colors from syntax highlighting,
// letting RenderStyle.code_bg control the background instead
renderer.set_highlight_background(Some((30, 30, 30)));
```

### Parser Cloning

`Parser` implements `Clone`, enabling state snapshotting for preview rendering:

```rust
let snapshot = parser.clone();
// ... do preview rendering ...
parser = snapshot; // restore original state
```

### Crate Structure

| Crate | Description |
|-------|-------------|
| `streamdown-core` | Core types, traits, and state management |
| `streamdown-ansi` | ANSI escape codes and terminal utilities |
| `streamdown-config` | Configuration loading and style computation |
| `streamdown-parser` | Streaming markdown parser |
| `streamdown-syntax` | Syntax highlighting via syntect |
| `streamdown-render` | Terminal rendering engine |
| `streamdown-plugin` | Plugin system (LaTeX, etc.) |

## 🔧 Development

### Building

```bash
cargo build
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p streamdown-parser

# Run with output
cargo test -- --nocapture
```

### Documentation

```bash
cargo doc --workspace --open
```

## 🆚 Comparison with Python Version

| Feature | Python | Rust |
|---------|--------|------|
| Streaming parsing | ✅ | ✅ |
| Syntax highlighting | ✅ (Pygments) | ✅ (syntect, stateful) |
| Tables | ✅ | ✅ |
| Code blocks | ✅ | ✅ |
| Lists (nested) | ✅ | ✅ |
| Think blocks | ✅ | ✅ (with code block support) |
| PTY exec mode | ✅ | ✅ |
| Clipboard (OSC 52) | ✅ | ✅ |
| Savebrace | ✅ | ✅ |
| LaTeX to Unicode | ✅ | ✅ (with Rewrite for inline formatting) |
| Configuration | ✅ | ✅ |
| Custom themes | ❌ | ✅ |
| Persistent render state | ❌ | ✅ |
| Parser cloning | ❌ | ✅ |
| Performance | Good | Excellent |
| Memory safety | Manual | Guaranteed |
| Binary size | ~50MB (with Python) | ~5MB |

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [kristopolous](https://github.com/kristopolous) for the original [Streamdown](https://github.com/kristopolous/Streamdown) Python implementation
- [syntect](https://github.com/trishume/syntect) for syntax highlighting
- The Rust community for excellent crates

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

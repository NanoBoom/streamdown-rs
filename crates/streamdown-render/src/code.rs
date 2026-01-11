//! Code block rendering.
//!
//! Renders fenced code blocks with:
//! - Syntax highlighting
//! - Pretty padding (▄▄▄ / ▀▀▀ borders) or space-based borders
//! - Line wrapping for long lines (optional)
//! - Language labels

use crate::{bg_color, fg_color, RenderStyle};
use streamdown_ansi::codes::RESET;
use streamdown_syntax::{HighlightState, Highlighter};

/// Characters for pretty code block borders.
pub const CODEPAD_TOP: char = '▄'; // Lower half block
pub const CODEPAD_BOTTOM: char = '▀'; // Upper half block

/// Code block rendering state.
pub struct CodeBlockState<'a> {
    /// The syntax highlighter
    pub highlighter: &'a Highlighter,
    /// Highlight state for streaming
    pub highlight_state: Option<HighlightState<'a>>,
    /// Current language
    pub language: Option<String>,
    /// Background color for the block
    pub background: String,
    /// Whether pretty padding is enabled
    pub pretty_pad: bool,
    /// Whether pretty line breaking is enabled
    pub pretty_broken: bool,
    /// Code block indent level
    pub indent: usize,
    /// Accumulated raw code (for clipboard/savebrace)
    pub raw_buffer: String,
}

impl<'a> CodeBlockState<'a> {
    /// Create a new code block state.
    pub fn new(highlighter: &'a Highlighter) -> Self {
        Self {
            highlighter,
            highlight_state: None,
            language: None,
            background: String::new(),
            pretty_pad: true,
            pretty_broken: false,
            indent: 0,
            raw_buffer: String::new(),
        }
    }

    /// Start a new code block.
    pub fn start(&mut self, language: Option<String>, style: &RenderStyle) {
        self.language = language.clone();
        self.background = bg_color(&style.dark);
        self.raw_buffer.clear();

        // Create highlight state for the language
        let lang = language.as_deref().unwrap_or("text");
        self.highlight_state = Some(self.highlighter.new_highlight_state(lang));
    }

    /// Add a line to the raw buffer.
    pub fn add_raw_line(&mut self, line: &str) {
        if !self.raw_buffer.is_empty() {
            self.raw_buffer.push('\n');
        }
        self.raw_buffer.push_str(line);
    }

    /// Get the raw code buffer.
    pub fn raw_code(&self) -> &str {
        &self.raw_buffer
    }

    /// End the current code block.
    pub fn end(&mut self) {
        self.highlight_state = None;
        self.language = None;
    }
}

/// Render the opening of a code block.
///
/// # Arguments
/// * `language` - Optional language for the code block
/// * `width` - Available width
/// * `left_margin` - Left margin string
/// * `style` - Render style
/// * `pretty_pad` - Whether to use pretty padding (▄▄▄)
///
/// # Returns
/// Vector of lines for the code block header
pub fn render_code_start(
    language: Option<&str>,
    width: usize,
    left_margin: &str,
    style: &RenderStyle,
    pretty_pad: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    let bg = bg_color(&style.dark);
    let fg = fg_color(&style.grey);

    if pretty_pad {
        // Pretty top border: ▄▄▄▄▄ (foreground color on dark background)
        let border = CODEPAD_TOP.to_string().repeat(width);
        lines.push(format!("{}{}{}{}{}", left_margin, fg, bg, border, RESET));
    } else {
        // Simple border with spaces (copy-paste friendly)
        lines.push(format!(
            "{}{}{}{}",
            left_margin,
            bg,
            " ".repeat(width),
            RESET
        ));
    }

    // Language label if provided
    if let Some(lang) = language {
        if !lang.is_empty() && lang != "text" {
            let label_fg = fg_color(&style.symbol);
            let padding = width.saturating_sub(lang.len() + 2);
            lines.push(format!(
                "{}{}{}[{}]{}{}",
                left_margin,
                bg,
                label_fg,
                lang,
                " ".repeat(padding),
                RESET
            ));
        }
    }

    lines
}

/// Render a line of code with syntax highlighting.
///
/// # Arguments
/// * `line` - The code line
/// * `state` - Code block state (for highlighting)
/// * `width` - Available width
/// * `left_margin` - Left margin string
/// * `style` - Render style
/// * `pretty_broken` - Whether to wrap long lines
///
/// # Returns
/// Vector of rendered lines (may be multiple if wrapped)
pub fn render_code_line(
    line: &str,
    state: &mut CodeBlockState<'_>,
    width: usize,
    left_margin: &str,
    style: &RenderStyle,
    pretty_broken: bool,
) -> Vec<String> {
    let bg = bg_color(&style.dark);

    // Wrap long lines if pretty_broken is enabled
    let (indent, wrapped_lines) = code_wrap(line, width, pretty_broken);

    let mut result = Vec::new();

    for (i, code_line) in wrapped_lines.iter().enumerate() {
        // Highlight the line
        let highlighted = if let Some(ref mut hl_state) = state.highlight_state {
            state
                .highlighter
                .highlight_line_with_state(code_line, hl_state)
        } else {
            code_line.to_string()
        };

        // Calculate padding
        let line_indent = if i == 0 { 0 } else { indent };
        let indent_str = " ".repeat(line_indent);

        // Build the line with background
        let visible_len = streamdown_ansi::utils::visible_length(&highlighted) + line_indent;
        let padding = width.saturating_sub(visible_len);

        result.push(format!(
            "{}{}{}{}{}{}",
            left_margin,
            bg,
            indent_str,
            highlighted,
            " ".repeat(padding),
            RESET
        ));
    }

    if result.is_empty() {
        // Empty line - still show background
        result.push(format!(
            "{}{}{}{}",
            left_margin,
            bg,
            " ".repeat(width),
            RESET
        ));
    }

    result
}

/// Render the closing of a code block.
///
/// # Arguments
/// * `width` - Available width
/// * `left_margin` - Left margin string
/// * `style` - Render style
/// * `pretty_pad` - Whether to use pretty padding (▀▀▀)
pub fn render_code_end(
    width: usize,
    left_margin: &str,
    style: &RenderStyle,
    pretty_pad: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    let bg = bg_color(&style.dark);
    let fg = fg_color(&style.grey);

    if pretty_pad {
        // Pretty bottom border: ▀▀▀▀▀
        let border = CODEPAD_BOTTOM.to_string().repeat(width);
        lines.push(format!("{}{}{}{}{}", left_margin, fg, bg, border, RESET));
    } else {
        // Simple border with spaces
        lines.push(format!(
            "{}{}{}{}",
            left_margin,
            bg,
            " ".repeat(width),
            RESET
        ));
    }

    lines
}

/// Wrap a code line if it exceeds the width.
///
/// Unlike text wrapping, code wrapping preserves indentation
/// and doesn't break on word boundaries.
///
/// # Arguments
/// * `text` - The code line
/// * `width` - Maximum width
/// * `pretty_broken` - If false, don't wrap (let terminal handle it)
///
/// # Returns
/// (indent, lines) - The detected indent and wrapped lines
pub fn code_wrap(text: &str, width: usize, pretty_broken: bool) -> (usize, Vec<String>) {
    if text.is_empty() {
        return (0, vec![String::new()]);
    }

    // If pretty_broken is disabled, don't wrap - let terminal handle it
    // This preserves copy-paste behavior
    if !pretty_broken {
        return (0, vec![text.to_string()]);
    }

    // Detect indentation
    let indent = text.len() - text.trim_start().len();
    let content = text.trim_start();

    if content.is_empty() {
        return (indent, vec![text.to_string()]);
    }

    // Calculate effective width (accounting for indent on continuation lines)
    let effective_width = width.saturating_sub(4).saturating_sub(indent);

    if effective_width == 0 || content.len() <= effective_width {
        return (indent, vec![text.to_string()]);
    }

    // Wrap the content
    let mut lines = Vec::new();
    let mut start = 0;

    while start < content.len() {
        let end = (start + effective_width).min(content.len());
        let line = &content[start..end];

        if start == 0 {
            // First line includes original indentation
            lines.push(format!("{}{}", " ".repeat(indent), line));
        } else {
            lines.push(line.to_string());
        }

        start = end;
    }

    // Remove trailing empty lines
    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    (indent, lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_style() -> RenderStyle {
        RenderStyle::default()
    }

    #[test]
    fn test_code_wrap_short_line() {
        let (indent, lines) = code_wrap("let x = 1;", 80, true);
        assert_eq!(indent, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "let x = 1;");
    }

    #[test]
    fn test_code_wrap_with_indent() {
        let (indent, lines) = code_wrap("    let x = 1;", 80, true);
        assert_eq!(indent, 4);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_code_wrap_long_line_pretty_broken() {
        let long_line = "x".repeat(100);
        let (_, lines) = code_wrap(&long_line, 40, true);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_code_wrap_long_line_no_pretty_broken() {
        let long_line = "x".repeat(100);
        let (_, lines) = code_wrap(&long_line, 40, false);
        // Should NOT wrap when pretty_broken is false
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], long_line);
    }

    #[test]
    fn test_code_wrap_empty() {
        let (indent, lines) = code_wrap("", 80, true);
        assert_eq!(indent, 0);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_code_start_pretty() {
        let lines = render_code_start(Some("rust"), 80, "", &default_style(), true);
        assert!(lines.len() >= 1);
        // Should have ▄ border
        assert!(lines[0].contains(CODEPAD_TOP));
    }

    #[test]
    fn test_render_code_start_not_pretty() {
        let lines = render_code_start(Some("rust"), 80, "", &default_style(), false);
        assert!(lines.len() >= 1);
        // Should NOT have ▄ border (space-based instead)
        assert!(!lines[0].contains(CODEPAD_TOP));
    }

    #[test]
    fn test_render_code_end_pretty() {
        let lines = render_code_end(80, "", &default_style(), true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains(CODEPAD_BOTTOM));
    }

    #[test]
    fn test_render_code_end_not_pretty() {
        let lines = render_code_end(80, "", &default_style(), false);
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].contains(CODEPAD_BOTTOM));
    }

    #[test]
    fn test_code_block_state_raw_buffer() {
        let highlighter = Highlighter::new();
        let mut state = CodeBlockState::new(&highlighter);
        let style = default_style();

        state.start(Some("rust".to_string()), &style);
        state.add_raw_line("fn main() {");
        state.add_raw_line("    println!(\"Hello\");");
        state.add_raw_line("}");

        assert_eq!(state.raw_code(), "fn main() {\n    println!(\"Hello\");\n}");
    }
}

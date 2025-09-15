//! Fast syntax highlighting using syntect
//!
//! Uses syntect to provide fast post-processing syntax highlighting

use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use std::collections::HashMap;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

// Only load from single thread once
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// Fast syntax highlighting using syntect with caching optimization
pub struct SyntaxHighlighter {
    /// Cache of file extension to syntax for performance
    syntax_cache: HashMap<String, &'static SyntaxReference>,
    /// Pre-loaded syntect theme for performance
    theme: &'static Theme,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with optimized global state
    pub fn new() -> Self {
        // Load theme set once
        let theme_set = THEME_SET.get_or_init(|| ThemeSet::load_defaults());

        // TODO: Add support for user-defined themes
        let theme = &theme_set.themes["base16-ocean.dark"];

        Self {
            syntax_cache: HashMap::new(),
            theme,
        }
    }

    /// Get the global syntax set
    fn get_syntax_set() -> &'static SyntaxSet {
        SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
    }

    /// Get cached syntax reference for a given file extension
    fn get_cached_syntax(&mut self, extension: &str) -> Option<&'static SyntaxReference> {
        // Check cache first
        if let Some(cached_syntax) = self.syntax_cache.get(extension) {
            return Some(*cached_syntax);
        }

        // Not in cache, so load from syntax set
        let syntax_set = Self::get_syntax_set();
        if let Some(syntax) = syntax_set.find_syntax_by_extension(extension) {
            // Cache syntax reference
            self.syntax_cache.insert(extension.to_string(), syntax);
            Some(syntax)
        } else {
            None
        }
    }

    /// Highlight plain text with syntax colors for file preview
    pub fn highlight_text(&mut self, content: &str, extension: Option<&str>) -> Text<'static> {
        let extension = match extension {
            Some(ext) => ext,
            None => return Text::from(content.to_string()),
        };

        //Use cached syntax lookup for performance
        let syntax = self.get_cached_syntax(extension);
        let syntax = match syntax {
            Some(syntax) => syntax,
            None => return Text::from(content.to_string()),
        };

        let mut hightlighter = HighlightLines::new(syntax, &self.theme);

        let syntax_set = Self::get_syntax_set();
        let mut lines = Vec::new();
        for line in LinesWithEndings::from(content) {
            let highlights = hightlighter
                .highlight_line(line, syntax_set)
                .unwrap_or_default();
            let spans: Vec<Span> = highlights
                .iter()
                .map(|(style, text)| {
                    let ratatui_style = self.syntect_style_to_ratatui(*style);
                    Span::styled(text.to_string(), ratatui_style)
                })
                .collect();

            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }

    /// Apply syntax highlighting and highlight the target line with background color
    fn highlight_preview_with_target_line(
        &mut self,
        content: &str,
        extension: Option<&str>,
        target_line: Option<usize>,
    ) -> Text<'static> {
        // First apply syntax highlighting to get the base highlighted text
        let mut highlighted_text = self.highlight_text(content, extension);

        // If we have a target line to highlight, apply the background color to it
        if let Some(target_line_num) = target_line {
            // Pre-compute target string once
            let target_str = target_line_num.to_string();
            
            // Parse the bat output to find the line with the target line number
            for line in highlighted_text.lines.iter_mut() {
                // Check if this line contains the target line number
                // bat output is formatted as "   2 | content here"
                if Self::line_contains_line_number(line, &target_str) {
                    // Apply background color to the target line
                    for span in &mut line.spans {
                        // Preserve existing foreground color but add background color
                        let existing_fg = span.style.fg.unwrap_or(Color::White);
                        span.style = span
                            .style
                            .bg(Color::Rgb(64, 64, 64))
                            .fg(existing_fg);
                    }
                    break; // Exit loop once we find the target line
                }
            }
        }
        
        highlighted_text
    }
    
    /// Check if a line contains a given line number
    fn line_contains_line_number(line: &Line, target_str: &str) -> bool {
        // Early exit if line is empty
        if line.spans.is_empty() {
            return false;
        }
        
        // Build line text by concatenating all spans
        let line_text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
    
        // Check if line contains target line number after trimming whitespace
        let trimmed = line_text.trim_start();
        
        if let Some(rest) = trimmed.strip_prefix(target_str) {
            // After line number, there should be a non-digit character (space, |, etc.)
            rest.is_empty() || !rest.chars().next().unwrap_or(' ').is_ascii_digit()
        } else {
            false
        }
    }

    /// Convert syntect style to ratatui style
    fn syntect_style_to_ratatui(&self, style: SyntectStyle) -> Style {
        let fg_color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);

        let mut ratatui_style = Style::default().fg(fg_color);

        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::BOLD)
        {
            ratatui_style = ratatui_style.bold();
        }

        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::ITALIC)
        {
            ratatui_style = ratatui_style.italic();
        }

        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::UNDERLINE)
        {
            ratatui_style = ratatui_style.underlined();
        }

        ratatui_style
    }

    /// Fast method to highlight line in search results
    pub fn highlight_line(&mut self, line: &str, extension: Option<&str>) -> Line<'static> {
        let extension = match extension {
            Some(ext) => ext,
            None => return Line::from(line.to_string()),
        };

        // Use cached syntax lookup for performance
        let syntax = self.get_cached_syntax(extension);
        let syntax = match syntax {
            Some(syntax) => syntax,
            None => return Line::from(line.to_string()),
        };

        let mut hightlighter = HighlightLines::new(syntax, &self.theme);
        let syntax_set = Self::get_syntax_set();

        // Highlight just this one line
        let highlights = hightlighter
            .highlight_line(line, syntax_set)
            .unwrap_or_default();
        let spans: Vec<Span> = highlights
            .iter()
            .map(|(style, text)| {
                let ratatui_style = self.syntect_style_to_ratatui(*style);
                Span::styled(text.to_string(), ratatui_style)
            })
            .collect();

        Line::from(spans)
    }

    /// Extract file extension from path
    fn get_extension(path: &str) -> Option<&str> {
        path.split('.').last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sytanx_highlighter_caching() {
        let mut highlighter = SyntaxHighlighter::new();

        // Test that repeated calls with same extension are cached
        let line = "fn main() {\n    println!(\"Hello, world!\");\n}";

        // First call - cache miss
        let start = Instant::now();
        let _ = highlighter.highlight_text(line, Some("rs"));
        let first_time = start.elapsed();

        // Second call - cache hit
        let start = Instant::now();
        let _ = highlighter.highlight_text(line, Some("rs"));
        let second_time = start.elapsed();

        // Cache lookup should be faster than syntax set lookup
        // Were mainly testing that it doesn't panic
        assert!(first_time.as_nanos() > 0);
        assert!(second_time.as_nanos() > 0);

        // Verify that cache is working
        assert!(highlighter.syntax_cache.contains_key("rs"));
    }

    #[test]
    fn test_sytanx_highlighter_different_extensions() {
        let mut highlighter = SyntaxHighlighter::new();

        // Test different extensions are cached separately
        let rust_line = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let js_line = "console.log(\"Hello, world!\");";
        let py_line = "print(\"Hello, world!\")";

        let _ = highlighter.highlight_text(rust_line, Some("rs"));
        let _ = highlighter.highlight_text(js_line, Some("js"));
        let _ = highlighter.highlight_text(py_line, Some("py"));

        // All extensions should be cached
        assert!(highlighter.syntax_cache.contains_key("rs"));
        assert!(highlighter.syntax_cache.contains_key("js"));
        assert!(highlighter.syntax_cache.contains_key("py"));

        // Cache should have 3 entries
        assert_eq!(highlighter.syntax_cache.len(), 3);
    }

    #[test]
    fn test_sytanx_highlighter_unknown_extension() {
        let mut highlighter = SyntaxHighlighter::new();

        let line = "some text with no extension";
        let result = highlighter.highlight_line(line, Some("unknowntext"));

        // Should return original text
        assert_eq!(result.spans.len(), 1);
        assert_eq!(result.spans[0].content, line);

        // Cache won't store unknown extension
        // We only cache known extensions
        assert!(!highlighter.syntax_cache.contains_key("unknowntext"));
    }

    #[test]
    fn test_global_syntax_set_initialization() {
        // Test that global syntax set is initialized
        let syntax_set = SyntaxHighlighter::get_syntax_set();

        // Should have common syntax definitions
        assert!(syntax_set.find_syntax_by_extension("rs").is_some());
        assert!(syntax_set.find_syntax_by_extension("py").is_some());
        assert!(syntax_set.find_syntax_by_extension("js").is_some());

        // Test that multiple calls return same syntax set
        let syntax_set_2 = SyntaxHighlighter::get_syntax_set();
        assert!(std::ptr::eq(syntax_set, syntax_set_2));
    }

    #[test]
    fn test_cached_performance_with_many_extensions() {
        let mut highlighter = SyntaxHighlighter::new();

        let extensions = [
            "rs", "py", "js", "java", "c", "cpp", "go", "rb", "php", "swift",
        ];
        let line = "test line";

        // First pass - populate cache
        let start = Instant::now();
        for extension in extensions {
            let _ = highlighter.highlight_line(line, Some(extension));
        }
        let first_pass = start.elapsed();

        // Second pass - cache hit
        let start = Instant::now();
        for extension in extensions {
            let _ = highlighter.highlight_line(line, Some(extension));
        }
        let second_pass = start.elapsed();

        // Cache should have most of extensions
        assert!(highlighter.syntax_cache.len() > extensions.len() - 2);

        // functions should not panic
        println!("First pass {:?} Second pass {:?}", first_pass, second_pass);
        assert!(first_pass.as_nanos() > 0);
        assert!(second_pass.as_nanos() > 0);
    }

    #[test]
    fn test_extract_file_extension() {
        assert_eq!(SyntaxHighlighter::get_extension("file.rs"), Some("rs"));
        assert_eq!(
            SyntaxHighlighter::get_extension("path/to/file.js"),
            Some("js")
        );
        assert_eq!(SyntaxHighlighter::get_extension("file.tar.gz"), Some("gz"));
        assert_eq!(SyntaxHighlighter::get_extension("file"), Some("file"));
        assert_eq!(SyntaxHighlighter::get_extension(""), Some(""));
        assert_eq!(SyntaxHighlighter::get_extension(".file"), Some("file"));
    }

    #[test]
    fn test_theme_consistency() {
        let highlighter1 = SyntaxHighlighter::new();
        let highlighter2 = SyntaxHighlighter::new();

        // Both highlighters should have same theme
        assert!(std::ptr::eq(highlighter1.theme, highlighter2.theme));
    }
}

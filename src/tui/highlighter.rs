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
        let highlights = hightlighter.highlight_line(line, syntax_set).unwrap_or_default();
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

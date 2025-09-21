//! Search orchestration module
//!
//! Manages the search piplenes: rg -> Rust program

pub mod engines;
pub mod sorter;

pub use engines::SearchEngine;

use crate::{cli::Cli, tui::highlighter::SyntaxHighlighter};
use ratatui::text::Line;

/// Represents a single search result
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub matched_text: String,
    /// Original line content with coloring from rg
    pub line_colored_content: Option<String>,
    /// Base directory of search (used for relative path)
    pub base_dir: Option<String>,
    /// Pre-computed display path (cached for performance)
    display_path: String,
}

impl SearchResult {
    /// Creates a new search result
    pub fn new(
        file_path: String,
        line_number: usize,
        line_content: String,
        matched_text: String,
        line_colored_content: Option<String>,
        base_dir: Option<String>,
    ) -> Self {
        let display_path = Self::compute_display_path(&file_path, base_dir.as_deref());
        Self {
            file_path,
            line_number,
            line_content,
            matched_text,
            line_colored_content,
            base_dir,
            display_path,
        }
    }

    /// Compute display path once during construction (for performance)
    fn compute_display_path(file_path: &str, base_dir: Option<&str>) -> String {
        let cleaned_path = if file_path.starts_with("./") {
            &file_path[2..]
        } else {
            file_path
        };

        // If base_dir is set, make path relative to it
        if let Some(base_directory) = base_dir {
            // if cleaned path starts with base_dir, make it relative
            if let Some(relative_path) = cleaned_path.strip_prefix(base_directory) {
                // Strip leading slash if present
                let relative_path = relative_path.strip_prefix('/').unwrap_or(relative_path);
                return relative_path.to_string();
            }
        }
        // Also handles case where base_dir might be absolute and file path might be relative
        // or other edge cases - just return the cleaned path
        cleaned_path.to_string()
    }

    /// Format the result for display in the TUI
    /// If use_color is true, the line will be syntax-highlighted
    pub fn format_for_display(&self, use_color: bool) -> String {
        // Use the pre-computed display path for optimal performance
        // Use colored content if available and requested, otherwise fallback to line content
        let content = if use_color && self.line_colored_content.is_some() {
            self.line_colored_content.as_ref().unwrap().trim()
        } else {
            self.line_content.trim()
        };

        format!("{}:{} {}", self.display_path, self.line_number, content)
    }

    /// Format the result for TUI display with fast syntax highlighting
    pub fn format_for_tui_display(&self, highlighter: &mut SyntaxHighlighter) -> Line<'static> {
        // Use the pre-computed display path for optimal performance
        // Extract file extension for syntax highlighting
        let extension = SyntaxHighlighter::get_extension(&self.display_path);

        // Create formated line with syntax highlighting
        let line_content = format!(
            "{}:{} {}",
            self.display_path,
            self.line_number,
            self.line_content.trim()
        );
        highlighter.highlight_line(&line_content, extension)
    }

    /// Get pre-computed display path
    pub fn get_display_path(&self) -> &str {
        &self.display_path
    }
}

/// Status information for progressive loading
#[derive(Debug, Clone)]
pub struct ProgressiveLoadStatus {
    pub total_loaded: usize,
    pub loading_complete: bool,
    pub total_files_found: usize,
    pub load_threshold: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_display_path_computation_and_formatting() {
        // Test basic cases
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            42,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            None,
        );
        assert_eq!(result.get_display_path(), "src/main.rs");
        assert_eq!(
            result.format_for_display(false),
            "src/main.rs:42 fn main() {"
        );

        // Test dot prefix removal
        let result = SearchResult::new(
            "./src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            None,
        );
        assert_eq!(result.get_display_path(), "src/main.rs");
        let display = result.format_for_display(false);
        assert!(!display.starts_with("./"));
        assert!(display.starts_with("src/main.rs:10"));

        // Test relative path
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            Some("src".to_string()),
        );
        assert_eq!(result.get_display_path(), "main.rs");
        let display = result.format_for_display(false);
        assert!(display.starts_with("main.rs:10"));

        // Test dot prefix removal with relative path
        let result = SearchResult::new(
            "./src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            Some("src".to_string()),
        );
        assert_eq!(result.get_display_path(), "main.rs");

        // Test caching and performance - very long path
        let result = SearchResult::new(
            "/home/user/project/src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            Some("/home/user/project".to_string()),
        );
        let formatted1 = result.format_for_display(false);
        let formatted2 = result.format_for_display(false);
        assert_eq!(formatted1, formatted2);
        assert!(formatted1.starts_with("src/main.rs:10"));

        // Test coloring
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            Some("assert_eq!(formatted1, formatted2);".to_string()),
            Some("src".to_string()),
        );
        let formatted = result.format_for_display(true);
        assert_eq!(formatted, "main.rs:10 assert_eq!(formatted1, formatted2);");

        // Test static method direclty
        assert_eq!(
            SearchResult::compute_display_path("src/main.rs", None),
            "src/main.rs"
        );
        assert_eq!(
            SearchResult::compute_display_path("./src/main.rs", None),
            "src/main.rs"
        );
        assert_eq!(
            SearchResult::compute_display_path("./src/main.rs", Some("src")),
            "main.rs"
        );
        assert_eq!(
            SearchResult::compute_display_path("tmp/main.rs", Some("src")),
            "tmp/main.rs"
        );
    }

    #[test]
    fn test_display_path_consistency_across_constructor() {
        let path = "src/main.rs";
        let line_number = 10;
        let line_content = "fn main() {";
        let matched_text = "main";

        // All constructors should produce the same display path for the same input
        let result1 = SearchResult::new(
            path.to_string(),
            line_number,
            line_content.to_string(),
            matched_text.to_string(),
            None,
            None,
        );

        let result2 = SearchResult::new(
            path.to_string(),
            line_number,
            line_content.to_string(),
            matched_text.to_string(),
            Some("colorized content".to_string()),
            None,
        );

        assert_eq!(result1.get_display_path(), result2.get_display_path());
        assert_eq!(result1.get_display_path(), path);
    }

    #[test]
    fn test_search_result_traits() {
        // Test basic field access and construction
        let basic_result = SearchResult::new(
            "src/main.rs".to_string(),
            42,
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            None,
            None,
        );
        assert_eq!(basic_result.file_path, "src/main.rs");
        assert_eq!(basic_result.line_number, 42);
        assert_eq!(
            basic_result.line_content,
            "    assert_eq!(formatted1, formatted2);"
        );
        assert_eq!(basic_result.matched_text, "assert_eq!");

        // Test basic display formmatting
        let display = basic_result.format_for_display(false);
        assert!(display.contains("src/main.rs:42"));
        assert!(display.contains("assert_eq!(formatted1, formatted2);"));

        // Test PartialEq
        let result1 = SearchResult::new(
            "src/main.rs".to_string(),
            42,
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            None,
            None,
        );
        let result2 = SearchResult::new(
            "src/main.rs".to_string(),
            42,
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            None,
            None,
        );
        let result3 = SearchResult::new(
            "src/main.rs".to_string(),
            36, // Different line number
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            None,
            None,
        );
        assert_eq!(result1, result2);
        assert_ne!(result1, result3);

        // Test Clone
        let cloned = result1.clone();
        assert_eq!(result1, cloned);
        assert_eq!(result1.file_path, cloned.file_path);
        assert_eq!(result1.line_number, 42);

        // Test Debug
        let debug_str = format!("{:?}", result1);
        assert!(debug_str.contains("src/main.rs"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("SearchResult"));
    }

    #[test]
    fn test_search_result_edge_cases_and_path_variants() {
        // Test path variants in a loop
        let test_cases = vec![
            ("simple.rs", 1, "simple.rs:1"),
            ("./simple.rs", 1, "simple.rs:1"),
            ("./simple.rs", 2, "simple.rs:2"),
            ("src/simple.rs", 100, "simple.rs:100"),
            ("files with spaces.rs", 100, "files with spaces.rs:100"),
            (
                "files_with_underscores.rs",
                100,
                "files_with_underscores.rs:100",
            ),
        ];

        for (file_path, line_number, expected_substring) in test_cases {
            let result = SearchResult::new(
                file_path.to_string(),
                line_number,
                "some content".to_string(),
                "some matched text".to_string(),
                None,
                None,
            );
            let display = result.format_for_display(false);
            assert!(display.contains(expected_substring));
            assert!(display.contains("some content"));
        }

        //Empty content
        let result = SearchResult::new(
            "".to_string(),
            0,
            "".to_string(),
            "".to_string(),
            None,
            None,
        );
        assert_eq!(result.file_path, "");
        assert_eq!(result.line_number, 0);
        assert_eq!(result.line_content, "");

        // Very long content
        let long_content = "a".repeat(1000);
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            long_content.to_string(),
            "a".to_string(),
            None,
            None,
        );
        assert_eq!(result.line_content, long_content);
        
        // Special characters
        let special_content = "fn test(a: u8) -> u8, Box<u8> {\n let a = 1;\n    let b = 2;\n}";
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            special_content.to_string(),
            "Box".to_string(),
            None,
            None,
        );
        assert_eq!(result.line_content, special_content);
        assert!(result.format_for_display(false).contains("src/main.rs:10"));
        
        // Unicode content - chinese + emoji
        let unicode_content = "// ❤️ 😍 你好 禾風紅土歡苗點不歌巴禾追休";
        let result = SearchResult::new(
            "src/歌巴.rs".to_string(),
            10,
            unicode_content.to_string(),
            "禾風紅土歡苗點不歌巴禾追休".to_string(),
            None,
            None,
        );
        assert_eq!(result.file_path, "src/歌巴.rs");
        assert_eq!(result.matched_text, "禾風紅土歡苗點不歌巴禾追休");
        assert!(result.format_for_display(false).contains("src/歌巴.rs:10"));
    }
    
    #[test]
    fn test_comprehensive_path_and_display_scenarios() {
        // Test with base directory
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            Some("src".to_string()),
        );
        let display = result.format_for_display(false);
        assert!(display.starts_with("main.rs:10"));
        assert!(!display.contains("src/main.rs"));
        
        // Test without base directory
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "main".to_string(),
            None,
            None,
        );
        let display = result.format_for_display(false);
        assert!(display.starts_with("src/main.rs:10"));
        
        // Test no match (fallback to full path)
        let result = SearchResult::new(
            "other/path/src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "x".to_string(),
            None,
            Some("myproject".to_string()),
        );
        let display = result.format_for_display(false);
        assert!(display.starts_with("other/path/src/main.rs:10"));
        
        // Test dot prefix with base directory
        let result = SearchResult::new(
            "./src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "test".to_string(),
            None,
            Some("myproj".to_string()),
        );
        let display = result.format_for_display(false);
        assert!(display.starts_with("src/main.rs:10"));
        assert!(!display.contains("./src/main.rs"));
        assert!(!display.contains("./myproj/src/main.rs"));
        
        // Test dot prefix without base directory
        let result = SearchResult::new(
            "./src/main.rs".to_string(),
            10,
            "fn main() {".to_string(),
            "test".to_string(),
            None,
            None,
        );
        let display = result.format_for_display(false);
        assert!(!display.starts_with("./"));
        assert!(display.starts_with("src/main.rs:10"));
        
        // Test content trimming
        let result = SearchResult::new(
            "src/main.rs".to_string(),
            10,
            "    assert_eq!(formatted1, formatted2);".to_string(),
            "assert_eq!".to_string(),
            None,
            None,
        );
        let display = result.format_for_display(false);
        assert!(display.contains("src/main.rs:10 assert_eq!(formatted1, formatted2);"));
        assert!(!display.contains("    assert_eq!(formatted1, formatted2);"));
        
        // Test complex content with dot prefix
        let result = SearchResult::new(
            "./very/long/path/src/main.rs".to_string(),
            999,
            "    let a = 1;\n    let b = 2;\n}".to_string(),
            "let".to_string(),
            None,
            None
        );
        let display = result.format_for_display(false);
        assert!(display.starts_with("very/long/path/src/main.rs:999"));
        assert!(display.contains(" let a = 1;\n    let b = 2;\n}"));
        assert!(!display.contains("./"));
    }
}

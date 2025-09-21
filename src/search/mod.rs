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

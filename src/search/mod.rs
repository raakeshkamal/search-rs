//! Search orchestration module
//! 
//! Manages the search piplenes: rg -> Rust program

pub mod engines;
pub mod sorter;

pub use engines::SearchEngine;

/// Represents a single search result
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub matched_text: String,
}

impl SearchResult {
    /// Creates a new search result
    pub fn new(
        file_path: String,
        line_number: usize,
        line_content: String,
        matched_text: String,
    ) -> Self {
        Self {
            file_path,
            line_number,
            line_content,
            matched_text,
        }
    }
}
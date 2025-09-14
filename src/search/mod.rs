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
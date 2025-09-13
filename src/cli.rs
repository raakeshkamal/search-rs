//! Command Line Interface module
//!
//! Handles command-line argument parsing using clap, supports multiple search modes
//!

use clap::Parser;
use std::path::PathBuf;

/// Interactive Search Tool - A TUI enhanced code search tool based on rip-grep
#[derive(Parser, Debug)]
#[command(
    name = "search-rs",
    about = "Interactive Search Tool - A TUI enhanced code search tool based on rip-grep",
    long_about = "Interactive Search Tool - Rust based TUI that orchestrates rip-grep for enhanced code-search

    EXAMPLES:
        search-rs \"search pattern\"
        search-rs -e \"search pattern\" # Case sensitive search (default)
        search-rs -i \"search pattern\" # Case insensitive search
        search-rs -s \"search pattern\" # Substring search
        search-rs -d /path/to/dir # Search in a specific directory

    USAGE TIP:
        Use arrow keys to navigate, press enter to open a search result in a code editor
    "
)]
#[command(version)]
pub struct Cli {
    /// Search pattern to search for
    #[arg(help = "Search pattern to search for in files")]
    pub pattern: String,

    /// Case sensitive search
    #[arg(short, long, help = "Case sensitive search")]
    pub exact: bool,

    /// Case insensitive search
    #[arg(short, long, help = "Case insensitive search (default)")]
    pub ignore_case: bool,

    /// Substring search
    #[arg(short, long, help = "Substring search (case sensitive)")]
    pub substring: bool,

    /// Search in a specific directory
    #[arg(
        short,
        long,
        help = "Search in a specific directory (default: current directory)"
    )]
    pub directory: Option<PathBuf>,

    /// debug mode
    #[arg(
        short,
        long,
        help = "Debug mode (logging to /tmp file with timestamps)"
    )]
    pub debug: bool,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Validate command line arguments
    pub fn validate(&self) -> bool {
        // Ensure only one search mode is selected
        let modes = [self.exact, self.ignore_case, self.substring];
        let mode_count = modes.iter().filter(|&&x| x).count();

        if mode_count > 1 {
            eprintln!("Error: Only one search mode can be selected");
            return false;
        }

        // Validate directory path if provided
        if let Some(dir) = &self.directory {
            if !dir.exists() {
                eprintln!("Error: Directory path must be an absolute path");
                return false;
            }
            if !dir.is_dir() {
                eprintln!("Error: Directory path must be a directory");
                return false;
            }
        }
        
        // Validate search pattern is not empty
        if self.pattern.is_empty() {
            eprintln!("Error: Search pattern cannot be empty");
            return false;
        }

        true
    }
    
    /// Get the search mode
    pub fn search_mode(&self) -> String {
        match (self.exact, self.ignore_case, self.substring) {
            (true, false, false) => "exact".to_string(),
            (false, true, false) => "ignore_case".to_string(),
            (false, false, true) => "substring".to_string(),
            _ => "exact".to_string(),
        }
    }
    
    /// Get the search directory, defaulting to current directory
    pub fn search_dir(&self) -> String {
        match &self.directory {
            Some(dir) => dir.to_str().unwrap().to_string(),
            None => ".".to_string(),
        }
    }
}

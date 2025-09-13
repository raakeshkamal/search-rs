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
        if self.pattern.trim().is_empty() {
            eprintln!("Error: Search pattern cannot be empty");
            return false;
        }

        true
    }
    
    /// Get the search mode
    pub fn search_mode(&self) -> SearchMode {
        match (self.exact, self.ignore_case, self.substring) {
            (true, false, false) => SearchMode::Exact,
            (false, true, false) => SearchMode::IgnoreCase,
            (false, false, true) => SearchMode::Substring,
            _ => SearchMode::Exact,
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

/// Search modes supported by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Exact,
    IgnoreCase,
    Substring,
}

impl SearchMode {
    /// Get the search mode as a string
    pub fn name(&self) -> &'static str {
        match self {
            SearchMode::Exact => "exact",
            SearchMode::IgnoreCase => "ignore_case",
            SearchMode::Substring => "substring",
        }
    }
    
    /// Get the search mode description
    pub fn description(&self) -> &'static str {
        match self {
            SearchMode::Exact => "Exact whole word matches (case sensitive)",
            SearchMode::IgnoreCase => "Case insensitive search (default)",
            SearchMode::Substring => "Substring search (case sensitive)",
        }
    }
}

#[cfg(test)]
mod tests {
    // import everything from above
    use super::*;

    fn create_test_cli(
        pattern: &str,
        exact: bool,
        ignore_case: bool,
        substring: bool,
        directory: Option<PathBuf>,
    ) -> Cli {
        Cli {
            pattern: pattern.to_string(),
            exact,
            ignore_case,
            substring,
            directory,
            debug: false,
        }
    }
    
    #[test]
    fn test_signle_mode_validation() {
        // Single modes should be valid
        let cli = create_test_cli("search pattern", true, false, false, None);
        assert!(cli.validate());
        
        let cli = create_test_cli("search pattern", false, true, false, None);
        assert!(cli.validate());
        
        let cli = create_test_cli("search pattern", false, false, true, None);
        assert!(cli.validate());
        
        // Nothing is specified
        let cli = create_test_cli("search pattern", false, false, false, None);
        assert!(cli.validate());
    }
    
    #[test]
    fn test_multiple_mode_is_invalid() {
        // Multiple modes should not be valid
        let cli = create_test_cli("search pattern", true, true, true, None);
        assert!(!cli.validate());
        
        let cli = create_test_cli("search pattern", true, true, false, None);
        assert!(!cli.validate());
        
        let cli = create_test_cli("search pattern", true, false, true, None);
        assert!(!cli.validate());
    }
    
    #[test]
    fn test_empty_search_pattern_is_invalid() {
        // Empty search pattern should not be valid
        let cli = create_test_cli("", true, false, false, None);
        assert!(!cli.validate());
        
        let cli = create_test_cli(" ", true, false, false, None);
        assert!(!cli.validate());
        
        let cli = create_test_cli("\t\n", true, false, false, None);
        assert!(!cli.validate());
    }
    
    #[test]
    fn test_valid_pattern() {
        // Valid search pattern should be valid
        let cli = create_test_cli("search pattern", true, false, false, None);
        assert!(cli.validate());
    }
    
    #[test]
    fn test_get_search_mode() {
        // Exact mode
        let cli = create_test_cli("search pattern", true, false, false, None);
        assert_eq!(cli.search_mode(), SearchMode::Exact);
        
        // Ignore case mode
        let cli = create_test_cli("search pattern", false, true, false, None);
        assert_eq!(cli.search_mode(), SearchMode::IgnoreCase);
        
        // Substring mode
        let cli = create_test_cli("search pattern", false, false, true, None);
        assert_eq!(cli.search_mode(), SearchMode::Substring);
    }
    
    #[test]
    fn test_searh_dir() {
        // Default directory
        let cli = create_test_cli("search pattern", false, false, false, None);
        assert_eq!(cli.search_dir(), ".");
        
        // Custom directory
        let cli = create_test_cli("search pattern", false, false, false, Some(PathBuf::from("/path/to/dir")));
        assert_eq!(cli.search_dir(), "/path/to/dir");
    }
    
    #[test]
    fn test_search_mode_name_and_description() {
        // Exact mode
        let cli = create_test_cli("search pattern", true, false, false, None);
        assert_eq!(cli.search_mode().name(), "exact");
        assert_eq!(cli.search_mode().description(), "Exact whole word matches (case sensitive)");
        
        // Ignore case mode
        let cli = create_test_cli("search pattern", false, true, false, None);
        assert_eq!(cli.search_mode().name(), "ignore_case");
        assert_eq!(cli.search_mode().description(), "Case insensitive search (default)");
        
        // Substring mode
        let cli = create_test_cli("search pattern", false, false, true, None);
        assert_eq!(cli.search_mode().name(), "substring");
        assert_eq!(cli.search_mode().description(), "Substring search (case sensitive)");
    }
    
    #[test]
    fn test_invalid_search_dir() {
        // Invalid directory
        let cli = create_test_cli("search pattern", false, false, false, Some(PathBuf::from("/path/to/dir/invalid")));
        assert!(!cli.validate());
    }

}
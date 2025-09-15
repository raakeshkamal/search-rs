//! Interactive Search Tool - A TUI enhanced code search tool based on rip-grep
//!
//! A Rust based terminal user interface (TUI) application that provides an enhanced code search
//! experience by orchestrating rip-grep
//! while offering superior user control and preview capabilities

pub mod cli;
pub mod dependencies;
pub mod logging;
pub mod error;
pub mod search;
pub mod tui;

// Re-export `Cli` for use from `main`
pub use cli::Cli;
pub use dependencies::Dependencies;
pub use error::{Result, SearchError};
pub use logging::init_debug_logging;
pub use search::SearchEngine;
pub use tui::{ResultsAreaInfo};


//! Interactive Search Tool - A TUI enhanced code search tool based on rip-grep
//! 
//! A Rust based terminal user interface (TUI) application that provides an enhanced code search
//! experience by orchestrating rip-grep
//! while offering superior user control and preview capabilities

pub mod cli;
pub mod error;

// Re-export `Cli` for use from `main`
pub use cli::Cli;
pub use error::{Result, SearchError};

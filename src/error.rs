//! Error handling.
//!
//! This module provides a custom error type for the project.

use std::fmt;

/// Result type alias for the search application.
pub type Result<T> = std::result::Result<T, SearchError>;

/// Main error type for the search application.
#[derive(Debug)]
pub enum SearchError {
    /// Invalid command line arguments.
    /// This allows you to store a more detailed message explaining why the arguments were invalid.
    InvalidArguments(String),

    /// Missing required dependency.
    MissingDependency {
        tool: String,
        install_instructions: String,
    },

    /// IO error.
    IoError(std::io::Error),

    /// Input validation error.
    InvalidInput(String),
    
    /// Invalid search pattern.
    InvalidPattern{pattern: String, reason: String},

    /// File access error.
    FileAccessError { path: String, reason: String },
}

//! Error handling.
//!
//! This module provides a custom error type for the project.

use colored::*;
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

    /// TUI rendering error.
    TuiError(String),

    /// Input validation error.
    InvalidInput(String),

    /// Invalid search pattern.
    InvalidPattern { pattern: String, reason: String },

    /// Terminal related error.
    TerminalError(String),

    /// File access error.
    FileAccessError { path: String, reason: String },

    /// Search process error.
    SearchProcessError(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let warn_msg: String;
        match self {
            SearchError::InvalidArguments(msg) => {
                warn_msg = format!("Invalid arguments: {}", msg).to_string();
            }
            SearchError::MissingDependency {
                tool,
                install_instructions,
            } => {
                warn_msg = format!(
                    "Missing dependency: {}\n Install instructions: {}",
                    tool, install_instructions
                );
            }
            SearchError::IoError(err) => {
                warn_msg = format!("IO error: {}", err);
            }
            SearchError::TuiError(err) => {
                warn_msg = format!("TUI error: {}", err);
            }
            SearchError::InvalidInput(err) => {
                warn_msg = format!("Invalid input: {}", err);
            }
            SearchError::InvalidPattern { pattern, reason } => {
                warn_msg = format!("Invalid search pattern: {}\n reason: {}", pattern, reason);
            }
            SearchError::TerminalError(err) => {
                warn_msg = format!(
                    "Terminal error: {}\n Try running in a proper terminal.",
                    err
                );
            }
            SearchError::FileAccessError { path, reason } => {
                warn_msg = format!("File access error: Path: {}\n Reason: {}", path, reason);
            }
            SearchError::SearchProcessError(err) => {
                warn_msg = format!("Search error: {}", err);
            }
        }
        write!(f, "{}", warn_msg.red().bold())
    }
}

impl std::error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SearchError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        SearchError::IoError(err)
    }
}

impl Clone for SearchError {
    fn clone(&self) -> Self {
        match self {
            SearchError::InvalidArguments(msg) => SearchError::InvalidArguments(msg.clone()),
            SearchError::MissingDependency {
                tool,
                install_instructions,
            } => SearchError::MissingDependency {
                tool: tool.clone(),
                install_instructions: install_instructions.clone(),
            },
            SearchError::IoError(err) => {
                SearchError::IoError(std::io::Error::new(err.kind(), err.to_string()))
            }
            SearchError::TuiError(err) => SearchError::TuiError(err.clone()),
            SearchError::InvalidInput(err) => SearchError::InvalidInput(err.clone()),
            SearchError::InvalidPattern { pattern, reason } => SearchError::InvalidPattern {
                pattern: pattern.clone(),
                reason: reason.clone(),
            },
            SearchError::TerminalError(err) => SearchError::TerminalError(err.clone()),
            SearchError::FileAccessError { path, reason } => SearchError::FileAccessError {
                path: path.clone(),
                reason: reason.clone(),
            },
            SearchError::SearchProcessError(err) => SearchError::SearchProcessError(err.clone()),
        }
    }
}

impl SearchError {
    /// Create a terminal error with context
    pub fn terminal_error(err: &str) -> Self {
        SearchError::TerminalError(err.to_string())
    }

    /// Create a file access error with context
    pub fn file_access_error(path: &str, reason: &str) -> Self {
        SearchError::FileAccessError {
            path: path.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a invalid pattern error
    pub fn invalid_pattern(pattern: &str, reason: &str) -> Self {
        SearchError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Check if this error is recover
    pub fn is_recoverable(&self) -> bool {
        match self {
            SearchError::InvalidInput(_) => true,
            SearchError::InvalidPattern { .. } => true,
            SearchError::SearchProcessError(_) => true,
            SearchError::FileAccessError { .. } => true,
            SearchError::MissingDependency { .. } => false,
            SearchError::TerminalError(_) => false,
            SearchError::IoError(_) => false,
            SearchError::TuiError(_) => false,
            SearchError::InvalidArguments(_) => false,
            _ => true,
        }
    }

    /// Get user-friendly recovery suggestion
    pub fn get_recovery_suggestion(&self) -> Option<String> {
        match self {
            SearchError::InvalidInput(..) => {
                Some("Please check your input and try again.".to_string())
            }
            SearchError::InvalidPattern { .. } => Some("Try a simpler search pattern.".to_string()),
            SearchError::SearchProcessError(..) => {
                Some("Try different search pattern or directory: {}".to_string())
            }
            SearchError::FileAccessError { .. } => {
                Some("Check file permissions and try again.".to_string())
            }
            SearchError::MissingDependency {
                install_instructions,
                ..
            } => Some(install_instructions.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SearchError::InvalidArguments("test error".to_string());
        assert!(err.to_string().contains("Invalid arguments"));
        assert!(err.to_string().contains("test error"));

        let err = SearchError::MissingDependency {
            tool: "rg".to_string(),
            install_instructions: "cargo install rg".to_string(),
        };
        assert!(err.to_string().contains("Missing dependency"));
        assert!(err.to_string().contains("cargo install rg"));

        let err = SearchError::InvalidInput("input test error".to_string());
        assert!(err.to_string().contains("Invalid input:"));
        assert!(err.to_string().contains("input test error"));

        let err = SearchError::InvalidPattern {
            pattern: "pattern".to_string(),
            reason: "reason test".to_string(),
        };
        assert!(err.to_string().contains("Invalid search pattern:"));
        assert!(err.to_string().contains("reason:"));

        let err = SearchError::TerminalError("terminal test error".to_string());
        assert!(err.to_string().contains("Terminal error:"));
        assert!(err
            .to_string()
            .contains("Try running in a proper terminal."));

        let err = SearchError::FileAccessError {
            path: "/path".to_string(),
            reason: "access reason".to_string(),
        };
        assert!(err.to_string().contains("File access error:"));
        assert!(err.to_string().contains("Path:"));
        assert!(err.to_string().contains("Reason:"));
    }

    #[test]
    fn test_error_helper_functions() {
        // Test invalid_pattern
        let err = SearchError::invalid_pattern("pattern", "reason");
        assert!(matches!(err, SearchError::InvalidPattern { .. }));

        // Test terminal_error
        let err = SearchError::terminal_error("terminal error");
        assert!(matches!(err, SearchError::TerminalError(_)));

        // Test file_access_error
        let err = SearchError::file_access_error("/path/to/file", "access denied");
        assert!(matches!(err, SearchError::FileAccessError { .. }));
    }

    #[test]
    fn test_error_is_recoverable() {
        // Recoverable errors
        assert!(SearchError::InvalidInput("input error".to_string()).is_recoverable());
        assert!(SearchError::InvalidPattern {
            pattern: "pattern".to_string(),
            reason: "reason".to_string(),
        }
        .is_recoverable());
        assert!(SearchError::SearchProcessError("process error".to_string()).is_recoverable());
        assert!(SearchError::FileAccessError {
            path: "/path".to_string(),
            reason: "reason".to_string(),
        }
        .is_recoverable());

        // Non-recoverable errors
        assert!(!SearchError::InvalidArguments("args error".to_string()).is_recoverable());
        assert!(!SearchError::MissingDependency {
            tool: "tool".to_string(),
            install_instructions: "instructions".to_string(),
        }
        .is_recoverable());
        assert!(!SearchError::TerminalError("terminal error".to_string()).is_recoverable());
        assert!(
            !SearchError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "io error"))
                .is_recoverable()
        );
        assert!(!SearchError::TuiError("tui error".to_string()).is_recoverable());
    }

    #[test]
    fn test_recovery_suggestion() {
        // Errors with recovery suggestion
        let err = SearchError::InvalidInput("input error".to_string());
        assert!(err.get_recovery_suggestion().is_some());
        assert_eq!(
            err.get_recovery_suggestion().unwrap(),
            "Please check your input and try again."
        );

        let err = SearchError::InvalidPattern {
            pattern: "pattern".to_string(),
            reason: "reason".to_string(),
        };
        assert!(err.get_recovery_suggestion().is_some());
        assert_eq!(
            err.get_recovery_suggestion().unwrap(),
            "Try a simpler search pattern."
        );

        let err = SearchError::SearchProcessError("process error".to_string());
        assert!(err.get_recovery_suggestion().is_some());
        assert_eq!(
            err.get_recovery_suggestion().unwrap(),
            "Try different search pattern or directory: {}"
        );

        let err = SearchError::FileAccessError {
            path: "/path".to_string(),
            reason: "reason".to_string(),
        };
        assert!(err.get_recovery_suggestion().is_some());
        assert_eq!(
            err.get_recovery_suggestion().unwrap(),
            "Check file permissions and try again."
        );

        let err = SearchError::MissingDependency {
            tool: "tool".to_string(),
            install_instructions: "install instructions".to_string(),
        };
        assert!(err.get_recovery_suggestion().is_some());
        assert_eq!(
            err.get_recovery_suggestion().unwrap(),
            "install instructions"
        );

        // Errors without recovery suggestion
        let err = SearchError::InvalidArguments("args error".to_string());
        assert!(err.get_recovery_suggestion().is_none());

        let err = SearchError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
        assert!(err.get_recovery_suggestion().is_none());

        let err = SearchError::TuiError("tui error".to_string());
        assert!(err.get_recovery_suggestion().is_none());

        let err = SearchError::TerminalError("terminal error".to_string());
        assert!(err.get_recovery_suggestion().is_none());
    }
}

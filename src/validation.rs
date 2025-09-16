//! Input validation and sanitization.
//!
//! Provides validation for user inputs and file paths

use crate::constants::*;
use crate::{Result, SearchError};
use regex::Regex;

/// Input validator for search patterns and user inputs
pub struct InputValidator;

impl InputValidator {
    /// Validates and sanitizes a search pattern
    pub fn validate_search_pattern(pattern: &str) -> Result<String> {
        // Check for empty or whitespace-only pattern
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            return Err(SearchError::InvalidPattern {
                pattern: pattern.to_string(),
                reason: "Patterm cannot be empty or whitespace-only".to_string(),
            });
        }

        // Check pattern length limits
        if trimmed.len() > PATTERN_MAX_LENGTH {
            return Err(SearchError::InvalidPattern {
                pattern: pattern.to_string(),
                reason: format!(
                    "Pattern cannot be longer than {} characters",
                    PATTERN_MAX_LENGTH
                ),
            });
        }

        // Check for potentially problematic regex characters
        if let Err(_) = Regex::new(trimmed) {
            // If its not a valid regex, that's ok for literal search
            // but we should check for common problematic patterns
            let problematic_patterns = ['*', '?', '[', ']', '{', '}', '(', ')', '+', '|']; // fixed size array
            for &ch in &problematic_patterns {
                if trimmed.matches(ch).count() > MAX_PROBLEM_CHARS {
                    return Err(SearchError::InvalidPattern {
                        pattern: pattern.to_string(),
                        reason: format!(
                            "Pattern contains {} characters which may be problematic",
                            MAX_PROBLEM_CHARS
                        ),
                    });
                }
            }

            // Check for nested quantifiers that could cause catastrophic backtracking
            if trimmed.contains("*+") || trimmed.contains("++") || trimmed.contains("?+") {
                return Err(SearchError::InvalidPattern {
                    pattern: pattern.to_string(),
                    reason: "Pattern contains nested quantifiers that could cause catastrophic backtracking".to_string(),
                });
            }
        }

        // Sanitize the pattern by removing null bytes and special characters
        let sanitized = trimmed
            .chars() // Iterate over chars
            .filter(|ch| !ch.is_control() || *ch == '\t' || *ch == '\n')
            .collect();

        Ok(sanitized)
    }

    /// Validates file path
    pub fn validate_file_path(path: &str) -> Result<String> {
        let trimmed = path.trim();

        if trimmed.is_empty() {
            return Err(SearchError::InvalidInput(
                "File path cannot be empty".to_string(),
                ));
        }
        
        // Check for null bytes
        if trimmed.contains('\0') {
            return Err(SearchError::InvalidInput( 
                "File path contains null bytes".to_string(),
                ));
        }
        
        // Check path length
        if trimmed.len() > MAX_PATH_LENGTH {
            return Err(SearchError::InvalidInput(
                "File path is too long (max length is 4096 characters)".to_string(),
            ));
        }
        
        Ok(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_search_pattern() {
        assert!(InputValidator::validate_search_pattern("hello").is_ok());
        assert!(InputValidator::validate_search_pattern(" hello there ").is_ok());
        assert!(InputValidator::validate_search_pattern("regex.*pattern").is_ok());
        assert!(InputValidator::validate_search_pattern("regex_pattern").is_ok());
    }
    
    #[test]
    fn test_validate_search_pattern_invalid() {
        // Empty pattern
        assert!(InputValidator::validate_search_pattern("").is_err());
        assert!(InputValidator::validate_search_pattern("    ").is_err());
        assert!(InputValidator::validate_search_pattern("\t\n").is_err());
        
        // Pattern too long
        let long_pattern = "a".repeat(PATTERN_MAX_LENGTH + 1);
        assert!(InputValidator::validate_search_pattern(&long_pattern).is_err());
        
        // Pattern contains problematic characters
        let problematic = "*".repeat(MAX_PROBLEM_CHARS + 1);
        assert!(InputValidator::validate_search_pattern(&problematic).is_err());
    }
    
    #[test]
    fn test_validate_file_path() {
        assert!(InputValidator::validate_file_path("/absolute/path").is_ok());
        assert!(InputValidator::validate_file_path("relative/path").is_ok());
        assert!(InputValidator::validate_file_path("").is_err());
        assert!(InputValidator::validate_file_path("path\0with\0null\0bytes").is_err());
        
        let long_path = "/".repeat(MAX_PATH_LENGTH + 1);
        assert!(InputValidator::validate_file_path(&long_path).is_err());
    }
}
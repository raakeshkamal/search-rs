//! External dependency management.
//!
//! Checks for requuired external dependencies and provides
//! installation instructions.

use crate::{Result, SearchError};
use std::process::Command;

/// External tool dependencies required by the program.
pub struct Dependencies {
    pub ripgrep: bool,
}

// struct
impl Dependencies {
    /// Check if all required dependencies are installed.
    pub fn check(&self) -> Result<Self> {
        // Succeed and return self
        let deps = Dependencies {
            ripgrep: check_tool("rg"),
        };

        if !deps.all_present() {
            return Err(SearchError::MissingDependency {
                tool: deps.missing_tools().join(", "),
                install_instructions: deps.install_instructions(),
            });
        }
        Ok(deps)
    }

    /// Check if all required dependencies are installed.
    pub fn all_present(&self) -> bool {
        self.ripgrep
    }

    /// Get list of missing dependencies.
    pub fn missing_tools(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if !self.ripgrep {
            missing.push("ripgrep (rg)".to_string());
        }
        missing
    }

    /// Get installation instructions.
    pub fn install_instructions(&self) -> String {
        let mut install = Vec::new();
        if !self.ripgrep {
            install.push(get_ripgrep_install_instructions());
        }
        
        if install.is_empty() {
            return "All required tools are installed.".to_string();
        }
        format!("Install missing tools:\n{}", install.join("\n"))
    }
}

/// Check if all required external dependencies are installed.
fn check_tool(tool_name: &str) -> bool {
    Command::new(tool_name).arg("--version").output().is_ok()
}

/// Get installation instructions.
fn get_ripgrep_install_instructions() -> String {
    format!(
        " ripgrep (rg) is required to run this program.\n\
          Install ripgrep (rg) with your package manager or by running:\n\
          cargo install ripgrep\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_check_tool() {
        // Just check if Command function panics. This depends on OS
        let _ = check_tool("ls");
        let _ = check_tool("nonexistent_tool_12345");
    }
    
    #[test]
    fn test_missing_tools() {
        let deps = Dependencies {
            ripgrep: false,
        };
        assert!(!deps.all_present());
        let missing = deps.missing_tools();
        assert!(missing.iter().any(|tool| tool.contains("ripgrep")));
        
    }

    #[test]
    fn test_install_instructions() {
        let deps = Dependencies {
            ripgrep: false,
        };
        let hints = deps.install_instructions();
        assert!(hints.contains("ripgrep"));
        assert!(hints.contains("cargo install"));
    }
    
    #[test]
    fn test_all_present() {
        let deps = Dependencies {
            ripgrep: true,
        };
        assert!(deps.all_present());
        let hints = deps.install_instructions();
        assert!(hints.contains("All required tools are installed."));
    }
}
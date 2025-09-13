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
            missing.push(get_ripgrep_install_instructions());
        }
        missing
    }

    /// Get installation instructions.
    pub fn install_instructions(&self) -> String {
        let mut install = String::new();
        if !self.ripgrep {
            install.push_str("Install ripgrep (rg) with your package manager or by running:\n");
            install.push_str("cargo install ripgrep\n");
        }
        install
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

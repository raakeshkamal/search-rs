//! File preview integration module.
//!
//! Handles file preview functionality using direct file buffer reading

use crate::constants::*;
use crate::{Result, SearchError};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// File preview handler using direct file buffer reading
pub struct PreviewHandler;

impl PreviewHandler {
    /// Create a new preview handler
    pub fn new() -> Self {
        Self
    }

    /// Generate a preview for a file at specific line number with optional dimensions
    // AsRef allows us to accept a &Path or &str as input
    pub fn preview_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        line_number: Option<usize>,
        terminal_dimensions: Option<(usize, usize)>,
    ) -> Result<String> {
        let file_path = file_path.as_ref();

        if !file_path.exists() {
            return Err(SearchError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", file_path.display()),
            )));
        }

        // Calculate max lines from terminal dimensions
        let max_lines = terminal_dimensions
            .map(|(_, height)| height as usize)
            .unwrap_or(DEFAULT_TERMINAL_HEIGHT);

        // Open file and create buffer reader
        let file = File::open(file_path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);

            if let Some(target_line) = line_number {
                // When we have a target line, show context around it
                let context_before = max_lines / 2;

                // max 0 to max 1
                let start_line = target_line.saturating_sub(context_before).max(1);
                let required_width = MAX_LINE_NUM_DIGITS;

                // Use iterator chains for efficienct line processing with target line context
                let results: std::result::Result<String, std::io::Error> = reader
                    .lines()
                    .skip(start_line.saturating_sub(1))
                    .take(max_lines)
                    .enumerate()
                    .map(|(line_idx, line_result)| {
                        let line_num = start_line + line_idx;
                        let line = line_result?;
                        let marker = if line_num == target_line { ">" } else { " " };
                        Ok(format!(
                            "{:width$}{}| {}\n",
                            line_num,
                            marker,
                            line,
                            width = required_width
                        ))
                    })
                    .collect::<std::result::Result<Vec<String>, _>>() //  Collect the results into a single vector
                    .map(|lines| lines.join("")); // Join the lines into a single string

                results.map_err(SearchError::IoError)
            } else {
                // No target line, show from beginning
                let results: std::result::Result<String, std::io::Error> = reader
                    .lines()
                    .take(max_lines)
                    .enumerate()
                    .map(|(line_idx, line_result)| {
                        let line_num = line_idx + 1;
                        let line = line_result?;
                        Ok(format!(
                            "{:width$}| {}\n",
                            line_num,
                            line,
                            width = MAX_LINE_NUM_DIGITS
                        ))
                    })
                    .collect::<std::result::Result<Vec<String>, _>>() //  Collect the results into a single vector
                    .map(|lines| lines.join("")); // Join the lines into a single string

                results.map_err(SearchError::IoError)
            }
        } else {
            return Err(SearchError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", file_path.display()),
            )));
        }
    }
}

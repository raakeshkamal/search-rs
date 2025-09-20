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
            // this is not a condition, but a pattern matching
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

impl Default for PreviewHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to create test files with numbered lines
    fn create_test_file(path: &std::path::Path, line_count: usize) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        for i in 1..=line_count {
            writeln!(file, "Line {}", i)?;
        }
        Ok(())
    }

    // Helper function to create test files with custom content
    fn create_test_file_with_content(
        path: &std::path::Path,
        lines: &[&str],
    ) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        for &line in lines {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    #[test]
    fn test_preview_handler_creation_and_default() {
        // Test both creation methods in one test since they are functionally the same
        let _handler1 = PreviewHandler::new();
        let _handler2 = PreviewHandler::default();
    }

    #[test]
    fn test_preview_nonexistent_file() {
        let handler = PreviewHandler::new();

        // Test with various dimensions
        let result1 = handler.preview_file("nonexistent.txt", None, Some((80, 24)));
        let result2 = handler.preview_file("nonexistent.txt", Some(42), Some((120, 30)));

        assert!(result1.is_err());
        assert!(result2.is_err());

        // Both should return the same error
        assert!(matches!(result1, Err(SearchError::IoError(_))));
        assert!(matches!(result2, Err(SearchError::IoError(_))));
    }

    #[test]
    fn test_preview_basic_functionality() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // unwrap will panic if the file cannot be created
        create_test_file_with_content(&file_path, &["Line 1", "Line 2", "Line 3"]).unwrap();

        // Test without line number with standard dimensions
        // Some() is option with value
        // None is option with no value
        let preview_no_line = handler
            .preview_file(&file_path, None, Some((80, 24)))
            .unwrap();

        assert!(preview_no_line.contains("Line 1"));
        assert!(preview_no_line.contains("Line 2"));
        assert!(preview_no_line.contains("Line 3"));
        assert!(preview_no_line.contains("   1|"));
        assert!(preview_no_line.contains("   2|"));
        assert!(preview_no_line.contains("   3|"));
        assert!(!preview_no_line.contains(">")); // No line marker

        // Test line number highlighting with wider dimensions
        let preview_line = handler
            .preview_file(&file_path, Some(2), Some((120, 30)))
            .unwrap();

        assert!(preview_line.contains("Line 1"));
        assert!(preview_line.contains("Line 2"));
        assert!(preview_line.contains("Line 3"));
        assert!(preview_line.contains("   1 |"));
        assert!(preview_line.contains("   2>|"));
        assert!(preview_line.contains("   3 |"));
    }

    #[test]
    fn test_preview_edge_cases() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();

        // Test empty file with minimal dimensions
        let empty_file_path = temp_dir.path().join("empty.txt");
        File::create(&empty_file_path).unwrap();

        let preview_empty = handler
            .preview_file(&empty_file_path, None, Some((20, 5)))
            .unwrap();
        assert_eq!(preview_empty, "");

        // Test single line file with very large dimensions
        let single_line_file_path = temp_dir.path().join("single_line.txt");
        create_test_file_with_content(&single_line_file_path, &["Line 1"]).unwrap();

        let preview_single_line = handler
            .preview_file(&single_line_file_path, Some(1), Some((200, 100)))
            .unwrap();
        assert!(preview_single_line.contains("Line 1"));
        assert!(preview_single_line.contains("   1>|"));
        assert_eq!(preview_single_line.lines().count(), 1);
    }

    #[test]
    fn test_preview_line_number_out_of_bounds() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        create_test_file_with_content(&file_path, &["Line 1", "Line 2", "Line 3"]).unwrap();

        // Test line number out of bounds
        let preview_out_of_bounds = handler
            .preview_file(&file_path, Some(100), Some((80, 24)))
            .unwrap();

        assert!(!preview_out_of_bounds.contains("  10>|"));
    }

    #[test]
    fn test_preview_special_characters_and_unicode() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("special.txt");

        create_test_file_with_content(
            &file_path,
            &[
                "Line with accents: é è à ç ß € ® ©",
                "Line with quotes: \" and \" 'apostrophes'",
                "Line with emoji 🚀 🤔",
                "Line with tabs:\t\t\t\t\t\t\t\t\t",
                "Chinese: 中文 漢字 字体 字型 フォント 字型",
                "Math: π √2 ∫∫ ∀x∈ℝ ∃x∈ℝ ∃!x∈ℝ",
            ],
        )
        .unwrap();

        let preview = handler
            .preview_file(&file_path, Some(2), Some((80, 24)))
            .unwrap();

        // Test various special characters
        assert!(preview.contains("é"));
        assert!(preview.contains("\""));
        assert!(preview.contains("'apostrophes'"));
        assert!(preview.contains("🚀"));
        assert!(preview.contains("\t"));
        assert!(preview.contains("√2"));
        assert!(preview.contains("中文"));
        assert!(preview.contains("   2>|"));
    }

    #[test]
    fn test_preview_path_types() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        create_test_file_with_content(&file_path, &["Line 1", "Line 2", "Line 3"]).unwrap();

        // Test different path types
        let dims = Some((80, 24));
        let preview1 = handler
            .preview_file(&file_path, None, dims.clone())
            .unwrap();
        let path_buf = file_path.clone();
        let preview2 = handler.preview_file(path_buf, None, dims).unwrap();
        let path_str = file_path.to_string_lossy().to_string();
        let preview3 = handler.preview_file(path_str, None, dims).unwrap();
        let path_str = file_path.to_str().unwrap();
        let preview4 = handler.preview_file(path_str, None, dims).unwrap();

        // All should produce the same result
        assert!(preview1.contains("Line 1"));
        assert_eq!(preview1, preview2);
        assert_eq!(preview1, preview3);
        assert_eq!(preview1, preview4);
    }

    #[test]
    fn test_dimension_edge_cases() {
        let hander = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("dimension_test.txt");

        let lines: Vec<String> = (1..=100).map(|i| format!("Line {}", i)).collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        create_test_file_with_content(&file_path, &line_refs).unwrap();

        // Test minimum dimensions
        let preview_tiny = hander
            .preview_file(&file_path, Some(50), Some((1, 1)))
            .unwrap();
        assert_eq!(preview_tiny.lines().count(), 1);
        assert!(preview_tiny.contains("  50>|"));

        // Test exterme width and minimal height
        let preview_wide = hander
            .preview_file(&file_path, Some(25), Some((1000, 2)))
            .unwrap();
        assert_eq!(preview_wide.lines().count(), 2);
        assert!(preview_wide.contains("Line 25"));

        // Test minimum width and large height
        let preview_tall = hander
            .preview_file(&file_path, Some(10), Some((10, 1000)))
            .unwrap();
        assert!(preview_tall.lines().count() <= 100);
        assert!(preview_tall.contains("  10>|"));

        // Test square dimensions
        let preview_square = hander
            .preview_file(&file_path, Some(75), Some((100, 100)))
            .unwrap();
        let line_count = preview_square.lines().count();
        assert!(line_count <= 100 && line_count >= 75);
        assert!(preview_square.contains("Line 75"));

        // Test no target line with various dimensions
        let preview_no_target_small = hander
            .preview_file(&file_path, None, Some((50, 10)))
            .unwrap();
        let preview_no_target_large = hander
            .preview_file(&file_path, None, Some((100, 100)))
            .unwrap();

        assert_eq!(preview_no_target_small.lines().count(), 10);
        assert_eq!(preview_no_target_large.lines().count(), 100);
        assert!(preview_no_target_small.contains("Line 1"));
        assert!(preview_no_target_large.contains("Line 1"));
        assert!(!preview_no_target_large.contains(">"));
        assert!(!preview_no_target_small.contains(">"));
    }

    #[test]
    fn test_preview_binary_file() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("binary.bin");
        let mut file = File::create(&file_path).unwrap();

        // Write some random bytes
        for _ in 0..100 {
            file.write_all(&[rand::random::<u8>()]).unwrap();
        }

        // Should not panic
        let _preview = handler.preview_file(&file_path, None, Some((80, 24)));
    }

    #[test]
    fn test_preview_no_ansi_escape_sequences() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        create_test_file_with_content(
            &file_path,
            &[
                "fn main() {",
                "    println!(\"Hello, world!\");",
                "}",
                "",
                "// This is a comment",
                "",
                "fn another_function() {",
                "    // This is another comment",
                "    println!(\"Another function\");",
                "}",
            ],
        )
        .unwrap();

        // Test both with and without line number
        let preview_with_line = handler
            .preview_file(&file_path, Some(5), Some((80, 24)))
            .unwrap();
        let preview_without_line = handler
            .preview_file(&file_path, None, Some((80, 24)))
            .unwrap();

        // Test both with and without line number
        let ansi_escape_pattern = ["\x1b[0m", "\x1b[1m", "\x1b[4m", "\x1b[7m"];

        for pattern in ansi_escape_pattern {
            assert!(
                !preview_with_line.contains(pattern),
                "Found ANSI escape sequence in preview with line number"
            );
            assert!(
                !preview_without_line.contains(pattern),
                "Found ANSI escape sequence in preview without line number"
            );
        }

        // Verify content is still present
        assert!(preview_with_line.contains("fn main"));
        assert!(preview_without_line.contains("fn main"));
        assert!(preview_with_line.contains("println"));
        assert!(preview_without_line.contains("println"));
    }

    #[test]
    fn test_direct_file_reading_comprehensive() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();

        // Test basic functionality and formatting
        let basic_file_path = temp_dir.path().join("basic.rs");
        create_test_file_with_content(
            &basic_file_path,
            &[
                "fn main() {",
                "    println!(\"Hello, world!\");",
                "}",
                "",
                "// This is a comment",
                "",
                "fn another_function() {",
                "    // This is another comment",
                "    println!(\"Another function\");",
                "}",
            ],
        )
        .unwrap();

        let preview = handler
            .preview_file(&basic_file_path, Some(2), Some((80, 24)))
            .unwrap();

        assert!(preview.contains("fn main"));
        assert!(preview.contains("println"));
        assert!(preview.contains("Hello, world!"));
        assert!(preview.contains("   1 |"));
        assert!(preview.contains("   2>|"));
        assert!(preview.contains("   3 |"));
        assert!(preview.contains("   4 |"));

        // Test context around target line
        let large_file_path = temp_dir.path().join("large.rs");
        create_test_file(&large_file_path, 1000).unwrap();

        let preview = handler
            .preview_file(&large_file_path, Some(50), Some((80, 24)))
            .unwrap();

        assert!(preview.contains("Line 50"));
        assert!(preview.contains("  50>|"));
        assert_eq!(preview.lines().count(), 24);
        assert!(preview.contains("Line 40") || preview.contains("Line 41")); // before context
        assert!(preview.contains("Line 59") || preview.contains("Line 60")); // after context
    }

    #[test]
    fn test_performance_and_large_files() {
        use std::time::Instant;
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();

        // Test performance with large files
        let large_file_path = temp_dir.path().join("large.txt");
        create_test_file(&large_file_path, 1000).unwrap();

        let start = Instant::now();
        let preview = handler
            .preview_file(&large_file_path, Some(50), Some((80, 24)))
            .unwrap();
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100, "Should be fast {:?}", duration); // Less than 100ms
        assert!(preview.contains("Line 50"));
        assert!(preview.contains("  50>|"));
        assert_eq!(preview.lines().count(), 24);

        // Test memory usage with very large files
        let very_large_file_path = temp_dir.path().join("very_large.txt");
        create_test_file(&very_large_file_path, 10_000).unwrap();

        let large_preview = handler
            .preview_file(&very_large_file_path, Some(8000), Some((80, 24)))
            .unwrap();

        assert_eq!(large_preview.lines().count(), 24);
        assert!(large_preview.contains("Line 8000"));
        assert!(large_preview.contains("8000>|"));

        // Should contain context but not early lines(memory efficiency)
        assert!(large_preview.contains("Line 7998") || large_preview.contains("Line 7999"));
        assert!(large_preview.contains("Line 8001") || large_preview.contains("Line 8002"));
        assert!(!large_preview.contains("Line 1"));
        assert!(!large_preview.contains("Line 200"));
    }

    #[test]
    fn test_context_management_edge_cases() {
        let handler = PreviewHandler::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        create_test_file(&file_path, 1000).unwrap();

        // Test target line at file beginning with limited height
        let beginning_preview = handler
            .preview_file(&file_path, Some(1), Some((80, 10)))
            .unwrap();

        assert!(beginning_preview.contains("Line 1"));
        assert!(beginning_preview.contains("   1>|"));
        assert_eq!(beginning_preview.lines().count(), 10);

        // Test target line at file end with limited height
        let end_preview = handler
            .preview_file(&file_path, Some(1000), Some((80, 10)))
            .unwrap();

        assert!(
            end_preview.contains("Line 1000")
                || end_preview.contains("Line 999")
                || end_preview.contains("Line 998")
        );
        assert!(end_preview.contains("1000>|"));
        assert!(end_preview.lines().count() <= 10);

        // Test middle target with assymetric height
        let middle_preview = handler
            .preview_file(&file_path, Some(500), Some((80, 20)))
            .unwrap();

        assert!(middle_preview.contains("Line 500"));
        assert!(middle_preview.contains(" 500>|"));
        assert_eq!(middle_preview.lines().count(), 20);

        // Should have roughly equal context before and after
        let lines: Vec<&str> = middle_preview.lines().collect();
        let target_pos = lines.iter().position(|line| line.contains(">")).unwrap();
        assert!(target_pos >= 9 && target_pos <= 11);
    }
}

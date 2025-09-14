//! Search engine implementations.
//!
//! Defines different search modes (exact, case-insensitive, substring)
//! and handles ripgrep command generation

use crate::{cli::Cli, Result};

/// Search Engine that configures ripgrep based on search mode
#[derive(Debug, Clone)]
pub struct SearchEngine {
    pub mode: SearchEngineMode,
    pub file_types: Vec<String>,
}

/// Search Engine Mode
#[derive(Debug, Clone)]
pub enum SearchEngineMode {
    /// Exact whole-word search (case-sensitive)
    Exact,
    /// Case-insensitive whole-word search
    CaseInsensitive,
    /// Substring search (case-sensitive)
    Substring,
}

impl SearchEngine {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        Self::from_cli_with_config(cli)
    }

    pub fn from_cli_with_config(cli: &Cli) -> Result<Self> {
        let mode = if cli.exact {
            SearchEngineMode::Exact
        } else if cli.ignore_case {
            SearchEngineMode::CaseInsensitive
        } else if cli.substring {
            SearchEngineMode::Substring
        } else {
            SearchEngineMode::CaseInsensitive
        };

        let file_types = vec![];

        Ok(Self { mode, file_types })
    }

    /// Generates the ripgrep command based on the search mode
    pub fn generate_rg_args(&self, pattern: &str, directory: Option<&str>) -> Vec<String> {
        crate::logging::debug_log(&format!("Generating ripgrep args for pattern: {}", pattern));
        let mut args = Vec::new();

        let search_pattern = pattern.to_string();

        // Add search mode-specific flags
        match &self.mode {
            SearchEngineMode::Exact => {
                args.push("--word-regexp".to_string());
                args.push("--case-sensitive".to_string());
            }
            SearchEngineMode::CaseInsensitive => {
                args.push("--ignore-case".to_string());
            }
            SearchEngineMode::Substring => {
                args.push("--case-sensitive".to_string());
            }
        }

        // Add common flags
        args.push("--line-number".to_string());
        args.push("--no-heading".to_string());
        args.push("--with-filename".to_string());

        // Add file type specifications only if file types are specified
        if !self.file_types.is_empty() {
            for file_type in &self.file_types {
                args.push(format!("--type-add=custom:*.{}", file_type));
            }
            args.push("--type=custom".to_string());
        }

        // Add search pattern
        args.push(search_pattern);

        // Add directory if specified
        if let Some(directory) = directory {
            args.push(directory.to_string());
        } else {
            args.push(".".to_string());
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::cli::Cli;

    // Helper function to create CLI
    fn create_cli(exact: bool, ignore_case: bool, substring: bool) -> Cli {
        Cli {
            pattern: "test".to_string(),
            exact,
            ignore_case,
            substring,
            directory: None,
            debug: false,
        }
    }

    // Helper function to create SearchEngine
    fn create_engine(mode: SearchEngineMode, file_types: Vec<&str>) -> SearchEngine {
        // .collect will create Vec<String> from Vec<&str>
        SearchEngine {
            mode,
            file_types: file_types.iter().map(|s| s.to_string()).collect(),
        }
    }

    // Helper function to assert common flags are present
    fn assert_common_flags(args: &[String]) {
        let common_flags = ["--line-number", "--no-heading", "--with-filename"];
        for flag in common_flags {
            // helpful error message if assertion fails
            assert!(
                args.contains(&flag.to_string()),
                "Missing common flag: {}",
                flag
            );
        }
    }

    // Helper function to assert file type args
    fn assert_file_type_args(args: &[String], file_types: &[&str]) {
        if file_types.is_empty() {
            assert!(!args.iter().any(|arg| arg.starts_with("--type-add=custom;")));
            assert!(!args.contains(&"--type=custom".to_string()));
        } else {
            for file_type in file_types {
                assert!(args.contains(&format!("--type-add=custom:*.{}", file_type)));
            }
            assert!(args.contains(&"--type=custom".to_string()));
        }
    }

    #[test]
    fn test_searchengine_from_exact_mode() {
        let cli = create_cli(true, false, false);

        let search_engine = SearchEngine::from_cli(&cli).unwrap();
        matches!(search_engine.mode, SearchEngineMode::Exact);
    }

    #[test]
    fn test_searchengine_mode_selection() {
        let test_cases = vec![
            (true, false, false, SearchEngineMode::Exact),
            (false, false, false, SearchEngineMode::CaseInsensitive), // default
            (false, true, false, SearchEngineMode::CaseInsensitive),
            (false, false, true, SearchEngineMode::Substring),
        ];

        for (exact, ignore_case, substring, expected_mode) in test_cases {
            let cli = create_cli(exact, ignore_case, substring);
            let search_engine = SearchEngine::from_cli(&cli).unwrap();
            match expected_mode {
                SearchEngineMode::Exact => {
                    assert!(matches!(search_engine.mode, SearchEngineMode::Exact))
                }
                SearchEngineMode::CaseInsensitive => assert!(matches!(
                    search_engine.mode,
                    SearchEngineMode::CaseInsensitive
                )),
                SearchEngineMode::Substring => {
                    assert!(matches!(search_engine.mode, SearchEngineMode::Substring))
                }
            }
        }
    }

    #[test]
    fn test_rg_args_by_mode() {
        let test_cases = vec![
            (
                SearchEngineMode::Exact,
                vec!["--word-regexp", "--case-sensitive"],
                vec!["--ignore-case"],
            ),
            (
                SearchEngineMode::CaseInsensitive,
                vec!["--ignore-case"],
                vec!["--case-sensitive", "--word-regexp"],
            ),
            (
                SearchEngineMode::Substring,
                vec!["--case-sensitive"],
                vec!["--word-regexp", "--ignore-case"],
            ),
        ];

        for (mode, should_contain, should_not_contain) in test_cases {
            let engine = create_engine(mode.clone(), vec!["rs"]);
            let args = engine.generate_rg_args("pattern", Some("src/"));

            // check mode-specific flags
            for flag in should_contain {
                assert!(
                    args.contains(&flag.to_string()),
                    "Mode {:?} should contain flag: {}",
                    mode,
                    flag
                );
            }
            for flag in should_not_contain {
                assert!(
                    !args.contains(&flag.to_string()),
                    "Mode {:?} should not contain flag: {}",
                    mode,
                    flag
                );
            }

            // check common args
            assert_common_flags(&args);
            assert!(args.contains(&"pattern".to_string()));
            assert!(args.contains(&"src/".to_string()));
            assert_file_type_args(&args, &["rs"]);
        }
    }

    #[test]
    fn test_search_engine_from_cli_all_combinations() {
        let test_cases = vec![
            (false, false, false, SearchEngineMode::CaseInsensitive),
            (true, false, false, SearchEngineMode::Exact),
            (false, true, false, SearchEngineMode::CaseInsensitive),
            (false, false, true, SearchEngineMode::Substring),
            (true, true, false, SearchEngineMode::Exact), // exact wins
            (true, false, true, SearchEngineMode::Exact), // exact wins
            (false, true, true, SearchEngineMode::CaseInsensitive), // ignore case wins
            (true, true, true, SearchEngineMode::Exact),  // exact wins
        ];

        for (exact, ignore_case, substring, expected_mode) in test_cases {
            let cli = create_cli(exact, ignore_case, substring);
            let search_engine = SearchEngine::from_cli(&cli).unwrap();
            match expected_mode {
                SearchEngineMode::Exact => {
                    assert!(
                        matches!(search_engine.mode, SearchEngineMode::Exact),
                        "Failed for exact: {}, ignore_case: {}, substring: {}",
                        exact,
                        ignore_case,
                        substring
                    );
                }
                SearchEngineMode::CaseInsensitive => {
                    assert!(
                        matches!(search_engine.mode, SearchEngineMode::CaseInsensitive),
                        "Failed for exact: {}, ignore_case: {}, substring: {}",
                        exact,
                        ignore_case,
                        substring
                    );
                }
                SearchEngineMode::Substring => {
                    assert!(
                        matches!(search_engine.mode, SearchEngineMode::Substring),
                        "Failed for exact: {}, ignore_case: {}, substring: {}",
                        exact,
                        ignore_case,
                        substring
                    );
                }
            }
        }
    }

    // Test file type handling across different modes
    #[test]
    fn test_file_type_handling() {
        let file_types = vec!["rs", "py", "js"];
        let modes = vec![
            SearchEngineMode::Exact,
            SearchEngineMode::CaseInsensitive,
            SearchEngineMode::Substring,
        ];

        for mode in modes {
            let engine = create_engine(mode, file_types.clone());
            let args = engine.generate_rg_args("pattern", Some("src/"));
            assert_file_type_args(&args, &file_types);
            assert!(args.contains(&"pattern".to_string()));
            assert!(args.contains(&"src/".to_string()));
        }
    }
    
    #[test]
    fn test_search_engine_empty_file_types() {
        let cli = create_cli(false, true, false);
        let engine = SearchEngine::from_cli(&cli).unwrap();
        assert!(engine.file_types.is_empty());
        
        let args = engine.generate_rg_args("pattern", Some("src/"));
        assert_file_type_args(&args, &[]);
    }
    
    // Test directory handling
    #[test]
    fn test_rg_args_directory_handling() {
        let engine = create_engine(SearchEngineMode::CaseInsensitive, vec!["rs"]);
        
        // Test with directory
        let args = engine.generate_rg_args("pattern", Some("src/"));
        assert!(args.contains(&"src/".to_string()));
        assert!(!args.contains(&".".to_string()));
        
        // Test without directory
        let args = engine.generate_rg_args("pattern", None);
        assert!(args.contains(&".".to_string()));
        assert!(!args.contains(&"src/".to_string()));
    }
    
    // Test pattern handling
    #[test]
    fn test_rg_args_pattern_handling() {
        let engine = create_engine(SearchEngineMode::CaseInsensitive, vec!["rs"]);
        let args = engine.generate_rg_args("pattern", Some("src/"));
        assert!(args.contains(&"pattern".to_string()));
        assert_common_flags(&args);
    }
    
    // Test special characters in paths
    #[test]
    fn test_special_characters_in_paths() {
        let engine = create_engine(SearchEngineMode::CaseInsensitive, vec!["rs"]);
        let special_chars = vec![
            "path with spaces/file.rs",
            "path-with-hyphens/file.rs",
            "path.with.dots/file.rs",
            "path_with_underscores/file.rs",
        ];
        
        for dir in special_chars {
            let args = engine.generate_rg_args("pattern", Some(dir));
            assert!(args.contains(&dir.to_string()), "Failed for dir: {}", dir);
        }
    }
    
    // Test Debug and Clone traits
    #[test]
    fn test_search_engine_triats() {
        let engine = create_engine(SearchEngineMode::CaseInsensitive, vec!["rs"]);
        
        // Test Clone
        let cloned = engine.clone();
        assert!(matches!(cloned.mode, SearchEngineMode::CaseInsensitive));
        assert_eq!(cloned.file_types, vec!["rs"]);
        
        // Test Debug
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("SearchEngine"));
        assert!(debug_str.contains("CaseInsensitive"));
        assert!(debug_str.contains("rs"));
        
        // Test mode debug
        for mode in vec![SearchEngineMode::Exact, SearchEngineMode::CaseInsensitive, SearchEngineMode::Substring] {
            let debug_str = format!("{:?}", mode);
            assert!(!debug_str.is_empty());
        }
    }
}

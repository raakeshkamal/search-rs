//! TUI application state and event handling

use crate::preview::PreviewHandler;
use crate::search::sorter::FileSorter;
use crate::search::{ProgressiveLoadStatus, SearchResult};
use crate::tui::highlighter::SyntaxHighlighter;
use ratatui::text::Line;
use std::cell::RefCell;
use std::collections::HashMap;

/// Input focus state for search interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFocus {
    /// Primary search box is focused
    Primary,
    /// Results list is focused
    Results,
}

/// Search progress state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchProgress {
    /// Number of files with matches found so far
    pub files_with_matches: usize,
    /// Whether the search is currently in progress
    pub is_searching: bool,
    /// Whether the search is complete
    pub is_complete: bool,
}

impl SearchProgress {
    /// Create a new search progress state
    pub fn new() -> Self {
        Self {
            files_with_matches: 0,
            is_searching: false,
            is_complete: false,
        }
    }

    /// Start a new search
    pub fn start_search(&mut self) {
        self.files_with_matches = 0;
        self.is_searching = true;
        self.is_complete = false;
    }

    /// Update the search progress with current file count
    pub fn update_file_count(&mut self, file_with_matches: usize) {
        self.files_with_matches = file_with_matches;
    }

    /// Mark the search as complete
    pub fn complete_search(&mut self) {
        self.is_searching = false;
        self.is_complete = true;
    }

    /// Reset the search progress
    pub fn reset(&mut self) {
        self.files_with_matches = 0;
        self.is_searching = false;
        self.is_complete = false;
    }
}

impl Default for SearchProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Main TUI application state
pub struct App {
    /// Current search results
    pub search_results: Vec<SearchResult>,

    /// Currently selected search result index
    pub selected_index: usize,

    /// Current search pattern
    pub current_pattern: String,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Current input focus state
    pub input_focus: InputFocus,

    /// preview handler for the file content
    pub preview_handler: PreviewHandler,

    /// Search progress tracking
    pub search_progress: SearchProgress,

    /// Progressive load status
    pub progressive_load_status: Option<ProgressiveLoadStatus>,

    /// Flag to trigger progressive loading check
    pub needs_progressive_load_check: bool,

    /// Cache for syntax-highlighted results to avoid re-processing
    /// Key: (file_path, line_number, line_content) hash, Value: syntax-highlighted line
    // Refcell smart pointer moves borrowing checks to runtime
    // allows mutability of contents while ensuring safety
    highlighted_cache: RefCell<HashMap<u64, Line<'static>>>, // static lifetime makes the memory persist

    /// Cache size limit to prevent unlimited memory usage
    cache_size_limit: usize,

    /// File sorter for maintaining global sort order
    sorter: FileSorter,
}

impl App {
    /// Crete new application instance
    pub fn new() -> Self {
        Self {
            search_results: Vec::new(),
            selected_index: 0,
            current_pattern: String::new(),
            should_quit: false,
            input_focus: InputFocus::Primary,
            preview_handler: PreviewHandler::new(),
            search_progress: SearchProgress::new(),
            progressive_load_status: None,
            needs_progressive_load_check: false,
            highlighted_cache: RefCell::new(HashMap::new()),
            cache_size_limit: 1000,
            sorter: FileSorter::new(),
        }
    }

    /// Update search results (replace all results)
    pub fn update_search_results(&mut self, results: Vec<SearchResult>) {
        self.search_results = results.clone();
        self.selected_index = 0;

        // update sorter
        self.sorter.clear();
        if !results.is_empty() {
            let _ = self.sorter.add_results(results);
        }
    }

    /// Add a new search results (for streamng results) - maintains sort order
    pub fn add_search_result(&mut self, result: SearchResult) {
        // Let the sorter handle the insertion and maintain the master list
        let _ = self.sorter.add_results(vec![result]);

        // Sync our display with the sorter's sorted list
        self.sync_results_from_sorter();
    }

    /// Add multiple search results (for streamng results) - maintains sort order
    pub fn add_sarch_results(&mut self, results: Vec<SearchResult>) {
        if results.is_empty() {
            return;
        }

        // Let the sorter handle the insertion and maintain the master list
        let _ = self.sorter.add_results(results);

        // Sync our display with the sorter's sorted list
        self.sync_results_from_sorter();
    }

    /// Sync the results from the sorter to the display
    fn sync_results_from_sorter(&mut self) {
        self.search_results = self.sorter.get_all_results().to_vec()
    }

    /// Clear all search results (when starting a new search)
    pub fn clear_search_results(&mut self) {
        self.search_results.clear();
        self.selected_index = 0;
        self.sorter.clear();
        self.clear_highlighting_cache();
    }

    /// Start a new search
    pub fn start_new_search(&mut self) {
        self.clear_search_results();
        self.search_progress.start_search();
    }

    /// Update file counts with matches
    pub fn update_file_count(&mut self, file_with_matches: usize) {
        self.search_progress.update_file_count(file_with_matches);
    }

    /// Complete the current search
    pub fn complete_search(&mut self) {
        self.search_progress.complete_search();
    }

    /// Get currently selected search result
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.search_results.get(self.selected_index)
    }

    /// Get the search results
    pub fn active_results(&self) -> &Vec<SearchResult> {
        &self.search_results
    }

    /// Toggle input focus
    pub fn toggle_focus(&mut self) {
        match self.input_focus {
            InputFocus::Primary => self.input_focus = InputFocus::Results,
            InputFocus::Results => self.input_focus = InputFocus::Primary,
        }
    }

    /// Set quit flag
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Update search pattern
    pub fn update_pattern(&mut self, pattern: String) {
        self.current_pattern = pattern;
    }

    /// Get preview content for the currently selected result with optional terminal dimensions
    pub fn get_preview_content(&self, terminal_dimensions: Option<(usize, usize)>) -> String {
        if let Some(result) = self.selected_result() {
            match self.preview_handler.preview_file(
                &result.file_path,
                Some(result.line_number),
                terminal_dimensions,
            ) {
                Ok(preview) => preview,
                Err(e) => format!("Error Loading Preview: {:?}", e),
            }
        } else {
            "No file selected".to_string()
        }
    }

    /// Handle mouse click within the results list
    /// Returns true if the click resulted in selection change
    pub fn handle_results_click(
        &mut self,
        click_row: u16,
        results_area_top: u16,
        results_area_height: u16,
    ) -> bool {
        // Calculate which result was clicked based on the click position
        // results_area_top is the top of the results list (after header)
        // Each result takes up exactly one row in the list

        if click_row < results_area_top || click_row >= results_area_top + results_area_height {
            return false; // Click was outside of the results list
        }

        let click_index = (click_row - results_area_top) as usize;
        if click_index >= self.search_results.len() {
            self.selected_index = click_index;
            true
        } else {
            false
        }
    }

    /// Set selection to a specific index
    pub fn select_iindex(&mut self, index: usize) {
        if index < self.search_results.len() {
            self.selected_index = index;
        }
    }

    /// Get a reference to the search pattern
    pub fn active_pattern(&self) -> &str {
        &self.current_pattern
    }

    /// Get a reference to the search pattern
    pub fn active_pattern_mut(&mut self) -> &mut String {
        &mut self.current_pattern
    }

    /// Get progressive loading status for display
    pub fn get_progressive_load_status(&self) -> Option<&ProgressiveLoadStatus> {
        self.progressive_load_status.as_ref()
    }

    /// Override select_next to trigger progressive loading
    pub fn select_next(&mut self) {
        if !self.active_results().is_empty()
            && self.selected_index < self.active_results().len() - 1
        {
            self.selected_index += 1;
            // Request progressive loading check when navigating down
            self.needs_progressive_load_check = true;
        }
    }

    /// Override select_prev to trigger progressive loading
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Also check when navigating up
            self.needs_progressive_load_check = true;
        }
    }

    /// Get loading progress message for display
    pub fn get_loading_message(&self) -> String {
        if let Some(status) = &self.progressive_load_status {
            if status.loading_complete {
                format!(
                    "Loaded {} results from {} files",
                    status.total_loaded, status.total_files_found
                )
            } else {
                format!(
                    "Loading... {} results from {} files",
                    status.total_loaded, status.total_files_found
                )
            }
        } else if self.search_progress.is_searching {
            format!(
                "Searching... {} files found",
                self.search_progress.files_with_matches
            )
        } else if self.search_progress.is_complete {
            format!(
                "Search complete - {} files",
                self.search_progress.files_with_matches
            )
        } else {
            "Ready to search".to_string()
        }
    }

    /// Get a cached highlighted line or compute and cache it
    pub fn get_cached_highlighted_line(
        &self,
        result: &SearchResult,
        highlighter: &mut SyntaxHighlighter,
    ) -> Line<'static> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create cache key from result data
        let mut hasher = DefaultHasher::new();
        result.file_path.hash(&mut hasher);
        result.line_number.hash(&mut hasher);
        result.line_content.hash(&mut hasher);
        let cache_key = hasher.finish();

        // Check cache first
        if let Some(cached_line) = self.highlighted_cache.borrow().get(&cache_key) {
            return cached_line.clone();
        }

        // Not in cache, compute and cache
        let highlighted_line = result.format_for_tui_display(highlighter);

        // Manage cache size and insert
        {
            let mut cache = self.highlighted_cache.borrow_mut();

            // Manage cache size before inserting
            if cache.len() >= self.cache_size_limit {
                // Remove oldest entries - not exactly LRU
                let keys_to_remove: Vec<u64> = cache
                    .keys()
                    .take(self.cache_size_limit / 4)
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }

            cache.insert(cache_key, highlighted_line.clone());
        }
        highlighted_line
    }

    /// Clear the highlighting cache (called when starting a new search)
    pub fn clear_highlighting_cache(&mut self) {
        self.highlighted_cache.borrow_mut().clear();
    }

    /// Get cache stats for debugging
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.highlighted_cache.borrow().len(), self.cache_size_limit)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

//! Modified-time based sorting for serach results.
//!
//! Implements sorting based on file modification time using git line history
//! Most recently modified lines are prioritized in search results.

use super::SearchResult;
use git2::Repository;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Sorts search results based on file modification time using git line history and file metadata
pub struct FileSorter {
    /// Whether sorting is enabled
    enabled: bool,
    /// global sorted results maintained across all modules
    global_results: Vec<SearchResult>,
    /// metadata cache to avoid re-reading file metadata
    metadata_cache: HashMap<String, SystemTime>,
    /// Git repository for line history (if available)
    git_repo: Option<Repository>,
}

impl std::fmt::Debug for FileSorter {
    // Foratter with anonymous lifetime
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSorter")
            .field("enabled", &self.enabled)
            .field("global_results", &self.global_results.len())
            .field("metadata_cache", &self.metadata_cache.len())
            .field(
                "git_repo",
                &self.git_repo.as_ref().map(|_| "Repository(...)"),
            ) // .as_ref() converts &Option<T> to Option<&T>
            .finish()
    }
}

impl Clone for FileSorter {
    fn clone(&self) -> Self {
        // Git repository needs to be reopened for the clone
        let git_repo = {
            // If the is valid git repo, get reference to it
            if let Some(ref repo) = self.git_repo {
                if let Some(workdir) = repo.workdir() {
                    // returns either Some(Repository) or None
                    Repository::open(workdir).ok()
                } else {
                    None
                }
            } else {
                None
            }
        };

        Self {
            enabled: self.enabled,
            global_results: self.global_results.clone(),
            metadata_cache: self.metadata_cache.clone(),
            git_repo,
        }
    }
}

impl Default for FileSorter {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSorter {
    /// Creates a new file sorter
    pub fn new() -> Self {
        let git_repo = Repository::open(".").ok();

        Self {
            enabled: false,
            global_results: Vec::new(),
            metadata_cache: HashMap::new(),
            git_repo: git_repo,
        }
    }

    /// Enables sorting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Checks if sorting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all sorted results and metadata cache
    pub fn clear(&mut self) {
        self.global_results.clear();
        self.metadata_cache.clear();
    }

    /// Get the current count of sorted results
    pub fn len(&self) -> usize {
        self.global_results.len()
    }

    /// Check if the sorter has any results
    pub fn is_empty(&self) -> bool {
        self.global_results.is_empty()
    }

    /// Get reference to the global results
    pub fn get_all_results(&self) -> &Vec<SearchResult> {
        &self.global_results
    }

    /// Get the file modification time of a line using git history (with caching)
    fn get_modification_time(&mut self, result: &SearchResult) -> SystemTime {
        let cache_key = format!("{}:{}", result.file_path, result.line_number);
        if let Some(mtime) = self.metadata_cache.get(&cache_key) {
            return *mtime;
        }

        let mtime = self.get_git_line_modification_time(&result.file_path, result.line_number)
            .unwrap_or_else(||{
                // Fallback to file metadata if git line history is unavailable
                // and_then is daisy-chained only if first operation is successful the second one is executed
                fs::metadata(&result.file_path)
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            });
        
        // Cache the result
        self.metadata_cache.insert(cache_key, mtime);

        mtime
    }

    /// Get git line modification time using blame
    fn get_git_line_modification_time(&self, file_path: &str, line_number: usize) -> Option<SystemTime> {
        let repo = self.git_repo.as_ref()?;

        // Convert absolute path to relative path within git repo
        let workdir = repo.workdir()?;
        let file_path = Path::new(file_path);
        let relative_path = if file_path.is_absolute() {
            file_path.strip_prefix(workdir).ok()?
        } else {
            file_path
        };

        // Get blame for file
        let blame = repo.blame_file(relative_path, None).ok()?;

        // Git uses 1-based line numbers
        let line_idx = line_number.saturating_sub(1);

        // Get the hunk that contains the line
        let hunk = blame.get_line(line_idx)?;

        // Get the commit that modified the line
        let commit_oid = hunk.final_commit_id();
        let commit = repo.find_commit(commit_oid).ok()?;

        // Convert git time to SystemTime
        let git_time = commit.time();
        let timestamp = git_time.seconds();

        // Convert to SystemTime
        if timestamp >=0{
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64))
        } else {
            // Handles negative timestamps (before epoch)
            let duration = std::time::Duration::from_secs(-timestamp as u64);
            SystemTime::UNIX_EPOCH.checked_sub(duration)
        }
    }
    
    /// Add new results to the global sorted collection
    /// Retunrs only the newly added results in their correct sorted positions
    pub fn add_results(&mut self, mut new_results: Vec<SearchResult>) -> Vec<SearchResult> {
        if(!self.enabled || new_results.is_empty()){
            self.global_results.extend(new_results.clone());
            return new_results;
        }
        
        // Pre-populate metadata cache for the new results
        for result in &new_results {
            self.get_modification_time(result);
        }
        
        // Sort the new batch internally first
        self.sort_results(&mut new_results);
        
        // If global results are empty, just add the new results
        if self.global_results.is_empty() {
            self.global_results = new_results.clone();
            return new_results;
        }
        
        // Merge the sorted results with the global results
        self.merge_sorted_results(new_results.clone());
        
        // Return the newly added results
        new_results
    }
    
    /// Merge a sorted batch of results with the global results
    fn merge_sorted_results(&mut self, sorted_batch: Vec<SearchResult>) {
        let mut merged = Vec::with_capacity(self.global_results.len() + sorted_batch.len());
        let mut i = 0;
        let mut j = 0;
        
        // Merge the two sorted batches
        while i < self.global_results.len() && j < sorted_batch.len() {
            let global_result = &self.global_results[i];
            let batch_result = &sorted_batch[j];
            
            if self.compare_results(global_result, batch_result) == std::cmp::Ordering::Equal {
                merged.push(self.global_results[i].clone());
                i += 1;
            } else {
                merged.push(sorted_batch[j].clone());
                j += 1;
            }
        }
        
        // Add the remaining results from either array
        while i < self.global_results.len() {
            merged.push(self.global_results[i].clone());
            i += 1;
        }
        while j < sorted_batch.len() {
            merged.push(sorted_batch[j].clone());
            j += 1;
        }
        
        self.global_results = merged;
    }
    
    /// Merge a sorted batch of results with the global results
    fn merge_sorted_results_mut(&mut self, sorted_batch: Vec<SearchResult>) {
        let mut merged = Vec::with_capacity(self.global_results.len() + sorted_batch.len());
        let mut i = 0;
        let mut j = 0;
        
        // Merge the two sorted batches
        while i < self.global_results.len() && j < sorted_batch.len() {
            let global_result = &self.global_results[i];
            let batch_result = &sorted_batch[j];
            
            if self.compare_results(global_result, batch_result) == std::cmp::Ordering::Equal {
                merged.push(self.global_results[i].clone());
                i += 1;
            } else {
                merged.push(sorted_batch[j].clone());
                j += 1;
            }
        }
        
        // Add the remaining results from either array
        while i < self.global_results.len() {
            merged.push(self.global_results[i].clone());
        }
    }
    
    /// Compares two search results based on sorting criteria
    fn compare_results(&self, a: &SearchResult, b: &SearchResult) -> std::cmp::Ordering {
        let cache_key_a = format!("{}:{}", a.file_path, a.line_number);
        let cache_key_b = format!("{}:{}", b.file_path, b.line_number);
        
        let mtime_a = self.metadata_cache.get(&cache_key_a).unwrap();
        let mtime_b = self.metadata_cache.get(&cache_key_b).unwrap();
        
        // Sort by modification time (most recently modified first)
        mtime_b.cmp(mtime_a)
    }
    
    
    /// Sorts the results
    fn sort_results(&mut self, results: &mut [SearchResult]) { // &mut [] does not allow you to change its size
        results.sort_by(|a,b| self.compare_results(a,b));
    }
    
    
}

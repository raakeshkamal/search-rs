//! Logging module for debug mode
//!
//! Provides logging module that writes to /tmp file
//! with timestamps when --debug is specified

use log::{debug, error, info, trace, warn};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Once;

// run once in a singke thread. this prevents race conditions
static INIT: Once = Once::new();

/// Initializes logging module when debug mode is enabled
/// Creates a file in /tmp directory and sets up logger with timestamps
pub fn init_debug_logging() -> crate::Result<PathBuf> {
    let mut log_path = std::env::temp_dir();
    log_path.push("search-rs-debug.log");

    // Create or truncate the log file
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| crate::SearchError::FileAccessError {
            path: log_path.to_string_lossy().to_string(),
            reason: format!("Failed to create log file: {}", e),
        })?;

    // Initialize env_logger to write log file
    INIT.call_once(move || {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug) // debug and above
            .filter_module("crossterm", log::LevelFilter::Warn)
            .filter_module("ratatui", log::LevelFilter::Warn)
            .target(env_logger::Target::Pipe(Box::new(log_file))) // pipe console to file
            .format(|buf, record| { // format log message
                writeln!(
                    buf,
                    "{} [{}] {}:{} - {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S.%3f UTC"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
    });
    
    // display works for path in all OSes
    info!("Debug logging initialized to: {}", log_path.display());

    Ok(log_path)
}

/// Log a debug message if debug mode is enabled
pub fn debug_log(msg: &str) {
    debug!("{}",msg);
}

/// Log an info message if debug mode is enabled
pub fn info_log(msg: &str) {
    info!("{}",msg);
}

/// Log a warning message if debug mode is enabled
pub fn warn_log(msg: &str) {
    warn!("{}",msg);
}

/// Log an error message if debug mode is enabled
pub fn error_log(msg: &str) {
    error!("{}",msg);
}

/// Log a trace message if debug mode is enabled
pub fn trace_log(msg: &str) {
    trace!("{}",msg);
}
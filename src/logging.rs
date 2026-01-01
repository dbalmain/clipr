use anyhow::{Context, Result};
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Flash message for TUI display
#[derive(Debug, Clone)]
pub struct FlashMessage {
    pub level: Level,
    pub message: String,
    pub timestamp: Instant,
}

/// Custom logger that writes to both file and optional flash message channel
struct CliprLogger {
    file_writer: Arc<Mutex<RollingFileAppender>>,
    flash_tx: Option<Arc<Mutex<Sender<FlashMessage>>>>,
    file_level: LevelFilter,
    flash_level: LevelFilter,
}

impl Log for CliprLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.file_level || metadata.level() <= self.flash_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = format!("{}", record.args());
        let level = record.level();
        let timestamp = chrono::Local::now();

        // Write to file if level is enabled
        if level <= self.file_level {
            if let Ok(mut writer) = self.file_writer.lock() {
                let _ = writeln!(
                    writer,
                    "{} [{}] {}",
                    timestamp.format("%Y-%m-%d %H:%M:%S"),
                    level,
                    message
                );
            }
        }

        // Send to flash message channel if level is enabled and channel exists
        if level <= self.flash_level {
            if let Some(tx) = &self.flash_tx {
                if let Ok(tx) = tx.lock() {
                    let _ = tx.send(FlashMessage {
                        level,
                        message,
                        timestamp: Instant::now(),
                    });
                }
            }
        }
    }

    fn flush(&self) {
        // RollingFileAppender handles flushing automatically
    }
}

/// Parse log level string to LevelFilter
fn parse_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info, // Default to info
    }
}

/// Initialize the custom logger
pub fn init_logger(
    log_file_path: PathBuf,
    flash_tx: Option<Sender<FlashMessage>>,
    file_level: &str,
    flash_level: &str,
) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = log_file_path.parent() {
        fs::create_dir_all(parent).context("Failed to create log directory")?;
    }

    // Create rotating file appender with 10MB max size, keep 3 files
    // Note: tracing-appender doesn't support size-based rotation directly,
    // so we use daily rotation as a reasonable compromise
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(3)
        .filename_prefix(
            log_file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("clipr"),
        )
        .filename_suffix(
            log_file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("log"),
        )
        .build(
            log_file_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid log file path"))?,
        )
        .context("Failed to create rotating file appender")?;

    let file_level = parse_level(file_level);
    let flash_level = parse_level(flash_level);

    let logger = CliprLogger {
        file_writer: Arc::new(Mutex::new(file_appender)),
        flash_tx: flash_tx.map(|tx| Arc::new(Mutex::new(tx))),
        file_level,
        flash_level,
    };

    // Set as global logger
    let max_level = file_level.max(flash_level);
    log::set_boxed_logger(Box::new(logger)).context("Failed to set global logger")?;
    log::set_max_level(max_level);

    Ok(())
}

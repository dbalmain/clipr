use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::{self, Read};

use clipr::clipboard::watch;
use clipr::models::ClipContent;
use clipr::storage::{ensure_directories, BincodeHistoryStorage, HistoryStorage, TomlConfigStorage, ConfigStorage};

enum ContentType {
    Text,
    Image,
}

#[derive(Parser)]
#[command(name = "clipr")]
#[command(about = "Clipboard History Manager TUI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start clipboard watchers (daemon mode)
    Listen,

    /// Store text from stdin (called by text watcher)
    StoreText,

    /// Store image from stdin (called by image watcher)
    StoreImage,

    /// Show clipboard history statistics
    Stats,

    /// Show clipboard history entries
    History {
        /// Number of entries to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Listen) => cmd_listen(),
        Some(Commands::StoreText) => cmd_store_text(),
        Some(Commands::StoreImage) => cmd_store_image(),
        Some(Commands::Stats) => cmd_stats(),
        Some(Commands::History { limit }) => cmd_history(limit),
        None => {
            // Default: launch TUI
            println!("TUI mode not yet implemented. Use --help for available commands.");
            Ok(())
        }
    }
}

/// Start clipboard watchers in background
fn cmd_listen() -> Result<()> {
    log::info!("Starting clipboard watchers");

    // Start text watcher
    watch::start_text_watcher()
        .context("Failed to start text watcher")?;

    // Start image watcher (for US2, but can start now)
    watch::start_image_watcher()
        .context("Failed to start image watcher")?;

    println!("Clipboard watchers started successfully.");
    println!("Use 'ps -ef | grep wl-paste' to see running processes.");
    println!("Use 'pkill -f \"wl-paste.*clipr\"' to stop watchers.");

    Ok(())
}

/// Store text content from stdin
fn cmd_store_text() -> Result<()> {
    store_clip(ContentType::Text)
}

/// Store image content from stdin
fn cmd_store_image() -> Result<()> {
    store_clip(ContentType::Image)
}

/// Store clipboard content from stdin
fn store_clip(content_type: ContentType) -> Result<()> {
    let type_name = match content_type {
        ContentType::Text => "text",
        ContentType::Image => "image",
    };
    log::debug!("Storing clipboard content, type: {}", type_name);

    // Get directories
    let (data_dir, config_dir) = ensure_directories()?;

    // Load config to get max_history
    let config_storage = TomlConfigStorage::new(config_dir.join("clipr.toml"));
    let config = config_storage.load()?;

    // Load existing history
    let history_path = data_dir.join("history.bin");
    let history_storage = BincodeHistoryStorage::new(history_path, config.general.max_history);
    let mut history = history_storage.load()?;

    // Read content from stdin
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .context("Failed to read from stdin")?;

    // Skip if empty
    if buffer.is_empty() {
        log::debug!("Empty clipboard content, skipping");
        return Ok(());
    }

    // Create clip entry based on type
    let content = match content_type {
        ContentType::Text => {
            let text = String::from_utf8(buffer)
                .context("Clipboard text is not valid UTF-8")?;
            ClipContent::Text(text)
        }
        ContentType::Image => {
            ClipContent::Image {
                data: buffer,
                mime_type: "image/png".to_string(),
            }
        }
    };

    // Add to history
    let clip_id = history.add_entry(content);
    log::info!("Stored clip {} (type: {})", clip_id, type_name);

    // Save history
    history_storage.save(&history)?;

    Ok(())
}

/// Show clipboard statistics
fn cmd_stats() -> Result<()> {
    let (data_dir, config_dir) = ensure_directories()?;

    // Load config
    let config_storage = TomlConfigStorage::new(config_dir.join("clipr.toml"));
    let config = config_storage.load()?;

    // Load history
    let history_path = data_dir.join("history.bin");
    let history_storage = BincodeHistoryStorage::new(history_path, config.general.max_history);
    let history = history_storage.load()?;

    // Count by type
    let mut text_count = 0;
    let mut image_count = 0;
    let mut file_count = 0;
    let mut pinned_count = 0;

    for entry in history.entries() {
        match &entry.content {
            ClipContent::Text(_) => text_count += 1,
            ClipContent::Image { .. } => image_count += 1,
            ClipContent::File { .. } => file_count += 1,
        }
        if entry.pinned {
            pinned_count += 1;
        }
    }

    println!("Clipboard History Statistics");
    println!("============================");
    println!("Total entries: {}", history.len());
    println!("  Text: {}", text_count);
    println!("  Images: {}", image_count);
    println!("  Files: {}", file_count);
    println!("Pinned entries: {}", pinned_count);
    println!("Max history: {}", config.general.max_history);

    Ok(())
}

/// Show clipboard history entries
fn cmd_history(limit: usize) -> Result<()> {
    let (data_dir, config_dir) = ensure_directories()?;

    // Load config
    let config_storage = TomlConfigStorage::new(config_dir.join("clipr.toml"));
    let config = config_storage.load()?;

    // Load history
    let history_path = data_dir.join("history.bin");
    let history_storage = BincodeHistoryStorage::new(history_path, config.general.max_history);
    let history = history_storage.load()?;

    println!("Recent Clipboard Entries (showing up to {}):", limit);
    println!("{}", "=".repeat(60));

    for (i, entry) in history.entries().iter().take(limit).enumerate() {
        let type_label = match &entry.content {
            ClipContent::Text(_) => "TEXT",
            ClipContent::Image { .. } => "IMAGE",
            ClipContent::File { .. } => "FILE",
        };

        let preview = entry.preview(50);
        let pinned_mark = if entry.pinned { " 📌" } else { "" };

        println!("{:3}. [{}]{} {}", i + 1, type_label, pinned_mark, preview);
    }

    if history.len() == 0 {
        println!("(empty - no clipboard history yet)");
    }

    Ok(())
}

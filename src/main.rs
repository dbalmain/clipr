use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ratatui::crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Read};
use std::time::Duration;

use clipr::app::App;
use clipr::clipboard::{create_backend, watch};
use clipr::models::{ClipContent, Registry};
use clipr::storage::{
    BincodeHistoryStorage, ConfigStorage, HistoryStorage, TomlConfigStorage, ensure_directories,
};

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

    /// Export a theme to TOML format
    ExportTheme {
        /// Theme name (built-in or custom)
        theme_name: String,
    },

    /// Grab content from a temporary register to clipboard
    GrabTempRegister {
        /// Register key (a-z, A-Z, 0-9)
        register: char,
    },

    /// Grab content from a permanent register to clipboard
    GrabPermRegister {
        /// Register key (a-z, A-Z, 0-9)
        register: char,
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
        Some(Commands::ExportTheme { theme_name }) => cmd_export_theme(&theme_name),
        Some(Commands::GrabTempRegister { register }) => cmd_grab_temp_register(register),
        Some(Commands::GrabPermRegister { register }) => cmd_grab_perm_register(register),
        None => {
            // Default: launch TUI
            cmd_tui()
        }
    }
}

/// Start clipboard watchers in background
fn cmd_listen() -> Result<()> {
    log::info!("Starting clipboard watchers");

    // Start text watcher
    watch::start_text_watcher().context("Failed to start text watcher")?;

    // Start image watcher (for US2, but can start now)
    watch::start_image_watcher().context("Failed to start image watcher")?;

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
            let text = String::from_utf8(buffer).context("Clipboard text is not valid UTF-8")?;
            ClipContent::Text(text)
        }
        ContentType::Image => ClipContent::Image {
            data: buffer,
            mime_type: "image/png".to_string(),
        },
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

    if history.is_empty() {
        println!("(empty - no clipboard history yet)");
    }

    Ok(())
}

/// Export a theme to TOML format
fn cmd_export_theme(theme_name: &str) -> Result<()> {
    use clipr::ui::Theme;

    // Load the theme
    let theme = Theme::load(theme_name).context(format!(
        "Failed to load theme '{}'. Use built-in theme names like 'catppuccin-mocha' or a custom theme file.",
        theme_name
    ))?;

    // Export to TOML and print to stdout
    let toml = theme.to_toml();
    println!("{}", toml);

    Ok(())
}

/// Launch the TUI (default mode)
fn cmd_tui() -> Result<()> {
    // Load state from storage
    let (data_dir, config_dir) = ensure_directories()?;

    // Load config
    let config_storage = TomlConfigStorage::new(config_dir.join("clipr.toml"));
    let config = config_storage.load()?;

    // Load history
    let history_path = data_dir.join("history.bin");
    let history_storage =
        BincodeHistoryStorage::new(history_path.clone(), config.general.max_history);
    let mut history = history_storage.load()?;

    // Create registry and rebuild from loaded history to sync register assignments
    let mut registers = Registry::new();
    registers.rebuild_from_history(&history);

    // Load permanent registers from config into history
    // This ensures permanent register content is always available in history
    registers.load_permanent_from_config(&config, &mut history)?;

    // Rebuild hash map after loading permanent registers
    history.rebuild_hash_map();

    // Create clipboard backend
    let backend = create_backend()?;

    // Create image protocol handler (if terminal supports it)
    let image_protocol = clipr::image::create_image_protocol();

    // Create app
    let mut app = App::new(history, registers, config, backend, image_protocol)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    let result = run_tui(&mut terminal, &mut app);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Save state on exit
    if let Err(e) = &result {
        eprintln!("Error running TUI: {}", e);
    }

    // Save history
    history_storage.save(&app.history)?;

    result
}

/// Run the TUI event loop
fn run_tui<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    // Trigger initial image load
    app.update_image_cache();

    loop {
        // Check for completed image loads
        app.update_image_cache();

        // Check for theme file changes (development mode)
        app.check_theme_reload();

        // Render
        terminal.draw(|f| app.draw(f))?;

        // Handle events with timeout for responsive UI (60fps)
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key)?;
        }

        // Exit check
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Grab content from a temporary register to clipboard
fn cmd_grab_temp_register(register: char) -> Result<()> {
    grab_register(false, register)
}

/// Grab content from a permanent register to clipboard
fn cmd_grab_perm_register(register: char) -> Result<()> {
    grab_register(true, register)
}

/// Common implementation for grab-register commands
fn grab_register(is_permanent: bool, register: char) -> Result<()> {
    // Get directories
    let (data_dir, config_dir) = ensure_directories()?;

    // Load config to get max_history
    let config_storage = TomlConfigStorage::new(config_dir.join("clipr.toml"));
    let config = config_storage.load()?;

    // Load existing history
    let history_path = data_dir.join("history.bin");
    let history_storage = BincodeHistoryStorage::new(history_path, config.general.max_history);
    let mut history = history_storage.load()?;

    // Create and rebuild registry from history to sync register assignments
    let mut registry = Registry::new();
    registry.rebuild_from_history(&history);

    // Load permanent registers from config into history
    registry.load_permanent_from_config(&config, &mut history)?;

    // Get clip ID from register
    let clip_id = if is_permanent {
        registry.get_permanent(register)
    } else {
        registry.get_temporary(register)
    };

    let Some(clip_id) = clip_id else {
        eprintln!("Register '{}' not found", register);
        return Ok(());
    };

    // Get clip content
    let Some(clip) = history.get_entry(clip_id) else {
        eprintln!("Clip {} not found in history", clip_id);
        return Ok(());
    };

    // Create clipboard backend
    let backend = create_backend()?;

    // Copy to clipboard based on content type
    match &clip.content {
        ClipContent::Text(text) => {
            backend.write_text(text)?;
            println!("Copied text from register '{}' to clipboard", register);
        }
        ClipContent::Image { data, .. } => {
            if backend.supports_images() {
                backend.write_image(data)?;
                println!("Copied image from register '{}' to clipboard", register);
            } else {
                eprintln!("Image clipboard not supported by backend");
                return Ok(());
            }
        }
        ClipContent::File { path, .. } => {
            // For files, we copy the file path as text
            backend.write_text(&path.display().to_string())?;
            println!(
                "Copied file path from register '{}' to clipboard: {}",
                register,
                path.display()
            );
        }
    }

    // When run from terminal, add to history for future use
    history.add_entry(clip.content.clone());
    history_storage.save(&history)?;

    Ok(())
}

pub mod protocol;

pub use protocol::ImageProtocol;

/// Create an image protocol handler using ratatui-image
///
/// Uses ratatui-image's Picker which auto-detects terminal capabilities:
/// - Kitty graphics protocol
/// - Sixel protocol
/// - iTerm2 protocol
/// - Halfblocks fallback (always works)
///
/// This always returns Some because Picker falls back to Halfblocks if
/// no proper graphics protocol is available.
pub fn create_image_protocol() -> ImageProtocol {
    let protocol = ImageProtocol::new();
    log::debug!("Created ratatui-image protocol (auto-detected terminal capabilities)");
    protocol
}

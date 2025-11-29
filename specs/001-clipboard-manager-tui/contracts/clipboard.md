# Contract: Clipboard Backend Interface

**Feature**: 001-clipboard-manager-tui
**Component**: `src/clipboard/backend.rs`
**Purpose**: Abstract clipboard operations across display servers (Wayland/X11)

## Interface Definition

```rust
use anyhow::Result;
use crate::models::ClipContent;

/// Clipboard backend abstraction for cross-platform clipboard access
pub trait ClipboardBackend: Send + Sync {
    /// Read text content from clipboard
    ///
    /// **Performance**: MUST complete in <10ms p99
    /// **Error handling**: Return Err if clipboard unavailable or read fails
    fn read_text(&self) -> Result<String>;

    /// Write text content to clipboard
    ///
    /// **Performance**: MUST complete in <10ms p99
    /// **Error handling**: Return Err if clipboard unavailable or write fails
    fn write_text(&self, text: &str) -> Result<()>;

    /// Read image content from clipboard (PNG format)
    ///
    /// **Performance**: MUST complete in <50ms for typical images
    /// **Error handling**: Return Err if no image or read fails
    fn read_image(&self) -> Result<Vec<u8>>;

    /// Write image content to clipboard (PNG format expected)
    ///
    /// **Performance**: MUST complete in <50ms for typical images
    /// **Error handling**: Return Err if clipboard unavailable or write fails
    fn write_image(&self, data: &[u8]) -> Result<()>;

    /// Check if backend supports image operations
    ///
    /// **Returns**: true if read_image/write_image are implemented
    fn supports_images(&self) -> bool;

    /// Get backend name for debugging
    ///
    /// **Returns**: "wayland", "x11", etc.
    fn name(&self) -> &'static str;
}
```

---

## Implementations

### WaylandBackend

**Location**: `src/clipboard/wayland.rs`

**Implementation Strategy**:
- Direct `wl-paste` and `wl-copy` subprocess calls
- Text: `wl-paste --no-newline` for reading
- Text: `wl-copy` with stdin piping for writing
- Images: `wl-paste --type image/png` for reading
- Images: `wl-copy --type image/png` for writing

**Example**:
```rust
pub struct WaylandBackend;

impl ClipboardBackend for WaylandBackend {
    fn read_text(&self) -> Result<String> {
        let output = Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .context("Failed to execute wl-paste")?;

        if !output.status.success() {
            bail!("wl-paste failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        String::from_utf8(output.stdout)
            .context("Clipboard content is not valid UTF-8")
    }

    fn write_text(&self, text: &str) -> Result<()> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn wl-copy")?;

        child.stdin
            .as_mut()
            .unwrap()
            .write_all(text.as_bytes())
            .context("Failed to write to wl-copy stdin")?;

        let status = child.wait().context("wl-copy process failed")?;

        if !status.success() {
            bail!("wl-copy exited with status: {}", status);
        }

        Ok(())
    }

    fn supports_images(&self) -> bool { true }
    fn name(&self) -> &'static str { "wayland" }
}
```

**Performance Requirements**:
- Text operations: 2-5ms typical
- Image operations: 20-40ms typical
- No blocking on UI thread

---

### X11Backend

**Location**: `src/clipboard/x11.rs`

**Implementation Strategy**:
- Use `arboard` crate for cross-platform abstraction
- Handles X11 selection complexity (PRIMARY vs CLIPBOARD)
- Fallback implementation for non-Wayland systems

**Example**:
```rust
use arboard::Clipboard;

pub struct X11Backend {
    clipboard: Clipboard,
}

impl X11Backend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            clipboard: Clipboard::new()
                .context("Failed to initialize clipboard")?,
        })
    }
}

impl ClipboardBackend for X11Backend {
    fn read_text(&self) -> Result<String> {
        self.clipboard.get_text()
            .context("Failed to read clipboard text")
    }

    fn write_text(&self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)
            .context("Failed to write clipboard text")
    }

    fn read_image(&self) -> Result<Vec<u8>> {
        let img = self.clipboard.get_image()
            .context("Failed to read clipboard image")?;

        // Convert to PNG
        let mut buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buf);
        encoder.write_image(&img.bytes, img.width, img.height, image::ColorType::Rgba8)
            .context("Failed to encode image as PNG")?;

        Ok(buf)
    }

    fn write_image(&self, data: &[u8]) -> Result<()> {
        let img = image::load_from_memory(data)
            .context("Failed to decode image")?;

        let rgba = img.to_rgba8();

        self.clipboard.set_image(arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: rgba.into_raw().into(),
        }).context("Failed to write clipboard image")
    }

    fn supports_images(&self) -> bool { true }
    fn name(&self) -> &'static str { "x11" }
}
```

---

## Factory Pattern

**Location**: `src/clipboard/mod.rs`

```rust
/// Detect display server and create appropriate backend
pub fn create_backend() -> Result<Box<dyn ClipboardBackend>> {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        debug!("Detected Wayland display server");
        Ok(Box::new(WaylandBackend))
    } else if env::var("DISPLAY").is_ok() {
        debug!("Detected X11 display server");
        Ok(Box::new(X11Backend::new()?))
    } else {
        bail!("No display server detected (neither WAYLAND_DISPLAY nor DISPLAY set)")
    }
}
```

---

## Clipboard Monitoring

**Location**: `src/clipboard/monitor.rs`

**Polling Strategy**:
```rust
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

pub struct ClipboardMonitor {
    backend: Box<dyn ClipboardBackend>,
    last_hash: u64,
    poll_interval: Duration,
}

impl ClipboardMonitor {
    /// Start monitoring clipboard changes in background thread
    pub fn start(backend: Box<dyn ClipboardBackend>, poll_interval_ms: u64) -> Sender<ClipContent> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let mut monitor = ClipboardMonitor {
                backend,
                last_hash: 0,
                poll_interval: Duration::from_millis(poll_interval_ms),
            };

            loop {
                if let Ok(content) = monitor.check_clipboard() {
                    if tx.send(content).is_err() {
                        break;  // Receiver dropped, exit thread
                    }
                }

                thread::sleep(monitor.poll_interval);
            }
        });

        rx
    }

    /// Check if clipboard changed and return new content
    fn check_clipboard(&mut self) -> Result<Option<ClipContent>> {
        // Try reading text first (more common)
        if let Ok(text) = self.backend.read_text() {
            let hash = Self::hash_content(&text);
            if hash != self.last_hash {
                self.last_hash = hash;
                return Ok(Some(ClipContent::Text(text)));
            }
        }

        // Then try reading image if supported
        if self.backend.supports_images() {
            if let Ok(image_data) = self.backend.read_image() {
                let hash = Self::hash_content(&image_data);
                if hash != self.last_hash {
                    self.last_hash = hash;
                    return Ok(Some(ClipContent::Image(image_data)));
                }
            }
        }

        Ok(None)
    }

    fn hash_content<T: AsRef<[u8]>>(content: T) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.as_ref().hash(&mut hasher);
        hasher.finish()
    }
}
```

---

## Error Handling

**Expected Errors**:
1. **ClipboardUnavailable**: Display server not responding
2. **InvalidFormat**: Content not in expected format
3. **AccessDenied**: Permission issues (rare on Linux)
4. **TimeoutError**: Operation took too long (>100ms)

**Error Messages**:
```rust
// User-facing error messages (displayed in TUI)
match backend.read_text() {
    Err(e) if e.to_string().contains("wl-paste") => {
        "Clipboard tool (wl-paste) not available. Install wl-clipboard package."
    }
    Err(e) if e.to_string().contains("UTF-8") => {
        "Clipboard contains non-text data"
    }
    Err(e) => {
        format!("Failed to read clipboard: {}", e)
    }
}
```

---

## Testing Contract

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_round_trip_text() {
        let backend = WaylandBackend;
        let test_text = "Hello, clipboard!";

        backend.write_text(test_text).unwrap();
        let read_text = backend.read_text().unwrap();

        assert_eq!(test_text, read_text);
    }

    #[test]
    fn test_image_support() {
        let backend = WaylandBackend;
        assert!(backend.supports_images());
    }
}
```

### Integration Tests

**Location**: `tests/integration/clipboard_ops.rs`

```rust
#[test]
fn test_clipboard_monitoring() {
    let backend = create_backend().unwrap();
    let rx = ClipboardMonitor::start(backend, 100);

    // Simulate clipboard change (external tool)
    std::process::Command::new("wl-copy")
        .arg("test content")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // Wait for monitor to detect change
    let content = rx.recv_timeout(Duration::from_secs(1)).unwrap();

    assert!(matches!(content, ClipContent::Text(s) if s == "test content"));
}
```

---

## Performance Benchmarks

**Location**: `benches/clipboard_operations.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_clipboard_read(c: &mut Criterion) {
    let backend = create_backend().unwrap();

    c.bench_function("clipboard_read_text", |b| {
        b.iter(|| backend.read_text())
    });
}

fn bench_clipboard_write(c: &mut Criterion) {
    let backend = create_backend().unwrap();
    let test_text = "benchmark data";

    c.bench_function("clipboard_write_text", |b| {
        b.iter(|| backend.write_text(test_text))
    });
}

criterion_group!(benches, bench_clipboard_read, bench_clipboard_write);
criterion_main!(benches);
```

**Expected Results**:
- Wayland text read: 2-5ms
- Wayland text write: 2-5ms
- Wayland image read: 20-40ms
- Wayland image write: 20-40ms
- All within <10ms p99 for text, <50ms for images

---

## Platform Dependencies

### Wayland
```bash
# Required system packages
sudo apt install wl-clipboard  # Debian/Ubuntu
sudo pacman -S wl-clipboard    # Arch
```

### X11
```toml
[dependencies]
arboard = "3.4"  # Handles X11 selections automatically
```

---

## Future Enhancements

1. **Event-based monitoring**: Wayland protocol listeners (complex, defer to P4/P5)
2. **Rich text support**: HTML clipboard content
3. **Multi-target clipboard**: Different formats simultaneously
4. **Clipboard history sync**: Across multiple machines (far future)

**Contract Complete** ✅

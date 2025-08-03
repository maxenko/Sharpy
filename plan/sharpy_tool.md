# Sharpy Tool - CLI Application Plan

## Overview
A command-line tool that provides access to all sharpy library features through an easy-to-use executable interface.

## Architecture

### Binary Structure
- Location: `src/bin/sharpy_tool.rs`
- Dependencies:
  - `clap` (v4) - CLI argument parsing
  - `indicatif` - Progress bars
  - `glob` - File pattern matching
  - `anyhow` - Error handling

### Command Structure

#### Basic Commands
```bash
# Unsharp Mask
sharpy_tool unsharp <INPUT> <OUTPUT> [OPTIONS]
  -r, --radius <RADIUS>       Blur radius (0.5-10.0) [default: 1.0]
  -a, --amount <AMOUNT>       Sharpening strength (0.0-5.0) [default: 1.0]
  -t, --threshold <THRESHOLD> Minimum difference threshold (0-255) [default: 0]

# High-Pass Sharpening
sharpy_tool highpass <INPUT> <OUTPUT> [OPTIONS]
  -s, --strength <STRENGTH>   Blend strength (0.0-3.0) [default: 0.5]

# Edge Enhancement
sharpy_tool edges <INPUT> <OUTPUT> [OPTIONS]
  -s, --strength <STRENGTH>   Enhancement strength (0.0-3.0) [default: 1.0]
  -m, --method <METHOD>       Edge detection method (sobel|prewitt) [default: sobel]

# Clarity Enhancement
sharpy_tool clarity <INPUT> <OUTPUT> [OPTIONS]
  -s, --strength <STRENGTH>   Enhancement strength (0.0-3.0) [default: 1.0]
  -r, --radius <RADIUS>       Local area radius (1.0-20.0) [default: 2.0]

# Preset Operations
sharpy_tool preset <INPUT> <OUTPUT> [OPTIONS]
  -p, --preset <PRESET>       Preset name (subtle|moderate|strong|edge-aware|portrait|landscape)
```

#### Advanced Commands
```bash
# Batch Processing with Multiple Operations
sharpy_tool batch <INPUT> <OUTPUT> [OPTIONS]
  -o, --operations <OPS>      Comma-separated operations
  
  Example: --operations "unsharp:1.0:1.0:0,clarity:0.5:2.0"

# Process Multiple Files
sharpy_tool process <PATTERN> [OPTIONS]
  -o, --output-dir <DIR>      Output directory
  -p, --prefix <PREFIX>       Output filename prefix
  -s, --suffix <SUFFIX>       Output filename suffix [default: "_sharp"]
  
  Example: sharpy_tool process "images/*.jpg" -o processed/ -s "_enhanced"
```

### Global Options
```bash
-v, --verbose               Verbose output
-q, --quiet                 Suppress all output except errors
--dry-run                   Preview operations without processing
--format <FORMAT>           Force output format (jpg|png|webp|bmp)
--quality <QUALITY>         JPEG quality (1-100) [default: 95]
--overwrite                 Overwrite existing files without prompting
--threads <N>               Number of parallel threads [default: auto]
```

## Features

### 1. Smart File Handling
- Auto-detect input format
- Preserve metadata when possible
- Support for common formats: JPEG, PNG, WebP, BMP, TIFF
- Intelligent output naming

### 2. Batch Processing
- Process entire directories
- Glob pattern support
- Progress bar with ETA
- Parallel file processing

### 3. Operation Chaining
- Apply multiple operations in sequence
- Save operation recipes
- Load operation presets from JSON

### 4. Error Handling
- Graceful error messages
- Continue on error option for batch operations
- Detailed error logs with --verbose

### 5. Interactive Mode (Future)
```bash
sharpy_tool interactive <INPUT>
```
- Preview changes in real-time
- Adjust parameters interactively
- Save final parameters as preset

## Implementation Details

### Main Binary Structure
```rust
// src/bin/sharpy_tool.rs

use clap::{Parser, Subcommand};
use sharpy::{Image, EdgeMethod, SharpeningPresets};
use anyhow::{Result, Context};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "sharpy_tool")]
#[command(about = "High-performance image sharpening tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    Unsharp {
        input: String,
        output: String,
        #[arg(short, long, default_value = "1.0")]
        radius: f32,
        #[arg(short, long, default_value = "1.0")]
        amount: f32,
        #[arg(short, long, default_value = "0")]
        threshold: u8,
    },
    // ... other commands
}
```

### Cargo.toml Additions
```toml
[[bin]]
name = "sharpy_tool"
path = "src/bin/sharpy_tool.rs"

[dependencies]
# ... existing dependencies
clap = { version = "4.5", features = ["derive"] }
indicatif = "0.17"
glob = "0.3"
anyhow = "1.0"
```

## Build Instructions

### Development
```bash
cargo build --bin sharpy_tool
```

### Release Build
```bash
# Optimized for performance
cargo build --release --bin sharpy_tool

# Output location:
# Windows: target/release/sharpy_tool.exe
# Linux/Mac: target/release/sharpy_tool
```

### Cross-Platform Building
```bash
# Install cross
cargo install cross

# Build for Linux
cross build --release --target x86_64-unknown-linux-gnu --bin sharpy_tool

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu --bin sharpy_tool
```

## Usage Examples

### Basic Usage
```bash
# Simple sharpening
sharpy_tool unsharp photo.jpg photo_sharp.jpg

# With custom parameters
sharpy_tool unsharp photo.jpg photo_sharp.jpg -r 2.0 -a 1.5 -t 10

# Using presets
sharpy_tool preset portrait.jpg portrait_enhanced.jpg -p portrait
```

### Batch Processing
```bash
# Process all JPEGs in a directory
sharpy_tool process "photos/*.jpg" -o enhanced/ -s "_sharp"

# Apply multiple operations
sharpy_tool batch input.jpg output.jpg -o "unsharp:1.0:1.0:0,clarity:0.5:2.0"
```

### Advanced Usage
```bash
# Dry run to see what would happen
sharpy_tool process "*.jpg" --dry-run

# Verbose output for debugging
sharpy_tool unsharp photo.jpg out.jpg -v

# Force PNG output
sharpy_tool unsharp photo.jpg photo_sharp.png --format png
```

## Future Enhancements

1. **Configuration Files**
   - `.sharpyrc` for default settings
   - Operation preset files in JSON/YAML

2. **GUI Version**
   - Simple GUI wrapper using `egui`
   - Drag-and-drop support
   - Real-time preview

3. **Plugin System**
   - Custom sharpening algorithms
   - External filter support

4. **Performance Features**
   - GPU acceleration support
   - Memory-mapped file I/O for large images
   - Streaming processing for very large images

5. **Integration Features**
   - Watch mode for automatic processing
   - HTTP API server mode
   - Integration with image CDNs
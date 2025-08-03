# Sharpy

High-performance image sharpening library and CLI tool for Rust.

## Quick Start

### Library Usage

```rust
use sharpy::Image;

// Load and sharpen an image
let image = Image::load("photo.jpg")?;
let sharpened = image.unsharp_mask(1.0, 1.0, 0)?;
sharpened.save("photo_sharp.jpg")?;
```

### CLI Usage

```bash
# Install the CLI tool
cargo install sharpy

# Sharpen an image
sharpy unsharp photo.jpg photo_sharp.jpg

# Use a preset
sharpy preset portrait.jpg portrait_enhanced.jpg -p portrait
```

## Features

- **Performance-focused** - Parallel processing with Rayon
- **Multiple algorithms** - Unsharp mask, high-pass, edge enhancement, clarity
- **Flexible API** - Builder pattern for complex workflows
- **Minimal dependencies** - Core functionality with carefully selected dependencies
- **CLI included** - Full-featured command-line tool

## Installation

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
sharpy = "0.1"
```

### As a CLI Tool

```bash
cargo install sharpy
```

Or build from source:

```bash
git clone https://github.com/maxenko/sharpy
cd sharpy
cargo build --release
```

## Library Usage

### Basic Sharpening

```rust
use sharpy::{Image, EdgeMethod};

// Unsharp mask - the classic sharpening method
let image = Image::load("input.jpg")?;
let sharpened = image.unsharp_mask(
    1.0,  // radius
    1.0,  // amount
    0     // threshold
)?;

// High-pass sharpening
let sharpened = image.high_pass_sharpen(0.5)?;

// Edge enhancement
let sharpened = image.enhance_edges(1.0, EdgeMethod::Sobel)?;

// Clarity (local contrast enhancement)
let sharpened = image.clarity(1.0, 2.0)?;
```

### Using the Builder Pattern

```rust
use sharpy::{Image, EdgeMethod};

let result = Image::load("landscape.jpg")?
    .sharpen()
    .unsharp_mask(1.0, 1.2, 1)
    .edge_enhance(0.5, EdgeMethod::Sobel)
    .clarity(0.4, 3.0)
    .apply()?;

result.save("landscape_enhanced.jpg")?;
```

### Using Presets

```rust
use sharpy::{Image, SharpeningPresets};

// Built-in presets for common use cases
let image = Image::load("photo.jpg")?;

// Subtle sharpening
let result = SharpeningPresets::subtle(image).apply()?;

// Portrait enhancement (avoids over-sharpening skin)
let result = SharpeningPresets::portrait(image).apply()?;

// Landscape enhancement (enhanced detail)
let result = SharpeningPresets::landscape(image).apply()?;
```

### Advanced Examples

#### Custom Sharpening Pipeline

```rust
use sharpy::{Image, SharpeningBuilder, EdgeMethod};

fn custom_enhancement(image: Image) -> sharpy::Result<Image> {
    image.sharpen()
        // Start with subtle unsharp mask
        .unsharp_mask(0.8, 0.6, 2)
        // Add edge enhancement
        .edge_enhance(0.3, EdgeMethod::Sobel)
        // Finish with clarity for local contrast
        .clarity(0.5, 5.0)
        .apply()
}
```

#### Processing Multiple Images

```rust
use sharpy::Image;
use rayon::prelude::*;
use std::path::Path;

fn batch_process(input_dir: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let images: Vec<_> = std::fs::read_dir(input_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .map(|ext| ext == "jpg" || ext == "png")
                .unwrap_or(false)
        })
        .collect();

    images.par_iter().try_for_each(|entry| -> Result<(), Box<dyn std::error::Error>> {
        let path = entry.path();
        let image = Image::load(&path)?;
        
        let enhanced = image.unsharp_mask(1.0, 1.0, 0)?;
        
        let output_path = output_dir.join(path.file_name().unwrap());
        enhanced.save(output_path)?;
        
        Ok(())
    })?;
    
    Ok(())
}
```

#### Working with Image Data

```rust
use sharpy::Image;
use image::{RgbImage, DynamicImage};

// From various image types
let rgb_image = RgbImage::new(800, 600);
let image = Image::from_rgb(rgb_image);

let dynamic_image = DynamicImage::new_rgb8(800, 600);
let image = Image::from_dynamic(dynamic_image);

// Get dimensions and histogram
let (width, height) = image.dimensions();
let histogram = image.histogram(); // [u32; 256] luminance histogram

// Convert back to standard image types
let rgb: RgbImage = image.clone().into_rgb();
let dynamic: DynamicImage = image.into_dynamic();
```

## CLI Tool (sharpy)

### Basic Commands

```bash
# Unsharp mask with default settings
sharpy unsharp input.jpg output.jpg

# Specify parameters
sharpy unsharp input.jpg output.jpg -r 2.0 -a 1.5 -t 10

# High-pass sharpening
sharpy highpass input.jpg output.jpg -s 0.7

# Edge enhancement
sharpy edges input.jpg output.jpg -s 1.0 -m sobel

# Clarity enhancement
sharpy clarity input.jpg output.jpg -s 1.0 -r 3.0

# Use a preset
sharpy preset photo.jpg enhanced.jpg -p moderate
```

### Available Presets

- `subtle` - Light sharpening for general use
- `moderate` - Balanced sharpening with clarity
- `strong` - Heavy sharpening for soft images
- `edge-aware` - Emphasizes edges while preserving smooth areas
- `portrait` - Optimized for portraits (avoids over-sharpening skin)
- `landscape` - Enhanced detail extraction for landscapes

### Batch Processing

```bash
# Process all JPG files in current directory
sharpy batch "*.jpg" -o sharpened/

# Process with custom suffix
sharpy batch "photos/*.jpg" -o processed/ -s "_enhanced"

# Apply multiple operations
sharpy batch "*.png" -o output/ -p "unsharp:1.0:1.0:0,clarity:0.5:2.0"
```

### Advanced CLI Usage

#### Dry Run Mode

```bash
# Preview what would happen without processing
sharpy batch "*.jpg" -o processed/ --dry-run
```

#### Verbose Output

```bash
# See detailed processing information
sharpy unsharp photo.jpg sharp.jpg -v
```

#### Overwrite Protection

```bash
# Force overwrite existing files
sharpy unsharp input.jpg output.jpg --overwrite
```

#### Chaining Operations in Batch Mode

```bash
# Format: "operation:param1:param2:..."
sharpy batch "*.jpg" -o enhanced/ -p "unsharp:1.0:1.0:0,edges:0.5:sobel,clarity:0.3:2.0"
```

Operation formats:
- `unsharp:radius:amount:threshold`
- `highpass:strength`
- `edges:strength:method` (method: sobel or prewitt)
- `clarity:strength:radius`

### CLI Examples by Use Case

#### Portrait Photography

```bash
# Gentle sharpening for portraits
sharpy preset portrait.jpg portrait_final.jpg -p portrait

# Custom portrait enhancement
sharpy unsharp portrait.jpg enhanced.jpg -r 1.2 -a 0.7 -t 10
```

#### Landscape Photography

```bash
# Enhanced detail for landscapes
sharpy preset landscape.jpg landscape_final.jpg -p landscape

# Custom landscape workflow
sharpy batch "landscapes/*.jpg" -o final/ -p "unsharp:1.0:1.2:1,edges:0.5:sobel,clarity:0.4:3.0"
```

#### Web Images

```bash
# Batch process for web upload
sharpy batch "products/*.jpg" -o web/ -p "unsharp:0.8:0.8:2,clarity:0.3:2.0"
```

#### Scanned Documents

```bash
# Enhance text clarity
sharpy edges scan.png scan_enhanced.png -s 1.5 -m prewitt
```

## Performance

Sharpy uses parallel processing for optimal performance:

- Separable convolution for Gaussian blur
- Parallel pixel processing with Rayon
- Efficient memory usage with copy-on-write
- Optimized memory operations

Benchmark results on typical hardware (1024x1024 image):
- Unsharp mask: ~45ms
- High-pass sharpen: ~25ms
- Edge enhancement: ~35ms
- Clarity: ~65ms

*Performance may vary based on hardware and image characteristics.

## Algorithm Details

### Unsharp Mask
Creates a blurred version of the image and subtracts it from the original to enhance edges.

Parameters:
- `radius`: Blur radius (0.5-10.0)
- `amount`: Strength multiplier (0.0-5.0)
- `threshold`: Minimum difference to sharpen (0-255)

### High-Pass Sharpen
Uses a 3x3 convolution kernel to enhance high-frequency details.

Parameters:
- `strength`: Blend with original (0.0-3.0)

### Edge Enhancement
Detects edges using Sobel or Prewitt operators and enhances them.

Parameters:
- `strength`: Enhancement amount (0.0-3.0)
- `method`: Edge detection algorithm (Sobel/Prewitt)

### Clarity
Enhances local contrast by comparing each pixel to its surrounding area.

Parameters:
- `strength`: Enhancement amount (0.0-3.0)
- `radius`: Local area size (1.0-20.0)

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/sharpy
cd sharpy

# Build library and CLI
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Install CLI globally
cargo install --path .
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [image](https://github.com/image-rs/image) crate for image I/O
- Parallel processing with [rayon](https://github.com/rayon-rs/rayon)
- CLI interface using [clap](https://github.com/clap-rs/clap)
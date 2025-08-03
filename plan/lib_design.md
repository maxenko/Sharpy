# Rust Image Editing Library Design

A high-performance, memory-efficient image editing library for Rust with fluent API, multithreading support via Rayon, and smart ownership handling for both owned and shared images.

## Core Design Goals

- **Fluent API**: `image.brighten(1.2).sharpen(0.3).contrast(1.1)`
- **Zero-copy when possible**: Take ownership of `DynamicImage`, defer cloning for `Arc<DynamicImage>`
- **Parallel processing**: All operations use Rayon for multi-core performance
- **Memory efficient**: Copy-on-write for shared images, in-place mutations when possible
- **Flexible input types**: Support both `DynamicImage` (owned) and `Arc<DynamicImage>` (shared)

## Architecture Overview

### Internal Data Structure

```rust
enum ImageData {
    Owned(RgbImage),        // Zero-copy path for owned images
    Shared(Arc<RgbImage>),  // Copy-on-write path for shared images
}

pub struct Image {
    data: ImageData,
}
```

The library uses an internal enum to handle both ownership models transparently:
- **Owned**: Direct mutations, no cloning needed
- **Shared**: Lazy cloning on first mutation (copy-on-write)

### Smart Ownership Logic

```rust
impl ImageData {
    fn get_mut(&mut self) -> &mut RgbImage {
        match self {
            ImageData::Owned(ref mut img) => img,  // Direct access
            ImageData::Shared(arc_img) => {
                // Clone only when first mutation happens
                *self = ImageData::Owned((**arc_img).clone());
                // Return mutable reference to newly owned data
            }
        }
    }
}
```

## API Design

### Constructors

**Consuming (Zero-Copy When Possible)**
```rust
// Takes ownership - zero copy
Image::from_dynamic(img: DynamicImage) -> Image
Image::from_rgb(img: RgbImage) -> Image  
Image::load(path: P) -> Result<Image, Error>

// Smart Arc handling - tries Arc::try_unwrap first
Image::from_arc_dynamic(arc_img: Arc<DynamicImage>) -> Image
Image::from_arc_rgb(arc_img: Arc<RgbImage>) -> Image
```

**Non-Consuming**
```rust
// Always clones
Image::from_dynamic_ref(img: &DynamicImage) -> Image
```

### Fluent Operations (All Consuming)

```rust
impl Image {
    pub fn brighten(mut self, factor: f32) -> Self
    pub fn contrast(mut self, factor: f32) -> Self  
    pub fn sharpen(mut self, strength: f32) -> Self
    pub fn blur(mut self, radius: f32) -> Self
    pub fn saturate(mut self, factor: f32) -> Self
    pub fn gamma(mut self, gamma: f32) -> Self
}
```

All operations:
- Use `mut self` (consuming) to enable chaining
- Call `self.data.get_mut()` which triggers copy-on-write if needed
- Use Rayon's parallel iterators (`par_iter_mut()`) for performance
- Return `Self` for continued chaining

### Terminal Operations

```rust
// Convert back to various formats
pub fn into_arc_dynamic(self) -> Arc<DynamicImage>
pub fn into_dynamic(self) -> DynamicImage  
pub fn into_rgb(self) -> RgbImage

// Save to file
pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), Error>

// Non-consuming read-only operations
pub fn dimensions(&self) -> (u32, u32)
pub fn histogram(&self) -> [u32; 256]
```

## Parallel Processing Implementation

### Pixel-wise Operations
```rust
pub fn brighten(mut self, factor: f32) -> Self {
    let buffer = self.data.get_mut(); // Copy-on-write happens here
    
    buffer
        .pixels_mut()
        .par_iter_mut()  // Rayon parallel iterator
        .for_each(|pixel| {
            let [r, g, b] = pixel.0;
            *pixel = Rgb([
                (r as f32 * factor).clamp(0.0, 255.0) as u8,
                (g as f32 * factor).clamp(0.0, 255.0) as u8,
                (b as f32 * factor).clamp(0.0, 255.0) as u8,
            ]);
        });
    
    self
}
```

### Convolution Operations
```rust
pub fn sharpen(mut self, strength: f32) -> Self {
    let buffer = self.data.get_mut();
    let (width, height) = buffer.dimensions();
    let original = buffer.clone(); // Temporary copy needed for reading
    
    buffer
        .enumerate_rows_mut()
        .par_bridge()  // Parallel row processing
        .for_each(|(y, row)| {
            for (x, pixel) in row.enumerate() {
                *pixel = apply_kernel(&original, x as u32, y, &kernel, strength, width, height);
            }
        });
    
    self
}
```

## Usage Examples

### Owned Image Path (Zero-Copy)
```rust
// Loading and processing owned image - no unnecessary copies
let result = Image::load("input.jpg")?
    .brighten(1.2)
    .sharpen(0.3)
    .contrast(1.1)
    .save("output.jpg")?;

// From existing DynamicImage - takes ownership
let owned_img = image::open("photo.jpg")?;
let processed = Image::from_dynamic(owned_img)
    .blur(2.0)
    .brighten(0.9)
    .into_dynamic();
```

### Shared Image Path (Copy-on-Write)
```rust
// Multiple variants from same source
let arc_img: Arc<DynamicImage> = Arc::new(image::open("source.jpg")?);

// Each variant clones only when first mutation happens
let bright_version = Image::from_arc_dynamic(Arc::clone(&arc_img))
    .brighten(1.5)  // Clone happens here
    .sharpen(0.2)   // No additional clone
    .into_arc_dynamic();

let dark_version = Image::from_arc_dynamic(Arc::clone(&arc_img))
    .brighten(0.7)  // Another clone happens here
    .contrast(1.3)
    .into_arc_dynamic();

// Original Arc is still available and unchanged
```

### Smart Ownership Optimization
```rust
// If this consumes the last Arc reference, no clone needed
let final_version = Image::from_arc_dynamic(arc_img) // Moves arc_img
    .brighten(1.1)  // Might avoid clone via Arc::try_unwrap
    .save("final.jpg")?;
```

### Real-World Pattern
```rust
struct ImageProcessor {
    source: Arc<DynamicImage>,
}

impl ImageProcessor {
    fn new(img: DynamicImage) -> Self {
        Self { source: Arc::new(img) }
    }
    
    fn create_variants(&self) -> Vec<Arc<DynamicImage>> {
        vec![
            Image::from_arc_dynamic(Arc::clone(&self.source))
                .brighten(1.3)
                .into_arc_dynamic(),
                
            Image::from_arc_dynamic(Arc::clone(&self.source))
                .contrast(1.5)
                .sharpen(0.4)
                .into_arc_dynamic(),
                
            Image::from_arc_dynamic(Arc::clone(&self.source))
                .blur(2.0)
                .brighten(0.8)
                .into_arc_dynamic(),
        ]
    }
}
```

## Performance Characteristics

### Memory Efficiency
- **Owned path**: Zero allocations during operation chain
- **Shared path**: Single clone on first mutation, then zero allocations
- **Convolution ops**: One temporary clone for reading during kernel application
- **Smart Arc handling**: `Arc::try_unwrap` avoids clone when possible

### CPU Performance  
- **Parallel processing**: All operations use Rayon for multi-core execution
- **SIMD potential**: Rayon enables auto-vectorization
- **Cache efficiency**: Row-wise processing for convolution operations
- **Memory bandwidth**: Often the limiting factor, not CPU

### Scaling
- **Thread count**: Rayon automatically scales to available cores
- **Custom thread pools**: Possible for fine-grained control
- **Chunk processing**: `par_chunks_mut()` for cache-friendly processing

## Dependencies

```toml
[dependencies]
image = "0.24"
rayon = "1.7"
```

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image format error: {0}")]
    Format(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, ImageError>;
```

## Key Implementation Details

### Copy-on-Write Trigger
```rust
impl ImageData {
    fn get_mut(&mut self) -> &mut RgbImage {
        match self {
            ImageData::Owned(ref mut img) => img,
            ImageData::Shared(arc_img) => {
                // This is where the magic happens - clone only when needed
                *self = ImageData::Owned((**arc_img).clone());
                if let ImageData::Owned(ref mut img) = self {
                    img
                } else {
                    unreachable!()
                }
            }
        }
    }
}
```

### Parallel Convolution Pattern
```rust
// Row-wise parallel processing for cache efficiency
buffer
    .enumerate_rows_mut()
    .par_bridge()
    .for_each(|(y, row)| {
        for (x, pixel) in row.enumerate() {
            *pixel = apply_convolution(&original, x as u32, y, &kernel);
        }
    });
```

This design provides maximum flexibility and performance for both owned and shared image processing workflows while maintaining a clean, idiomatic Rust API.
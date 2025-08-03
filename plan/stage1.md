# Stage 1 Implementation: Sharpening Methods

## Overview
Implement the core architecture and all 4 sharpening methods following the lib_design.md specification.

## Implementation Order

### 1. Dependencies & Core Architecture
- Add image, rayon, thiserror to Cargo.toml
- Implement ImageData enum with Owned/Shared variants
- Implement Image struct with smart ownership
- Add error handling types
- Constructor methods (from_dynamic, from_rgb, etc.)

### 2. Utility Functions
- Convolution kernel utilities
- Parallel processing helpers
- Pixel manipulation functions
- Boundary handling for convolution

### 3. Sharpening Methods

#### 3.1 Unsharp Masking
**Algorithm**: Original + Amount × (Original - GaussianBlur(Original))
- Parameters: radius (0.5-5.0), amount (0.1-3.0), threshold (0-255)
- Implementation:
  1. Create Gaussian blur of original
  2. Subtract blur from original to get mask
  3. Scale mask by amount parameter
  4. Add scaled mask back to original
  5. Apply threshold to prevent over-sharpening in smooth areas

#### 3.2 High Pass Sharpening
**Algorithm**: Frequency-domain sharpening using convolution
- Parameters: strength (0.1-2.0)
- High-pass kernel:
  ```
  [ 0, -1,  0]
  [-1,  5, -1]
  [ 0, -1,  0]
  ```
- Implementation:
  1. Apply high-pass convolution kernel
  2. Blend result with original based on strength
  3. Clamp values to valid range

#### 3.3 Edge Enhancement
**Algorithm**: Edge detection + amplification
- Parameters: strength (0.1-2.0), method (Sobel/Prewitt)
- Sobel kernels:
  ```
  X: [-1, 0, 1]    Y: [-1, -2, -1]
     [-2, 0, 2]       [ 0,  0,  0]
     [-1, 0, 1]       [ 1,  2,  1]
  ```
- Implementation:
  1. Apply Sobel/Prewitt edge detection
  2. Calculate edge magnitude
  3. Amplify edges based on strength
  4. Blend with original image

#### 3.4 Clarity/Structure
**Algorithm**: Mid-tone contrast enhancement
- Parameters: strength (0.1-2.0), radius (1.0-10.0)
- Implementation:
  1. Create luminance channel
  2. Apply local contrast enhancement using sliding window
  3. Enhance mid-tone contrast (avoid shadows/highlights)
  4. Apply back to color channels

## Parallel Processing Strategy

### Row-wise Processing
```rust
buffer
    .enumerate_rows_mut()
    .par_bridge()
    .for_each(|(y, row)| {
        // Process entire row
    });
```

### Pixel-wise Processing
```rust
buffer
    .pixels_mut()
    .par_iter_mut()
    .for_each(|pixel| {
        // Process individual pixels
    });
```

## File Structure
```
src/
├── lib.rs           # Core architecture, Image struct, constructors
├── sharpening.rs    # All 4 sharpening implementations
└── utils.rs         # Convolution, kernels, pixel helpers
```

## API Design
```rust
impl Image {
    // Unsharp masking
    pub fn unsharp_mask(self, radius: f32, amount: f32, threshold: u8) -> Self
    
    // High pass sharpening
    pub fn high_pass_sharpen(self, strength: f32) -> Self
    
    // Edge enhancement
    pub fn enhance_edges(self, strength: f32, method: EdgeMethod) -> Self
    
    // Clarity/structure
    pub fn clarity(self, strength: f32, radius: f32) -> Self
}

pub enum EdgeMethod {
    Sobel,
    Prewitt,
}
```

## Testing Strategy
- Unit tests for each sharpening method
- Test edge cases (small images, extreme parameters)
- Benchmark performance with different image sizes
- Memory usage validation for owned vs shared paths

## Performance Targets
- Multi-core scaling with Rayon
- Zero-copy for owned images
- Single clone for shared images
- Memory-efficient convolution operations

## Implementation Notes
- All operations maintain the fluent API pattern
- Copy-on-write triggers only on first mutation
- Convolution operations need temporary clone for reading
- Parameter validation with appropriate error messages
- Clamp all pixel values to valid ranges [0, 255]